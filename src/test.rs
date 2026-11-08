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
    let proof = String::from_str(&env, "ipfs://QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco");
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
    let deadline = env.ledger().timestamp() + 1000;

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


