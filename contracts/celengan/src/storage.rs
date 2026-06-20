use soroban_sdk::{Address, Env, Vec};
use crate::types::{DataKey, Error, PiggyBank};

pub fn get_next_id(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::NextId).unwrap_or(0u32)
}

pub fn increment_id(env: &Env) -> u32 {
    let id = get_next_id(env);
    env.storage().instance().set(&DataKey::NextId, &(id + 1));
    env.storage().instance().extend_ttl(100, 100);
    id
}

pub fn save_bank(env: &Env, id: u32, bank: &PiggyBank) {
    env.storage().persistent().set(&DataKey::PiggyBank(id), bank);
    env.storage().persistent().extend_ttl(&DataKey::PiggyBank(id), 100, 100);
}

pub fn get_bank(env: &Env, id: u32) -> Result<PiggyBank, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::PiggyBank(id))
        .ok_or(Error::BankNotFound)
}

pub fn get_owner_banks(env: &Env, owner: &Address) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerBanks(owner.clone()))
        .unwrap_or(Vec::new(env))
}

pub fn append_owner_bank(env: &Env, owner: &Address, id: u32) {
    let mut banks = get_owner_banks(env, owner);
    banks.push_back(id);
    env.storage().persistent().set(&DataKey::OwnerBanks(owner.clone()), &banks);
    env.storage().persistent().extend_ttl(&DataKey::OwnerBanks(owner.clone()), 100, 100);
}
