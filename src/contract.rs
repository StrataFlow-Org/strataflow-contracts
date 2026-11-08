use crate::errors::Error;
use crate::events::*;
use crate::storage::*;
use crate::types::*;
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, String, Vec};

#[contract]
pub struct StrataFlowContract;

#[contractimpl]
impl StrataFlowContract {
    /// Initialize the protocol with an admin address
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AgreementCounter, &0u64);
        env.storage().instance().set(&DataKey::IsPaused, &false);
        bump_instance_ttl(&env);
        Ok(())
    }

    /// Set protocol fee parameters (basis points and recipient)
    pub fn set_fee_config(env: Env, fee_bps: u32, recipient: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        bump_instance_ttl(&env);

        if fee_bps > MAX_FEE_BPS {
            return Err(Error::InvalidFeeBps);
        }

        let config = ProtocolFeeConfig {
            fee_bps,
            recipient: recipient.clone(),
        };

        env.storage().instance().set(&DataKey::FeeConfig, &config);
        publish_fee_config(&env, fee_bps, recipient);
        Ok(())
    }

    /// Query current protocol fee configuration
    pub fn get_fee_config(env: Env) -> Option<ProtocolFeeConfig> {
        env.storage().instance().get(&DataKey::FeeConfig)
    }

    /// Circuit breaker: pause or unpause contract operations
    pub fn set_pause(env: Env, paused: bool) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        bump_instance_ttl(&env);

        env.storage().instance().set(&DataKey::IsPaused, &paused);
        publish_pause(&env, paused);
        Ok(())
    }

    /// Check if the protocol is currently paused
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::IsPaused)
            .unwrap_or(false)
    }

    /// Query protocol semver version (major, minor, patch)
    pub fn get_contract_version(_env: Env) -> (u32, u32, u32) {
        (1, 0, 0)
    }

    /// Upgrade contract executable code
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        bump_instance_ttl(&env);

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        publish_upgrade(&env, new_wasm_hash);
        Ok(())
    }

    /// Query the protocol admin address
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Transfer contract administrative authority
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        new_admin.require_auth();
        bump_instance_ttl(&env);

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        publish_admin_transfer(&env, admin, new_admin);
        Ok(())
    }

    /// Create a new milestone escrow agreement
    pub fn create_agreement(
        env: Env,
        client: Address,
        provider: Address,
        arbiter: Address,
        token: Address,
        milestones: Vec<Milestone>,
        deadline: u64,
    ) -> Result<u64, Error> {
        if Self::is_paused(env.clone()) {
            return Err(Error::ProtocolPaused);
        }

        client.require_auth();
        bump_instance_ttl(&env);

        if milestones.is_empty() {
            return Err(Error::InvalidMilestone);
        }

        if milestones.len() > MAX_MILESTONES {
            return Err(Error::MilestoneLimitExceeded);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(Error::DeadlinePassed);
        }

        let mut total_amount: i128 = 0;
        for i in 0..milestones.len() {
            let m = milestones.get(i).unwrap();
            if m.amount <= 0 || m.description.len() > MAX_DESC_LENGTH {
                return Err(Error::InvalidMilestone);
            }
            total_amount = total_amount
                .checked_add(m.amount)
                .ok_or(Error::InvalidMilestone)?;
        }

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AgreementCounter)
            .unwrap_or(0u64);
        counter += 1;

        let agreement = Agreement {
            id: counter,
            client: client.clone(),
            provider: provider.clone(),
            arbiter: arbiter.clone(),
            token: token.clone(),
            total_amount,
            balance: 0,
            status: AgreementStatus::Created,
            milestones,
            deadline,
            created_at: env.ledger().timestamp(),
        };

        save_agreement(&env, &agreement);
        env.storage()
            .instance()
            .set(&DataKey::AgreementCounter, &counter);

        publish_create(&env, counter, client, provider, total_amount);

        Ok(counter)
    }

    /// Deposit funds into the agreement escrow
    pub fn fund_agreement(env: Env, agreement_id: u64, funder: Address) -> Result<(), Error> {
        if Self::is_paused(env.clone()) {
            return Err(Error::ProtocolPaused);
        }

        funder.require_auth();
        bump_instance_ttl(&env);

        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        if agreement.status != AgreementStatus::Created {
            return Err(Error::InvalidStatus);
        }

        if agreement.deadline <= env.ledger().timestamp() {
            return Err(Error::DeadlinePassed);
        }

        let token_client = token::Client::new(&env, &agreement.token);
        token_client.transfer(
            &funder,
            &env.current_contract_address(),
            &agreement.total_amount,
        );

        agreement.balance = agreement.total_amount;
        agreement.status = AgreementStatus::Funded;

        save_agreement(&env, &agreement);

        publish_fund(&env, agreement_id, funder, agreement.total_amount);

        Ok(())
    }

    /// Cancel an unfunded agreement
    pub fn cancel_agreement(env: Env, agreement_id: u64) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();

        if agreement.status != AgreementStatus::Created {
            return Err(Error::CannotCancelFunded);
        }

        agreement.status = AgreementStatus::Cancelled;
        save_agreement(&env, &agreement);

        publish_cancel(&env, agreement_id, agreement.client);

        Ok(())
    }

    /// Provider submits evidence of completion for a specific milestone with sequential ordering
    pub fn submit_milestone(
        env: Env,
        agreement_id: u64,
        milestone_idx: u32,
        proof_hash: String,
    ) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.provider.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if milestone_idx >= agreement.milestones.len() || proof_hash.len() > MAX_PROOF_LENGTH {
            return Err(Error::InvalidMilestone);
        }

        // Sequential submission rule: prior milestones must be either submitted or approved
        if milestone_idx > 0 {
            let prev_milestone = agreement.milestones.get(milestone_idx - 1).unwrap();
            if prev_milestone.status == MilestoneStatus::Pending {
                return Err(Error::MilestoneOutOfOrder);
            }
        }

        let mut milestone = agreement.milestones.get(milestone_idx).unwrap();
        if milestone.status != MilestoneStatus::Pending {
            return Err(Error::InvalidStatus);
        }

        milestone.status = MilestoneStatus::Submitted;
        milestone.proof_hash = proof_hash.clone();
        agreement.milestones.set(milestone_idx, milestone);

        save_agreement(&env, &agreement);

        publish_submit(&env, agreement_id, milestone_idx, proof_hash);

        Ok(())
    }

    /// Client approves and releases payment for a submitted milestone (Checks-Effects-Interactions + Protocol Fee)
    pub fn release_milestone(env: Env, agreement_id: u64, milestone_idx: u32) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if milestone_idx >= agreement.milestones.len() {
            return Err(Error::InvalidMilestone);
        }

        let mut milestone = agreement.milestones.get(milestone_idx).unwrap();
        if milestone.status != MilestoneStatus::Submitted
            && milestone.status != MilestoneStatus::Pending
        {
            return Err(Error::InvalidStatus);
        }

        if agreement.balance < milestone.amount {
            return Err(Error::InvalidMilestone);
        }

        let release_amount = milestone.amount;
        agreement.balance -= release_amount;
        milestone.status = MilestoneStatus::Approved;
        agreement.milestones.set(milestone_idx, milestone);

        // Check if all milestones are approved
        let mut all_approved = true;
        for i in 0..agreement.milestones.len() {
            if agreement.milestones.get(i).unwrap().status != MilestoneStatus::Approved {
                all_approved = false;
                break;
            }
        }

        if all_approved {
            agreement.status = AgreementStatus::Completed;
        }

        // State committed before external token interaction
        save_agreement(&env, &agreement);

        // Calculate protocol fee deduction if configured
        let fee_config: Option<ProtocolFeeConfig> =
            env.storage().instance().get(&DataKey::FeeConfig);
        let (fee_amount, provider_payout) = if let Some(ref cfg) = fee_config {
            let fee = (release_amount * cfg.fee_bps as i128) / 10000;
            (fee, release_amount - fee)
        } else {
            (0, release_amount)
        };

        let token_client = token::Client::new(&env, &agreement.token);

        if provider_payout > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &agreement.provider,
                &provider_payout,
            );
        }

        if fee_amount > 0 {
            if let Some(ref cfg) = fee_config {
                token_client.transfer(&env.current_contract_address(), &cfg.recipient, &fee_amount);
                publish_fee_deducted(
                    &env,
                    agreement_id,
                    milestone_idx,
                    cfg.recipient.clone(),
                    fee_amount,
                );
            }
        }

        publish_release(&env, agreement_id, milestone_idx, agreement.balance);

        Ok(())
    }

    /// Claim refund of remaining escrow balance if provider misses deadline without completing all milestones
    pub fn claim_timeout_refund(env: Env, agreement_id: u64) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if env.ledger().timestamp() <= agreement.deadline {
            return Err(Error::DeadlineNotPassed);
        }

        let refund_amount = agreement.balance;
        if refund_amount <= 0 {
            return Err(Error::InvalidStatus);
        }

        agreement.balance = 0;
        agreement.status = AgreementStatus::Cancelled;
        save_agreement(&env, &agreement);

        let token_client = token::Client::new(&env, &agreement.token);
        token_client.transfer(
            &env.current_contract_address(),
            &agreement.client,
            &refund_amount,
        );

        publish_refund(&env, agreement_id, agreement.client, refund_amount);

        Ok(())
    }

    /// Client or provider raises a dispute on a milestone
    pub fn raise_dispute(
        env: Env,
        agreement_id: u64,
        milestone_idx: u32,
        caller: Address,
    ) -> Result<(), Error> {
        caller.require_auth();
        bump_instance_ttl(&env);

        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        if caller != agreement.client && caller != agreement.provider {
            return Err(Error::Unauthorized);
        }

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if milestone_idx >= agreement.milestones.len() {
            return Err(Error::InvalidMilestone);
        }

        let mut milestone = agreement.milestones.get(milestone_idx).unwrap();
        milestone.status = MilestoneStatus::Disputed;
        agreement.milestones.set(milestone_idx, milestone);
        agreement.status = AgreementStatus::Disputed;

        save_agreement(&env, &agreement);

        publish_dispute(&env, agreement_id, milestone_idx, caller);

        Ok(())
    }

    /// Arbiter resolves dispute by dividing exact remaining balance
    pub fn resolve_dispute(
        env: Env,
        agreement_id: u64,
        client_amount: i128,
        provider_amount: i128,
    ) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.arbiter.require_auth();

        if agreement.status != AgreementStatus::Disputed {
            return Err(Error::DisputeNotActive);
        }

        let total_payout = client_amount
            .checked_add(provider_amount)
            .ok_or(Error::InvalidSplit)?;

        // Enforce exact distribution of remaining balance to prevent trapped tokens
        if total_payout != agreement.balance || client_amount < 0 || provider_amount < 0 {
            return Err(Error::InvalidSplit);
        }

        agreement.balance = 0;
        agreement.status = AgreementStatus::Resolved;
        save_agreement(&env, &agreement);

        let token_client = token::Client::new(&env, &agreement.token);

        if client_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &agreement.client,
                &client_amount,
            );
        }

        if provider_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &agreement.provider,
                &provider_amount,
            );
        }

        publish_resolve(&env, agreement_id, client_amount, provider_amount);

        Ok(())
    }

    /// Query specific milestone details
    pub fn get_milestone(
        env: Env,
        agreement_id: u64,
        milestone_idx: u32,
    ) -> Result<Milestone, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        agreement
            .milestones
            .get(milestone_idx)
            .ok_or(Error::InvalidMilestone)
    }

    /// Query total milestone count for an agreement
    pub fn get_milestones_count(env: Env, agreement_id: u64) -> Result<u32, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.milestones.len())
    }

    /// Query agreement details from persistent storage
    pub fn get_agreement(env: Env, agreement_id: u64) -> Result<Agreement, Error> {
        load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)
    }

    /// Query agreement summary without loading full milestone vector
    pub fn get_agreement_summary(env: Env, agreement_id: u64) -> Result<AgreementSummary, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(AgreementSummary {
            id: agreement.id,
            client: agreement.client,
            provider: agreement.provider,
            arbiter: agreement.arbiter,
            token: agreement.token,
            total_amount: agreement.total_amount,
            balance: agreement.balance,
            status: agreement.status,
            milestones_count: agreement.milestones.len(),
            deadline: agreement.deadline,
            created_at: agreement.created_at,
        })
    }

    /// Check if agreement exists
    pub fn has_agreement(env: Env, agreement_id: u64) -> bool {
        has_agreement(&env, agreement_id)
    }

    /// Get current total agreements count
    pub fn get_counter(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AgreementCounter)
            .unwrap_or(0u64)
    }
}
