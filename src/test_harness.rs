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
