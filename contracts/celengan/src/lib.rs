#![no_std]
mod contract;
mod storage;
mod types;

pub use contract::PiggyBankContract;
pub use types::{Error, LockCondition, PiggyBank};

#[cfg(test)]
mod test;
