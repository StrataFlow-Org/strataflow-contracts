use soroban_sdk::{symbol_short, Address, BytesN, Env, String};

pub fn publish_create(
    env: &Env,
    agreement_id: u64,
    client: Address,
    provider: Address,
    total_amount: i128,
) {
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
    env.events()
        .publish((symbol_short!("refund"), agreement_id), (client, amount));
}

pub fn publish_cancel(env: &Env, agreement_id: u64, client: Address) {
    env.events()
        .publish((symbol_short!("cancel"), agreement_id), client);
}

pub fn publish_pause(env: &Env, paused: bool) {
    env.events().publish((symbol_short!("pause"),), paused);
}

pub fn publish_admin_transfer(env: &Env, old_admin: Address, new_admin: Address) {
    env.events()
        .publish((symbol_short!("admin"),), (old_admin, new_admin));
}

pub fn publish_admin_nomination(env: &Env, current_admin: Address, pending_admin: Address) {
    env.events()
        .publish((symbol_short!("nom_adm"),), (current_admin, pending_admin));
}

pub fn publish_reject(env: &Env, agreement_id: u64, milestone_idx: u32, feedback_hash: String) {
    env.events().publish(
        (symbol_short!("reject"), agreement_id, milestone_idx),
        feedback_hash,
    );
}

pub fn publish_mutual_cancel(env: &Env, agreement_id: u64, client: Address, refund_amount: i128) {
    env.events().publish(
        (symbol_short!("mut_can"), agreement_id),
        (client, refund_amount),
    );
}

pub fn publish_reassign_arbiter(
    env: &Env,
    agreement_id: u64,
    old_arbiter: Address,
    new_arbiter: Address,
) {
    env.events().publish(
        (symbol_short!("arb_chg"), agreement_id),
        (old_arbiter, new_arbiter),
    );
}

pub fn publish_dispute_timeout(env: &Env, agreement_id: u64, refund_amount: i128) {
    env.events()
        .publish((symbol_short!("disp_to"), agreement_id), refund_amount);
}

pub fn publish_fee_recipient_update(env: &Env, old_recipient: Address, new_recipient: Address) {
    env.events()
        .publish((symbol_short!("fee_rec"),), (old_recipient, new_recipient));
}

pub fn publish_fee_bps_update(env: &Env, old_bps: u32, new_bps: u32) {
    env.events()
        .publish((symbol_short!("fee_bps"),), (old_bps, new_bps));
}

pub fn publish_fee_config(env: &Env, fee_bps: u32, recipient: Address) {
    env.events()
        .publish((symbol_short!("fee_cfg"),), (fee_bps, recipient));
}

pub fn publish_fee_deducted(
    env: &Env,
    agreement_id: u64,
    milestone_idx: u32,
    recipient: Address,
    fee_amount: i128,
) {
    env.events().publish(
        (symbol_short!("fee_paid"), agreement_id, milestone_idx),
        (recipient, fee_amount),
    );
}

pub fn publish_upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
    env.events()
        .publish((symbol_short!("upgrade"),), new_wasm_hash);
}

pub fn publish_dispute_verdict(
    env: &Env,
    agreement_id: u64,
    arbiter: Address,
    verdict_hash: String,
) {
    env.events().publish(
        (symbol_short!("verdict"), agreement_id),
        (arbiter, verdict_hash),
    );
}

pub fn publish_protocol_init(env: &Env, admin: Address) {
    env.events().publish((symbol_short!("init"),), admin);
}
