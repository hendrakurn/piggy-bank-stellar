use soroban_sdk::{contracttype, contracterror, Address, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum LockCondition {
    TargetAmount(i128),
    TargetTime(u64),
    Both(i128, u64), // ✅ tuple, bukan named fields
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PiggyBank {
    pub owner: Address,
    pub name: String,
    pub balance: i128,
    pub token: Address,
    pub condition: LockCondition,
    pub created_at: u64,
    pub is_withdrawn: bool,
}

#[contracttype]
pub enum DataKey {
    PiggyBank(u32),
    OwnerBanks(Address),
    NextId,
}

#[contracterror] // ✅ sekarang diimport
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    BankNotFound = 1,
    NotOwner = 2,
    AlreadyWithdrawn = 3,
    ConditionNotMet = 4,
    InvalidAmount = 5,
}