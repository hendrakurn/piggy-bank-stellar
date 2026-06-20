#![cfg(test)]

use super::*;
use contract::PiggyBankContractClient;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn setup_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);

    // Create a mock token (Stellar Asset)
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    // Mint tokens to owner
    let sac = StellarAssetClient::new(&env, &token_address);
    sac.mint(&owner, &10_000_000);

    (env, owner, token_address, admin)
}

fn deploy_contract(env: &Env) -> PiggyBankContractClient {
    let contract_id = env.register(PiggyBankContract, ());
    PiggyBankContractClient::new(env, &contract_id)
}

#[test]
fn create_bank_success() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name = String::from_str(&env, "My Savings");
    let condition = LockCondition::TargetAmount(1_000_000);

    let id = client.create(&owner, &name, &token, &condition).unwrap();
    assert_eq!(id, 0);

    let bank = client.get_bank(&id).unwrap();
    assert_eq!(bank.owner, owner);
    assert_eq!(bank.balance, 0);
    assert_eq!(bank.is_withdrawn, false);
}

#[test]
fn deposit_increases_balance() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name = String::from_str(&env, "Savings");
    let condition = LockCondition::TargetAmount(1_000_000);
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    client.deposit(&owner, &id, &500_000).unwrap();

    let bank = client.get_bank(&id).unwrap();
    assert_eq!(bank.balance, 500_000);
}

#[test]
fn withdraw_before_amount_condition_fails() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name = String::from_str(&env, "Savings");
    let condition = LockCondition::TargetAmount(1_000_000);
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    client.deposit(&owner, &id, &500_000).unwrap();

    let result = client.withdraw(&owner, &id);
    assert_eq!(result, Err(Ok(Error::ConditionNotMet)));
}

#[test]
fn withdraw_after_amount_target_met() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name = String::from_str(&env, "Savings");
    let target = 1_000_000i128;
    let condition = LockCondition::TargetAmount(target);
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    client.deposit(&owner, &id, &target).unwrap();

    client.withdraw(&owner, &id).unwrap();

    let bank = client.get_bank(&id).unwrap();
    assert_eq!(bank.balance, 0);
    assert_eq!(bank.is_withdrawn, true);
}

#[test]
fn withdraw_after_time_target_met() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let unlock_time = 1_000u64;
    env.ledger().set_timestamp(0);

    let name = String::from_str(&env, "Time Savings");
    let condition = LockCondition::TargetTime(unlock_time);
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    client.deposit(&owner, &id, &500_000).unwrap();

    // Time not met yet
    let result = client.withdraw(&owner, &id);
    assert_eq!(result, Err(Ok(Error::ConditionNotMet)));

    // Advance time past unlock
    env.ledger().set_timestamp(unlock_time + 1);

    client.withdraw(&owner, &id).unwrap();

    let bank = client.get_bank(&id).unwrap();
    assert_eq!(bank.is_withdrawn, true);
}

#[test]
fn withdraw_after_both_conditions_met() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let target_amount = 1_000_000i128;
    let unlock_time = 500u64;
    env.ledger().set_timestamp(0);

    let name = String::from_str(&env, "Both Savings");
    let condition = LockCondition::Both {
        amount: target_amount,
        time: unlock_time,
    };
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    // Deposit enough
    client.deposit(&owner, &id, &target_amount).unwrap();

    // Time not met yet — should fail
    let result = client.withdraw(&owner, &id);
    assert_eq!(result, Err(Ok(Error::ConditionNotMet)));

    // Advance time
    env.ledger().set_timestamp(unlock_time + 1);

    // Now both conditions met
    client.withdraw(&owner, &id).unwrap();

    let bank = client.get_bank(&id).unwrap();
    assert_eq!(bank.is_withdrawn, true);
}

#[test]
fn non_owner_cannot_withdraw() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name = String::from_str(&env, "Savings");
    let condition = LockCondition::TargetAmount(0); // always unlocked
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    client.deposit(&owner, &id, &100_000).unwrap();

    let attacker = Address::generate(&env);
    let result = client.withdraw(&attacker, &id);
    assert_eq!(result, Err(Ok(Error::NotOwner)));
}

#[test]
fn deposit_to_nonexistent_bank_fails() {
    let (env, owner, _token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let result = client.deposit(&owner, &999, &100_000);
    assert_eq!(result, Err(Ok(Error::BankNotFound)));
}

#[test]
fn double_withdraw_fails() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name = String::from_str(&env, "Savings");
    let condition = LockCondition::TargetAmount(0); // always unlocked
    let id = client.create(&owner, &name, &token, &condition).unwrap();

    client.deposit(&owner, &id, &100_000).unwrap();
    client.withdraw(&owner, &id).unwrap();

    let result = client.withdraw(&owner, &id);
    assert_eq!(result, Err(Ok(Error::AlreadyWithdrawn)));
}

#[test]
fn get_owner_banks_returns_correct_ids() {
    let (env, owner, token, _admin) = setup_env();
    let client = deploy_contract(&env);

    let name1 = String::from_str(&env, "Bank 1");
    let name2 = String::from_str(&env, "Bank 2");
    let condition = LockCondition::TargetAmount(1_000_000);

    let id1 = client.create(&owner, &name1, &token, &condition).unwrap();
    let id2 = client.create(&owner, &name2, &token, &condition).unwrap();

    let banks = client.get_owner_banks(&owner);
    assert_eq!(banks.len(), 2);
    assert_eq!(banks.get(0).unwrap(), id1);
    assert_eq!(banks.get(1).unwrap(), id2);
}
