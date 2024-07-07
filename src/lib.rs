use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Gas, NearToken, Promise, PromiseResult};
use near_sdk::collections::UnorderedMap;
use serde_json::json;

mod player;
mod externalcontract;
use crate::player::Player;
use crate::externalcontract::*;

const MIN_GIFT_AMOUNT: u128 = 10_000_000;
const MAX_GIFT_AMOUNT: u128 = 300_000_000;
const TRANSFER_GAS: Gas = Gas::from_gas(10_000_000_000_000);
const NO_DEPOSIT: NearToken = NearToken::from_yoctonear(1);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub players: UnorderedMap<AccountId, Player>,
    pub token_contract: AccountId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            players: UnorderedMap::new(b"p".to_vec()),
            token_contract: "blackdragontoken.testnet".parse().unwrap(),
        }
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Contract is already initialized");
        Self {
            players: UnorderedMap::new(b"p".to_vec()),
            token_contract: "blackdragontoken.testnet".parse().unwrap(),
        }
    }
    
    pub fn get_player(&self, account_id: AccountId) -> Option<Player> {
        self.players.get(&account_id)
    }

    pub fn create_player(&mut self) {
        let account_id = env::signer_account_id();
        if self.players.get(&account_id).is_none() {
            let new_player = Player::new(env::block_timestamp());
            self.players.insert(&account_id, &new_player);
        }
    }

    pub fn claim_key(&mut self) {
        let account_id = env::signer_account_id();
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        player.claim_key();
        self.players.insert(&account_id, &player);
    }

    pub fn exchange_chest(&mut self) {
        let account_id = env::signer_account_id();
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        player.exchange_chest();
        self.players.insert(&account_id, &player);
    }

    pub fn open_chest(&mut self) -> String {
        let account_id = env::signer_account_id();
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        let result : String = player.open_chest();
        self.players.insert(&account_id, &player);
        result
    }

        pub fn upgrade(&mut self) {
        let account_id = env::signer_account_id();
        if let Some(mut player) = self.players.get(&account_id) {
            player.upgrade();
            self.players.insert(&account_id, &player);
        }
    }

    pub fn swap_gift(&mut self) {
        let account_id = env::signer_account_id();
        let player = self.players.get(&account_id).expect("Player does not exist");

        if player.gift == 0 {
            env::panic_str("No gifts available to swap");
        }

        // Call ft_balance_of to get the current balance of the contract
        ext_ft::ext(self.token_contract.clone())
            .with_static_gas(TRANSFER_GAS)
            .ft_balance_of(env::current_account_id())
            .then(Self::ext(env::current_account_id()).with_static_gas(TRANSFER_GAS)
            .on_ft_balance_of(account_id));
    }
    

    #[private]
    pub fn on_ft_balance_of(&mut self, account_id: AccountId) -> String {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        let balance: U128 = match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                serde_json::from_slice::<U128>(&result).expect("Failed to parse the balance")
            }
            _ => env::panic_str("Failed to fetch the balance"),
        };

        let mut player = self.players.get(&account_id).expect("Player does not exist");

        let random_value = env::random_seed();
        let rand_val: u8 = random_value[0] % 100;
        let result: String;

        if balance.0 == 0 {
            // If balance is zero, award keys
            if rand_val < 15 {
                player.keys += 40;
                result = format!("Keys: {}", 40);
            } else if rand_val < 50 {
                player.keys += 20;
                result = format!("Keys: {}", 20);
            } else {
                player.keys += 10;
                result = format!("Keys: {}", 10);
            }
        } 
        else {
            // If balance > zero, award keys and 15% to rewards tokens
            if rand_val < 10 {
                player.keys += 40;
                result = format!("Keys: {}", 40);
            } else if rand_val < 30 {
                player.keys += 20;
                result = format!("Keys: {}", 20);
            } else if rand_val < 85 {
                player.keys += 10;
                result = format!("Keys: {}", 10);
            } else {
                let rand_gift:u32 = u32::from(random_value[0]) % 10000000;
                let gift_amount = (MIN_GIFT_AMOUNT + (u128::from(rand_gift) % (MAX_GIFT_AMOUNT - MIN_GIFT_AMOUNT + 1))) * 1_000_000_000_000_000_000_000_000;
                if balance.0 < gift_amount {
                    // If balance is less than the random amount, award the remaining balance
                    Promise::new(self.token_contract.clone()).function_call(
                        "ft_transfer".to_string(),
                        json!({
                            "receiver_id": account_id,
                            "amount": U128(balance.0),
                        }).to_string().into_bytes(),
                        NO_DEPOSIT,
                        TRANSFER_GAS,
                    );
                    result = format!("Tokens: {}", balance.0);
                    player.token_rewarded += balance.0;
                    player.last_token_rewarded = balance.0;
                } else {
                    // If balance is sufficient, award the random amount
                    Promise::new(self.token_contract.clone()).function_call(
                        "ft_transfer".to_string(),
                        json!({
                            "receiver_id": account_id,
                            "amount": U128(gift_amount),
                        }).to_string().into_bytes(),
                        NO_DEPOSIT,
                        TRANSFER_GAS,
                    );
                    result = format!("Tokens: {}", gift_amount);
                    player.token_rewarded += gift_amount;
                    player.last_token_rewarded = gift_amount;
                }
            }
        }
        player.gift -= 1;
        self.players.insert(&account_id, &player);

        result
    }
}
