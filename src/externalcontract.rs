// Find all our documentation at https://docs.near.org
use near_sdk::{ext_contract, json_types::U128};


#[ext_contract(ext_ft)]
pub trait Blackdragontoken {
    fn ft_balance_of(&self, account_id: near_sdk::AccountId) -> U128;
}