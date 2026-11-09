#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env, String,
};

#[test]
fn test_create_fund_and_release_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    // Mint tokens to client
    token_admin_client.mint(&client_addr, &1000);

    // Initialize contract
    client.initialize(&admin);

    // Create milestones
    let m1 = Milestone {
        description: String::from_str(&env, "Architecture and design docs"),
        amount: 400,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let m2 = Milestone {
        description: String::from_str(&env, "Implementation and testing"),
        amount: 600,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };

    let milestones = vec![&env, m1, m2];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    assert_eq!(agreement_id, 1);
    assert!(client.has_agreement(&1));

    let count = client.get_milestones_count(&1);
    assert_eq!(count, 2);

    let m_query = client.get_milestone(&1, &0);
    assert_eq!(m_query.amount, 400);

    let summary = client.get_agreement_summary(&1);
    assert_eq!(summary.total_amount, 1000);
    assert_eq!(summary.milestones_count, 2);

    let agreement = client.get_agreement(&1);
    assert_eq!(agreement.total_amount, 1000);
    assert_eq!(agreement.balance, 0);
    assert_eq!(agreement.status, AgreementStatus::Created);

    // Fund agreement
    client.fund_agreement(&1, &client_addr);

    let funded_agreement = client.get_agreement(&1);
    assert_eq!(funded_agreement.balance, 1000);
    assert_eq!(funded_agreement.status, AgreementStatus::Funded);
    assert_eq!(token_client.balance(&contract_id), 1000);

    // Submit milestone 0
    let proof = String::from_str(
        &env,
        "ipfs://QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
    );
    client.submit_milestone(&1, &0, &proof);

    let submitted_agreement = client.get_agreement(&1);
    assert_eq!(
        submitted_agreement.milestones.get(0).unwrap().status,
        MilestoneStatus::Submitted
    );

    // Release milestone 0
    client.release_milestone(&1, &0);
    let released_agreement = client.get_agreement(&1);
    assert_eq!(
        released_agreement.milestones.get(0).unwrap().status,
        MilestoneStatus::Approved
    );
    assert_eq!(released_agreement.balance, 600);
    assert_eq!(token_client.balance(&provider_addr), 400);

    // Release milestone 1
    client.release_milestone(&1, &1);
    let completed_agreement = client.get_agreement(&1);
    assert_eq!(completed_agreement.status, AgreementStatus::Completed);
    assert_eq!(completed_agreement.balance, 0);
    assert_eq!(token_client.balance(&provider_addr), 1000);
}

#[test]
fn test_dispute_and_resolution_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Full delivery"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&agreement_id, &client_addr);

    // Raise dispute
    client.raise_dispute(&agreement_id, &0, &client_addr);
    let disputed_agreement = client.get_agreement(&agreement_id);
    assert_eq!(disputed_agreement.status, AgreementStatus::Disputed);

    // Arbiter splits exact balance 500/500
    client.resolve_dispute(&agreement_id, &500, &500);
    let resolved_agreement = client.get_agreement(&agreement_id);
    assert_eq!(resolved_agreement.status, AgreementStatus::Resolved);
    assert_eq!(resolved_agreement.balance, 0);
    assert_eq!(token_client.balance(&client_addr), 500);
    assert_eq!(token_client.balance(&provider_addr), 500);
}

#[test]
fn test_timeout_refund_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &2000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Milestone 1"),
        amount: 2000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 5000;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&id, &client_addr);
    assert_eq!(token_client.balance(&client_addr), 0);

    // Advance ledger timestamp beyond deadline
    env.ledger().with_mut(|l| {
        l.timestamp = deadline + 100;
    });

    // Client claims timeout refund
    client.claim_timeout_refund(&id);

    let refunded_agreement = client.get_agreement(&id);
    assert_eq!(refunded_agreement.status, AgreementStatus::Cancelled);
    assert_eq!(refunded_agreement.balance, 0);
    assert_eq!(token_client.balance(&client_addr), 2000);
}

#[test]
fn test_circuit_breaker_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    assert!(!client.is_paused());

    client.set_pause(&true);
    assert!(client.is_paused());

    client.set_pause(&false);
    assert!(!client.is_paused());
}

#[test]
fn test_fee_deduction_on_release() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    // Set 2.5% protocol fee (250 bps)
    client.set_fee_config(&250, &fee_recipient);

    let m1 = Milestone {
        description: String::from_str(&env, "Milestone with fee"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&id, &client_addr);
    client.release_milestone(&id, &0);

    // 2.5% of 1000 = 25 fee, 975 to provider
    assert_eq!(token_client.balance(&fee_recipient), 25);
    assert_eq!(token_client.balance(&provider_addr), 975);
}

#[test]
fn test_contract_version() {
    let env = Env::default();
    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let (major, minor, patch) = client.get_contract_version();
    assert_eq!(major, 1);
    assert_eq!(minor, 0);
    assert_eq!(patch, 0);
}

#[test]
fn test_cancellation_unfunded_and_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Draft Scope"),
        amount: 500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    let created = client.get_agreement(&id);
    assert_eq!(created.status, AgreementStatus::Created);

    // Cancel agreement before funding
    client.cancel_agreement(&id);

    let cancelled = client.get_agreement(&id);
    assert_eq!(cancelled.status, AgreementStatus::Cancelled);
}

#[test]
fn test_multi_milestone_sequential_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &3000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Phase 1: Spec"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let m2 = Milestone {
        description: String::from_str(&env, "Phase 2: Code"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let m3 = Milestone {
        description: String::from_str(&env, "Phase 3: Deploy"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };

    let milestones = vec![&env, m1, m2, m3];
    let deadline = env.ledger().timestamp() + 86400 * 7;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&id, &client_addr);
    assert_eq!(client.get_agreement(&id).balance, 3000);

    // Process all 3 milestones sequentially
    for idx in 0..3 {
        let proof = String::from_str(&env, "ipfs://proof-hash");
        client.submit_milestone(&id, &idx, &proof);
        client.release_milestone(&id, &idx);
    }

    let final_agreement = client.get_agreement(&id);
    assert_eq!(final_agreement.balance, 0);
    assert_eq!(final_agreement.status, AgreementStatus::Completed);
    assert_eq!(token_client.balance(&provider_addr), 3000);
}

#[test]
#[should_panic]
fn test_dispute_split_mismatch_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Scope"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&id, &client_addr);
    client.raise_dispute(&id, &0, &client_addr);

    // Attempt to split 500 + 400 = 900 which is less than 1000 balance -> should panic
    client.resolve_dispute(&id, &500, &400);
}

#[test]
#[should_panic]
fn test_timeout_refund_before_deadline_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Active Project"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400 * 5;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&id, &client_addr);

    // Call refund before deadline -> must panic with DeadlineNotPassed
    client.claim_timeout_refund(&id);
}

#[test]
#[should_panic]
fn test_emergency_pause_rejects_deposits() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Scope"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    // Emergency pause activated by admin
    client.set_pause(&true);

    // Attempting to fund paused agreement must panic with EmergencyPaused
    client.fund_agreement(&id, &client_addr);
}

#[test]
fn test_dispute_resolution_deducts_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    // Set 2.5% protocol fee (250 bps)
    client.set_fee_config(&250, &fee_recipient);

    let m1 = Milestone {
        description: String::from_str(&env, "Full delivery"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&agreement_id, &client_addr);
    client.raise_dispute(&agreement_id, &0, &client_addr);

    // Arbiter splits 400 to client, 600 to provider
    client.resolve_dispute(&agreement_id, &400, &600);

    // 2.5% fee on provider payout of 600 = 15 fee, 585 to provider
    assert_eq!(token_client.balance(&fee_recipient), 15);
    assert_eq!(token_client.balance(&provider_addr), 585);
    assert_eq!(token_client.balance(&client_addr), 400);
}

#[test]
fn test_two_step_admin_nomination_and_acceptance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
    assert!(client.is_admin(&admin));
    assert!(!client.is_admin(&new_admin));
    assert_eq!(client.get_pending_admin(), None);

    // Step 1: Nominate new admin
    client.nominate_admin(&new_admin);
    assert_eq!(client.get_pending_admin(), Some(new_admin.clone()));
    assert_eq!(client.get_admin(), admin);

    // Step 2: Accept nomination
    client.accept_admin();
    assert_eq!(client.get_admin(), new_admin);
    assert!(client.is_admin(&new_admin));
    assert!(!client.is_admin(&admin));
    assert_eq!(client.get_pending_admin(), None);
}

#[test]
fn test_milestone_rejection_and_resubmission() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "API Implementation"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&agreement_id, &client_addr);

    // Provider submits initial proof
    let initial_proof = String::from_str(&env, "ipfs://initial-hash");
    client.submit_milestone(&agreement_id, &0, &initial_proof);

    let m = client.get_milestone(&agreement_id, &0);
    assert_eq!(m.status, MilestoneStatus::Submitted);

    // Client rejects with feedback requesting revision
    let feedback = String::from_str(&env, "ipfs://feedback-tests-needed");
    client.reject_milestone(&agreement_id, &0, &feedback);

    let rejected_m = client.get_milestone(&agreement_id, &0);
    assert_eq!(rejected_m.status, MilestoneStatus::Pending);

    // Provider resubmits with revised proof
    let revised_proof = String::from_str(&env, "ipfs://revised-hash-with-tests");
    client.submit_milestone(&agreement_id, &0, &revised_proof);

    let resubmitted_m = client.get_milestone(&agreement_id, &0);
    assert_eq!(resubmitted_m.status, MilestoneStatus::Submitted);

    // Client releases milestone
    client.release_milestone(&agreement_id, &0);
    let approved_m = client.get_milestone(&agreement_id, &0);
    assert_eq!(approved_m.status, MilestoneStatus::Approved);
}

#[test]
fn test_mutual_cancellation_and_escrow_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1500);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Phase 1"),
        amount: 1500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&agreement_id, &client_addr);
    assert_eq!(token_client.balance(&client_addr), 0);

    // Both parties agree to cancel
    client.cancel_agreement_mutual(&agreement_id);

    let agreement = client.get_agreement(&agreement_id);
    assert_eq!(agreement.status, AgreementStatus::Cancelled);
    assert_eq!(agreement.balance, 0);
    assert_eq!(token_client.balance(&client_addr), 1500);
}

#[test]
fn test_arbiter_reassignment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let initial_arbiter = Address::generate(&env);
    let new_arbiter = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Scope"),
        amount: 500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &initial_arbiter,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    let ag_before = client.get_agreement(&agreement_id);
    assert_eq!(ag_before.arbiter, initial_arbiter);

    // Reassign arbiter with mutual consent
    client.reassign_arbiter(&agreement_id, &new_arbiter);

    let ag_after = client.get_agreement(&agreement_id);
    assert_eq!(ag_after.arbiter, new_arbiter);
}

#[test]
fn test_dispute_timeout_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = TokenClient::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &2000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Full contract"),
        amount: 2000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + (60 * 86400);

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&agreement_id, &client_addr);
    client.raise_dispute(&agreement_id, &0, &client_addr);

    let dispute_ts = client.get_dispute_timestamp(&agreement_id);
    assert!(dispute_ts.is_some());

    // Advance ledger past DEFAULT_DISPUTE_WINDOW (30 days = 2,592,000s)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + (31 * 86400));

    client.resolve_dispute_timeout(&agreement_id);

    let ag = client.get_agreement(&agreement_id);
    assert_eq!(ag.status, AgreementStatus::Cancelled);
    assert_eq!(ag.balance, 0);
    assert_eq!(token_client.balance(&client_addr), 2000);
}

#[test]
fn test_ttl_bumping_and_milestones_getter() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Part 1"),
        amount: 300,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let m2 = Milestone {
        description: String::from_str(&env, "Part 2"),
        amount: 700,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1, m2];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    // Test bump_agreement_ttl
    client.bump_agreement_ttl(&agreement_id);

    // Test get_agreement_milestones
    let fetched_milestones = client.get_agreement_milestones(&agreement_id);
    assert_eq!(fetched_milestones.len(), 2);
    assert_eq!(fetched_milestones.get(0).unwrap().amount, 300);
    assert_eq!(fetched_milestones.get(1).unwrap().amount, 700);
}

#[test]
#[should_panic(expected = "Error(Contract, #22)")]
fn test_duration_exceeded_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Far future milestone"),
        amount: 500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    // Exceed MAX_AGREEMENT_DURATION (365 days)
    let deadline = env.ledger().timestamp() + (400 * 86400);

    client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );
}

#[test]
fn test_queries_and_role_predicates() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);
    let stranger = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    client.initialize(&admin);

    // Test ContractMetadata
    let metadata = client.get_contract_metadata();
    assert_eq!(metadata.version_major, 1);
    assert_eq!(metadata.version_minor, 0);

    let m1 = Milestone {
        description: String::from_str(&env, "Scope 1"),
        amount: 500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let m2 = Milestone {
        description: String::from_str(&env, "Scope 2"),
        amount: 500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1, m2];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    // Role predicates
    assert!(client.is_agreement_party(&agreement_id, &client_addr));
    assert!(client.is_agreement_party(&agreement_id, &provider_addr));
    assert!(client.is_agreement_party(&agreement_id, &arbiter_addr));
    assert!(!client.is_agreement_party(&agreement_id, &stranger));

    assert!(client.is_agreement_client(&agreement_id, &client_addr));
    assert!(!client.is_agreement_client(&agreement_id, &provider_addr));
    assert!(client.is_agreement_provider(&agreement_id, &provider_addr));
    assert!(client.is_agreement_arbiter(&agreement_id, &arbiter_addr));

    // Expiration check
    assert!(!client.is_agreement_expired(&agreement_id));

    // Batch summary
    let ids = vec![&env, agreement_id];
    let summaries = client.get_agreements_summary(&ids);
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries.get(0).unwrap().id, agreement_id);

    // Status breakdown
    let breakdown = client.get_milestones_status_breakdown(&agreement_id);
    assert_eq!(breakdown.pending, 2);
    assert_eq!(breakdown.submitted, 0);
    assert_eq!(breakdown.approved, 0);
    assert_eq!(breakdown.disputed, 0);
}

#[test]
fn test_rejection_count_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&client_addr, &1000);
    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Deliverable"),
        amount: 1000,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    let deadline = env.ledger().timestamp() + 86400;

    let agreement_id = client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );

    client.fund_agreement(&agreement_id, &client_addr);
    assert_eq!(client.get_milestone_rejection_count(&agreement_id, &0), 0);

    // Submit and reject 1st time
    client.submit_milestone(&agreement_id, &0, &String::from_str(&env, "ipfs://proof1"));
    client.reject_milestone(
        &agreement_id,
        &0,
        &String::from_str(&env, "ipfs://feedback1"),
    );
    assert_eq!(client.get_milestone_rejection_count(&agreement_id, &0), 1);

    // Submit and reject 2nd time
    client.submit_milestone(&agreement_id, &0, &String::from_str(&env, "ipfs://proof2"));
    client.reject_milestone(
        &agreement_id,
        &0,
        &String::from_str(&env, "ipfs://feedback2"),
    );
    assert_eq!(client.get_milestone_rejection_count(&agreement_id, &0), 2);
}

#[test]
fn test_fee_recipient_and_bps_updates() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);

    client.initialize(&admin);
    client.set_fee_config(&100, &recipient1);

    let cfg = client.get_fee_config().unwrap();
    assert_eq!(cfg.fee_bps, 100);
    assert_eq!(cfg.recipient, recipient1);

    // Update recipient only
    client.update_fee_recipient(&recipient2);
    let cfg2 = client.get_fee_config().unwrap();
    assert_eq!(cfg2.fee_bps, 100);
    assert_eq!(cfg2.recipient, recipient2);

    // Update bps only
    client.update_fee_bps(&250);
    let cfg3 = client.get_fee_config().unwrap();
    assert_eq!(cfg3.fee_bps, 250);
    assert_eq!(cfg3.recipient, recipient2);
}

#[test]
#[should_panic(expected = "Error(Contract, #24)")]
fn test_duration_too_short_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StrataFlowContract);
    let client = StrataFlowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let provider_addr = Address::generate(&env);
    let arbiter_addr = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    client.initialize(&admin);

    let m1 = Milestone {
        description: String::from_str(&env, "Quick milestone"),
        amount: 500,
        status: MilestoneStatus::Pending,
        proof_hash: String::from_str(&env, ""),
    };
    let milestones = vec![&env, m1];
    // Under MIN_AGREEMENT_DURATION (3600 seconds)
    let deadline = env.ledger().timestamp() + 1800;

    client.create_agreement(
        &client_addr,
        &provider_addr,
        &arbiter_addr,
        &token_contract.address(),
        &milestones,
        &deadline,
    );
}
