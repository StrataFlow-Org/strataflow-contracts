use crate::errors::Error;
use crate::events::*;
use crate::types::*;
use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};

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
        env.storage().instance().set(&DataKey::AgreementCounter, &0u64);
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
        client.require_auth();

        if milestones.is_empty() {
            return Err(Error::InvalidMilestone);
        }

        let mut total_amount: i128 = 0;
        for i in 0..milestones.len() {
            let m = milestones.get(i).unwrap();
            if m.amount <= 0 {
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

        env.storage().instance().set(&DataKey::Agreement(counter), &agreement);
        env.storage().instance().set(&DataKey::AgreementCounter, &counter);

        publish_create(&env, counter, client, provider, total_amount);

        Ok(counter)
    }

    /// Deposit funds into the agreement escrow
    pub fn fund_agreement(env: Env, agreement_id: u64, funder: Address) -> Result<(), Error> {
        funder.require_auth();

        let mut agreement: Agreement = env
            .storage()
            .instance()
            .get(&DataKey::Agreement(agreement_id))
            .ok_or(Error::AgreementNotFound)?;

        if agreement.status != AgreementStatus::Created {
            return Err(Error::InvalidStatus);
        }

        let token_client = token::Client::new(&env, &agreement.token);
        token_client.transfer(&funder, &env.current_contract_address(), &agreement.total_amount);

        agreement.balance = agreement.total_amount;
        agreement.status = AgreementStatus::Funded;

        env.storage().instance().set(&DataKey::Agreement(agreement_id), &agreement);

        publish_fund(&env, agreement_id, funder, agreement.total_amount);

        Ok(())
    }

    /// Provider submits evidence of completion for a specific milestone
    pub fn submit_milestone(
        env: Env,
        agreement_id: u64,
        milestone_idx: u32,
        proof_hash: String,
    ) -> Result<(), Error> {
        let mut agreement: Agreement = env
            .storage()
            .instance()
            .get(&DataKey::Agreement(agreement_id))
            .ok_or(Error::AgreementNotFound)?;

        agreement.provider.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if milestone_idx >= agreement.milestones.len() {
            return Err(Error::InvalidMilestone);
        }

        let mut milestone = agreement.milestones.get(milestone_idx).unwrap();
        if milestone.status != MilestoneStatus::Pending {
            return Err(Error::InvalidStatus);
        }

        milestone.status = MilestoneStatus::Submitted;
        milestone.proof_hash = proof_hash.clone();
        agreement.milestones.set(milestone_idx, milestone);

        env.storage().instance().set(&DataKey::Agreement(agreement_id), &agreement);

        publish_submit(&env, agreement_id, milestone_idx, proof_hash);

        Ok(())
    }

    /// Client approves and releases payment for a submitted milestone
    pub fn release_milestone(
        env: Env,
        agreement_id: u64,
        milestone_idx: u32,
    ) -> Result<(), Error> {
        let mut agreement: Agreement = env
            .storage()
            .instance()
            .get(&DataKey::Agreement(agreement_id))
            .ok_or(Error::AgreementNotFound)?;

        agreement.client.require_auth();

        if agreement.status != AgreementStatus::Funded {
            return Err(Error::InvalidStatus);
        }

        if milestone_idx >= agreement.milestones.len() {
            return Err(Error::InvalidMilestone);
        }

        let mut milestone = agreement.milestones.get(milestone_idx).unwrap();
        if milestone.status != MilestoneStatus::Submitted && milestone.status != MilestoneStatus::Pending {
            return Err(Error::InvalidStatus);
        }

        if agreement.balance < milestone.amount {
            return Err(Error::InvalidMilestone);
        }

        let token_client = token::Client::new(&env, &agreement.token);
        token_client.transfer(
            &env.current_contract_address(),
            &agreement.provider,
            &milestone.amount,
        );

        agreement.balance -= milestone.amount;
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

        env.storage().instance().set(&DataKey::Agreement(agreement_id), &agreement);

        publish_release(&env, agreement_id, milestone_idx, agreement.balance);

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

        let mut agreement: Agreement = env
            .storage()
            .instance()
            .get(&DataKey::Agreement(agreement_id))
            .ok_or(Error::AgreementNotFound)?;

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

        env.storage().instance().set(&DataKey::Agreement(agreement_id), &agreement);

        publish_dispute(&env, agreement_id, milestone_idx, caller);

        Ok(())
    }

    /// Arbiter resolves dispute by dividing remaining balance
    pub fn resolve_dispute(
        env: Env,
        agreement_id: u64,
        client_amount: i128,
        provider_amount: i128,
    ) -> Result<(), Error> {
        let mut agreement: Agreement = env
            .storage()
            .instance()
            .get(&DataKey::Agreement(agreement_id))
            .ok_or(Error::AgreementNotFound)?;

        agreement.arbiter.require_auth();

        if agreement.status != AgreementStatus::Disputed {
            return Err(Error::DisputeNotActive);
        }

        let total_payout = client_amount
            .checked_add(provider_amount)
            .ok_or(Error::InvalidSplit)?;

        if total_payout > agreement.balance || client_amount < 0 || provider_amount < 0 {
            return Err(Error::InvalidSplit);
        }

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

        agreement.balance -= total_payout;
        agreement.status = AgreementStatus::Resolved;

        env.storage().instance().set(&DataKey::Agreement(agreement_id), &agreement);

        publish_resolve(&env, agreement_id, client_amount, provider_amount);

        Ok(())
    }

    /// Query agreement details
    pub fn get_agreement(env: Env, agreement_id: u64) -> Result<Agreement, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Agreement(agreement_id))
            .ok_or(Error::AgreementNotFound)
    }

    /// Get current total agreements count
    pub fn get_counter(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AgreementCounter)
            .unwrap_or(0u64)
    }
}
