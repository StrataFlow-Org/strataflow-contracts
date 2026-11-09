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

    /// Update protocol fee recipient without changing basis points
    pub fn update_fee_recipient(env: Env, new_recipient: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        bump_instance_ttl(&env);

        let mut config: ProtocolFeeConfig = env
            .storage()
            .instance()
            .get(&DataKey::FeeConfig)
            .ok_or(Error::NotInitialized)?;

        let old_recipient = config.recipient.clone();
        config.recipient = new_recipient.clone();
        env.storage().instance().set(&DataKey::FeeConfig, &config);
        publish_fee_recipient_update(&env, old_recipient, new_recipient);
        Ok(())
    }

    /// Update protocol fee basis points without changing recipient
    pub fn update_fee_bps(env: Env, new_bps: u32) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        bump_instance_ttl(&env);

        if new_bps > MAX_FEE_BPS {
            return Err(Error::InvalidFeeBps);
        }

        let mut config: ProtocolFeeConfig = env
            .storage()
            .instance()
            .get(&DataKey::FeeConfig)
            .ok_or(Error::NotInitialized)?;

        let old_bps = config.fee_bps;
        config.fee_bps = new_bps;
        env.storage().instance().set(&DataKey::FeeConfig, &config);
        publish_fee_bps_update(&env, old_bps, new_bps);
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

    /// Query comprehensive contract metadata
    pub fn get_contract_metadata(env: Env) -> ContractMetadata {
        ContractMetadata {
            name: String::from_str(&env, "StrataFlow Protocol"),
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
        }
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

    /// Nominate a new admin address (step 1 of 2-step transfer)
    pub fn nominate_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        bump_instance_ttl(&env);

        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        publish_admin_nomination(&env, admin, new_admin);
        Ok(())
    }

    /// Accept pending admin nomination (step 2 of 2-step transfer)
    pub fn accept_admin(env: Env) -> Result<(), Error> {
        let pending_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(Error::NoPendingAdmin)?;
        pending_admin.require_auth();
        bump_instance_ttl(&env);

        let old_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        env.storage()
            .instance()
            .set(&DataKey::Admin, &pending_admin);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        publish_admin_transfer(&env, old_admin, pending_admin);
        Ok(())
    }

    /// Query pending admin address if nomination is active
    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Verify whether an address is the current admin
    pub fn is_admin(env: Env, address: Address) -> bool {
        env.storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .map(|a| a == address)
            .unwrap_or(false)
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

        if client == provider || client == arbiter || provider == arbiter {
            return Err(Error::Unauthorized);
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

        let duration = deadline.saturating_sub(env.ledger().timestamp());
        if duration < MIN_AGREEMENT_DURATION {
            return Err(Error::AgreementDurationTooShort);
        }
        if duration > MAX_AGREEMENT_DURATION {
            return Err(Error::DurationExceeded);
        }

        let mut total_amount: i128 = 0;
        for i in 0..milestones.len() {
            let m = milestones.get(i).unwrap();
            if m.amount < MIN_MILESTONE_AMOUNT
                || m.description.is_empty()
                || m.description.len() > MAX_DESC_LENGTH
            {
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

        let mut metrics = load_protocol_metrics(&env);
        metrics.total_escrow_volume = metrics
            .total_escrow_volume
            .saturating_add(agreement.total_amount);
        save_protocol_metrics(&env, &metrics);

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
            let mut metrics = load_protocol_metrics(&env);
            metrics.total_agreements_completed =
                metrics.total_agreements_completed.saturating_add(1);
            save_protocol_metrics(&env, &metrics);
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

    /// Client requests revisions by rejecting a submitted milestone with feedback hash
    pub fn reject_milestone(
        env: Env,
        agreement_id: u64,
        milestone_idx: u32,
        feedback_hash: String,
    ) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if milestone_idx >= agreement.milestones.len() || feedback_hash.len() > MAX_PROOF_LENGTH {
            return Err(Error::InvalidMilestone);
        }

        let mut milestone = agreement.milestones.get(milestone_idx).unwrap();
        if milestone.status != MilestoneStatus::Submitted {
            return Err(Error::MilestoneNotSubmitted);
        }

        milestone.status = MilestoneStatus::Pending;
        milestone.proof_hash = String::from_str(&env, "");
        agreement.milestones.set(milestone_idx, milestone);

        save_agreement(&env, &agreement);
        increment_milestone_rejections(&env, agreement_id, milestone_idx);

        publish_reject(&env, agreement_id, milestone_idx, feedback_hash);

        Ok(())
    }

    /// Mutual cancellation: both client and provider agree to cancel funded agreement and refund remaining balance
    pub fn cancel_agreement_mutual(env: Env, agreement_id: u64) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();
        agreement.provider.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        let refund_amount = agreement.balance;
        agreement.balance = 0;
        agreement.status = AgreementStatus::Cancelled;
        save_agreement(&env, &agreement);

        if refund_amount > 0 {
            let token_client = token::Client::new(&env, &agreement.token);
            token_client.transfer(
                &env.current_contract_address(),
                &agreement.client,
                &refund_amount,
            );
        }

        publish_mutual_cancel(&env, agreement_id, agreement.client, refund_amount);

        Ok(())
    }

    /// Mutual arbiter reassignment: both client and provider agree to change dispute arbiter
    pub fn reassign_arbiter(
        env: Env,
        agreement_id: u64,
        new_arbiter: Address,
    ) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();
        agreement.provider.require_auth();

        if new_arbiter == agreement.arbiter {
            return Err(Error::SameArbiter);
        }

        if new_arbiter == agreement.client || new_arbiter == agreement.provider {
            return Err(Error::Unauthorized);
        }

        let old_arbiter = agreement.arbiter.clone();
        agreement.arbiter = new_arbiter.clone();
        save_agreement(&env, &agreement);

        publish_reassign_arbiter(&env, agreement_id, old_arbiter, new_arbiter);

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
        set_dispute_timestamp(&env, agreement_id, env.ledger().timestamp());

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
        remove_dispute_timestamp(&env, agreement_id);

        let fee_config: Option<ProtocolFeeConfig> =
            env.storage().instance().get(&DataKey::FeeConfig);
        let (fee_amount, provider_payout) = if let Some(ref cfg) = fee_config {
            let fee = (provider_amount * cfg.fee_bps as i128) / 10000;
            (fee, provider_amount - fee)
        } else {
            (0, provider_amount)
        };

        let token_client = token::Client::new(&env, &agreement.token);

        if client_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &agreement.client,
                &client_amount,
            );
        }

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
                publish_fee_deducted(&env, agreement_id, 0, cfg.recipient.clone(), fee_amount);
            }
        }

        publish_resolve(&env, agreement_id, client_amount, provider_amount);

        Ok(())
    }

    /// Query timestamp when agreement dispute was initiated
    pub fn get_dispute_timestamp(env: Env, agreement_id: u64) -> Option<u64> {
        load_dispute_timestamp(&env, agreement_id)
    }

    /// Claim refund if dispute has been active beyond dispute window without arbiter resolution
    pub fn resolve_dispute_timeout(env: Env, agreement_id: u64) -> Result<(), Error> {
        bump_instance_ttl(&env);
        let mut agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();

        if agreement.status != AgreementStatus::Disputed {
            return Err(Error::DisputeNotActive);
        }

        let dispute_ts =
            load_dispute_timestamp(&env, agreement_id).ok_or(Error::DisputeNotActive)?;

        if env.ledger().timestamp() < dispute_ts.saturating_add(DEFAULT_DISPUTE_WINDOW) {
            return Err(Error::DisputeTimeoutNotPassed);
        }

        let refund_amount = agreement.balance;
        agreement.balance = 0;
        agreement.status = AgreementStatus::Cancelled;
        save_agreement(&env, &agreement);
        remove_dispute_timestamp(&env, agreement_id);

        if refund_amount > 0 {
            let token_client = token::Client::new(&env, &agreement.token);
            token_client.transfer(
                &env.current_contract_address(),
                &agreement.client,
                &refund_amount,
            );
        }

        publish_dispute_timeout(&env, agreement_id, refund_amount);

        Ok(())
    }

    /// Query count breakdown of milestones by status
    pub fn get_milestones_status_breakdown(
        env: Env,
        agreement_id: u64,
    ) -> Result<MilestonesStatusBreakdown, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        let mut pending = 0u32;
        let mut submitted = 0u32;
        let mut approved = 0u32;
        let mut disputed = 0u32;

        for i in 0..agreement.milestones.len() {
            let m = agreement.milestones.get(i).unwrap();
            match m.status {
                MilestoneStatus::Pending => pending += 1,
                MilestoneStatus::Submitted => submitted += 1,
                MilestoneStatus::Approved => approved += 1,
                MilestoneStatus::Disputed => disputed += 1,
            }
        }

        Ok(MilestonesStatusBreakdown {
            pending,
            submitted,
            approved,
            disputed,
        })
    }

    /// Batch query agreement summaries to reduce RPC roundtrips
    pub fn get_agreements_summary(env: Env, agreement_ids: Vec<u64>) -> Vec<AgreementSummary> {
        let mut summaries = Vec::new(&env);
        for i in 0..agreement_ids.len() {
            let id = agreement_ids.get(i).unwrap();
            if let Some(agreement) = load_agreement(&env, id) {
                summaries.push_back(AgreementSummary {
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
                });
            }
        }
        summaries
    }

    /// Query remaining escrow balance for an agreement
    pub fn get_remaining_escrow(env: Env, agreement_id: u64) -> Result<i128, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.balance)
    }

    /// Check if an address is any participant (client, provider, or arbiter) of an agreement
    pub fn is_agreement_party(
        env: Env,
        agreement_id: u64,
        address: Address,
    ) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.client == address
            || agreement.provider == address
            || agreement.arbiter == address)
    }

    /// Check if address is client of the agreement
    pub fn is_agreement_client(
        env: Env,
        agreement_id: u64,
        address: Address,
    ) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.client == address)
    }

    /// Check if address is provider of the agreement
    pub fn is_agreement_provider(
        env: Env,
        agreement_id: u64,
        address: Address,
    ) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.provider == address)
    }

    /// Check if address is arbiter of the agreement
    pub fn is_agreement_arbiter(
        env: Env,
        agreement_id: u64,
        address: Address,
    ) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.arbiter == address)
    }

    /// Check if agreement is actively in progress (Funded or Disputed)
    pub fn is_agreement_active(env: Env, agreement_id: u64) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.status == AgreementStatus::Funded
            || agreement.status == AgreementStatus::Disputed)
    }

    /// Check if agreement is completed
    pub fn is_agreement_completed(env: Env, agreement_id: u64) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.status == AgreementStatus::Completed)
    }

    /// Check whether agreement deadline has passed relative to current ledger time
    pub fn is_agreement_expired(env: Env, agreement_id: u64) -> Result<bool, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(env.ledger().timestamp() >= agreement.deadline)
    }

    /// Query remaining seconds before dispute resolution timeout expires
    pub fn get_dispute_window_remaining(env: Env, agreement_id: u64) -> Result<u64, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        if agreement.status != AgreementStatus::Disputed {
            return Err(Error::DisputeNotActive);
        }
        let dispute_ts =
            load_dispute_timestamp(&env, agreement_id).ok_or(Error::DisputeNotActive)?;
        let expire_time = dispute_ts.saturating_add(DEFAULT_DISPUTE_WINDOW);
        let now = env.ledger().timestamp();
        if now >= expire_time {
            Ok(0)
        } else {
            Ok(expire_time - now)
        }
    }

    /// Query number of times a milestone has been rejected for revision
    pub fn get_milestone_rejection_count(env: Env, agreement_id: u64, milestone_idx: u32) -> u32 {
        get_milestone_rejections(&env, agreement_id, milestone_idx)
    }

    /// Refresh persistent storage TTL for an agreement
    pub fn bump_agreement_ttl(env: Env, agreement_id: u64) -> Result<(), Error> {
        if !has_agreement(&env, agreement_id) {
            return Err(Error::AgreementNotFound);
        }
        bump_persistent_ttl(&env, &DataKey::Agreement(agreement_id));
        Ok(())
    }

    /// Query all milestones for an agreement directly
    pub fn get_agreement_milestones(env: Env, agreement_id: u64) -> Result<Vec<Milestone>, Error> {
        let agreement = load_agreement(&env, agreement_id).ok_or(Error::AgreementNotFound)?;
        Ok(agreement.milestones)
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
