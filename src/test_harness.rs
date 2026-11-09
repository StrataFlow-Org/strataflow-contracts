#![cfg(test)]
use crate::contract::{StrataFlowContract, StrataFlowContractClient};
use crate::types::MilestoneInput;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

pub struct TestFixture<'a> {
    pub env: Env,
    pub contract_id: Address,
    pub client: StrataFlowContractClient<'a>,
    pub admin: Address,
    pub fee_recipient: Address,
}

impl<'a> TestFixture<'a> {
    pub fn new(env: &'a Env) -> Self {
        let contract_id = env.register_contract(None, StrataFlowContract);
        let client = StrataFlowContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let fee_recipient = Address::generate(env);

        client.init(&admin, &fee_recipient, &100);

        TestFixture {
            env: env.clone(),
            contract_id,
            client,
            admin,
            fee_recipient,
        }
    }
}

pub fn create_mock_token<'a>(
    env: &'a Env,
    admin: &Address,
) -> (
    Address,
    soroban_sdk::token::Client<'a>,
    soroban_sdk::token::StellarAssetClient<'a>,
) {
    let token_address = env.register_stellar_asset_contract(admin.clone());
    let token_client = soroban_sdk::token::Client::new(env, &token_address);
    let token_admin = soroban_sdk::token::StellarAssetClient::new(env, &token_address);
    (token_address, token_client, token_admin)
}

pub struct AgreementFixtureBuilder<'a> {
    pub fixture: &'a TestFixture<'a>,
    pub client_addr: Address,
    pub provider_addr: Address,
    pub arbiter_addr: Address,
    pub token_addr: Address,
    pub duration_seconds: u64,
}

impl<'a> AgreementFixtureBuilder<'a> {
    pub fn new(fixture: &'a TestFixture<'a>, token_addr: Address) -> Self {
        let env = &fixture.env;
        Self {
            fixture,
            client_addr: Address::generate(env),
            provider_addr: Address::generate(env),
            arbiter_addr: Address::generate(env),
            token_addr,
            duration_seconds: 7200,
        }
    }

    pub fn with_duration(mut self, duration: u64) -> Self {
        self.duration_seconds = duration;
        self
    }
}
