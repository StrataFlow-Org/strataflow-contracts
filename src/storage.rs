use crate::types::*;
use soroban_sdk::Env;

pub fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn bump_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
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
