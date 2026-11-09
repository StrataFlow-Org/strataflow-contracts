use crate::types::*;
use soroban_sdk::Env;

pub fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn bump_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(
        key,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
}

pub fn save_agreement(env: &Env, agreement: &Agreement) {
    let key = DataKey::Agreement(agreement.id);
    env.storage().persistent().set(&key, agreement);
    bump_persistent_ttl(env, &key);
}

pub fn load_agreement(env: &Env, id: u64) -> Option<Agreement> {
    let key = DataKey::Agreement(id);
    let agreement: Option<Agreement> = env.storage().persistent().get(&key);
    if agreement.is_some() {
        bump_persistent_ttl(env, &key);
    }
    agreement
}

pub fn has_agreement(env: &Env, id: u64) -> bool {
    let key = DataKey::Agreement(id);
    env.storage().persistent().has(&key)
}

pub fn set_dispute_timestamp(env: &Env, agreement_id: u64, timestamp: u64) {
    let key = DataKey::DisputeTimestamp(agreement_id);
    env.storage().persistent().set(&key, &timestamp);
    bump_persistent_ttl(env, &key);
}

pub fn load_dispute_timestamp(env: &Env, agreement_id: u64) -> Option<u64> {
    let key = DataKey::DisputeTimestamp(agreement_id);
    let ts: Option<u64> = env.storage().persistent().get(&key);
    if ts.is_some() {
        bump_persistent_ttl(env, &key);
    }
    ts
}

pub fn remove_dispute_timestamp(env: &Env, agreement_id: u64) {
    let key = DataKey::DisputeTimestamp(agreement_id);
    env.storage().persistent().remove(&key);
}

pub fn get_milestone_rejections(env: &Env, agreement_id: u64, milestone_idx: u32) -> u32 {
    let key = DataKey::MilestoneRejectionCount(agreement_id, milestone_idx);
    env.storage().persistent().get(&key).unwrap_or(0u32)
}

pub fn increment_milestone_rejections(env: &Env, agreement_id: u64, milestone_idx: u32) -> u32 {
    let key = DataKey::MilestoneRejectionCount(agreement_id, milestone_idx);
    let count: u32 = env.storage().persistent().get(&key).unwrap_or(0u32);
    let next_count = count.saturating_add(1);
    env.storage().persistent().set(&key, &next_count);
    bump_persistent_ttl(env, &key);
    next_count
}

pub fn load_protocol_metrics(env: &Env) -> ProtocolMetrics {
    env.storage()
        .instance()
        .get(&DataKey::ProtocolMetrics)
        .unwrap_or(ProtocolMetrics {
            total_escrow_volume: 0,
            total_agreements_created: 0,
            total_agreements_completed: 0,
            total_agreements_disputed: 0,
        })
}

pub fn save_protocol_metrics(env: &Env, metrics: &ProtocolMetrics) {
    env.storage()
        .instance()
        .set(&DataKey::ProtocolMetrics, metrics);
    bump_instance_ttl(env);
}

pub const STORAGE_KEY_PREFIX_ADMIN: &str = "Admin";
pub const STORAGE_KEY_PREFIX_AGREEMENT: &str = "Agreement";
pub const STORAGE_KEY_PREFIX_DISPUTE: &str = "Dispute";
pub const STORAGE_KEY_PREFIX_METRICS: &str = "ProtocolMetrics";
