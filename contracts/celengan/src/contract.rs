use soroban_sdk::{contract, contractimpl, token, Address, Env, String, Vec};
use crate::types::{Error, LockCondition, PiggyBank};
use crate::storage;

#[contract]
pub struct PiggyBankContract;

#[contractimpl]
impl PiggyBankContract {
    pub fn create(
        env: Env,
        owner: Address,
        name: String,
        token: Address,
        condition: LockCondition,
    ) -> Result<u32, Error> {
        owner.require_auth();

        let id = storage::increment_id(&env);
        let bank = PiggyBank {
            owner: owner.clone(),
            name,
            balance: 0,
            token,
            condition,
            created_at: env.ledger().timestamp(),
            is_withdrawn: false,
        };
        storage::save_bank(&env, id, &bank);
        storage::append_owner_bank(&env, &owner, id);

        env.events().publish(("piggy_bank", "created"), (id, owner));
        Ok(id)
    }

    pub fn deposit(
        env: Env,
        caller: Address,
        bank_id: u32,
        amount: i128,
    ) -> Result<(), Error> {
        caller.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut bank = storage::get_bank(&env, bank_id)?;

        if bank.is_withdrawn {
            return Err(Error::AlreadyWithdrawn);
        }

        let token_client = token::Client::new(&env, &bank.token);
        token_client.transfer(&caller, &env.current_contract_address(), &amount);

        bank.balance += amount;
        storage::save_bank(&env, bank_id, &bank);

        env.events().publish(("piggy_bank", "deposited"), (bank_id, caller, amount));
        Ok(())
    }

    pub fn withdraw(
        env: Env,
        owner: Address,
        bank_id: u32,
    ) -> Result<(), Error> {
        owner.require_auth();

        let mut bank = storage::get_bank(&env, bank_id)?;

        if bank.owner != owner {
            return Err(Error::NotOwner);
        }

        if bank.is_withdrawn {
            return Err(Error::AlreadyWithdrawn);
        }

        if !Self::check_unlocked(&env, &bank) {
            return Err(Error::ConditionNotMet);
        }

        let amount = bank.balance;
        let token_client = token::Client::new(&env, &bank.token);
        token_client.transfer(&env.current_contract_address(), &owner, &amount);

        bank.balance = 0;
        bank.is_withdrawn = true;
        storage::save_bank(&env, bank_id, &bank);

        env.events().publish(("piggy_bank", "withdrawn"), (bank_id, owner, amount));
        Ok(())
    }

    pub fn get_bank(env: Env, bank_id: u32) -> Result<PiggyBank, Error> {
        storage::get_bank(&env, bank_id)
    }

    pub fn get_owner_banks(env: Env, owner: Address) -> Vec<u32> {
        storage::get_owner_banks(&env, &owner)
    }

    pub fn is_unlocked(env: Env, bank_id: u32) -> Result<bool, Error> {
        let bank = storage::get_bank(&env, bank_id)?;
        Ok(Self::check_unlocked(&env, &bank))
    }

    fn check_unlocked(env: &Env, bank: &PiggyBank) -> bool {
        let now = env.ledger().timestamp();
        match &bank.condition {
            LockCondition::TargetAmount(min) => bank.balance >= *min,
            LockCondition::TargetTime(unlock_at) => now >= *unlock_at,
            LockCondition::Both(amount, time) => { // ✅ tuple destructure
                bank.balance >= *amount && now >= *time
            }
        }
    }
}
