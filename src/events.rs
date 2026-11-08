use soroban_sdk::{symbol_short, Address, Env, String};

pub fn publish_create(env: &Env, agreement_id: u64, client: Address, provider: Address, total_amount: i128) {
    env.events().publish(
        (symbol_short!("create"), agreement_id),
        (client, provider, total_amount),
    );
}

pub fn publish_fund(env: &Env, agreement_id: u64, funder: Address, total_amount: i128) {
    env.events().publish(
        (symbol_short!("fund"), agreement_id),
        (funder, total_amount),
    );
}

pub fn publish_submit(env: &Env, agreement_id: u64, milestone_idx: u32, proof_hash: String) {
    env.events().publish(
        (symbol_short!("submit"), agreement_id, milestone_idx),
        proof_hash,
    );
}

pub fn publish_release(env: &Env, agreement_id: u64, milestone_idx: u32, remaining_balance: i128) {
    env.events().publish(
        (symbol_short!("release"), agreement_id, milestone_idx),
        remaining_balance,
    );
}

pub fn publish_dispute(env: &Env, agreement_id: u64, milestone_idx: u32, caller: Address) {
    env.events().publish(
        (symbol_short!("dispute"), agreement_id, milestone_idx),
        caller,
    );
}

pub fn publish_resolve(env: &Env, agreement_id: u64, client_amount: i128, provider_amount: i128) {
    env.events().publish(
        (symbol_short!("resolve"), agreement_id),
        (client_amount, provider_amount),
    );
}

pub fn publish_refund(env: &Env, agreement_id: u64, client: Address, amount: i128) {
    env.events().publish(
        (symbol_short!("refund"), agreement_id),
        (client, amount),
    );
}

pub fn publish_cancel(env: &Env, agreement_id: u64, client: Address) {
    env.events().publish(
        (symbol_short!("cancel"), agreement_id),
        client,
    );
}

pub fn publish_pause(env: &Env, paused: bool) {
    env.events().publish(
        (symbol_short!("pause"),),
        paused,
    );
}

pub fn publish_admin_transfer(env: &Env, old_admin: Address, new_admin: Address) {
    env.events().publish(
        (symbol_short!("admin"),),
        (old_admin, new_admin),
    );
}
