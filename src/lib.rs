use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Gas, NearToken, Promise};
use near_sdk::collections::UnorderedMap;
use serde_json::json;

mod player;
use crate::player::Player;

const MIN_GIFT_AMOUNT: u128 = 10_000_000;
const MAX_GIFT_AMOUNT: u128 = 300_000_000;
const TRANSFER_GAS: Gas = Gas::from_gas(10_000_000_000_000);
const NO_DEPOSIT: NearToken = NearToken::from_near(0);

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

    pub fn open_chest(&mut self) {
        let account_id = env::signer_account_id();
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        player.open_chest();
        self.players.insert(&account_id, &player);
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
        let mut player = self.players.get(&account_id).expect("Player does not exist");

        if player.gift == 0 {
            env::panic_str("No gifts available to swap");
        }

        let random_value = env::random_seed();
        let gift_amount = MIN_GIFT_AMOUNT + (u128::from(random_value[0]) % (MAX_GIFT_AMOUNT - MIN_GIFT_AMOUNT + 1));

        Promise::new(self.token_contract.clone()).function_call(
            "ft_transfer".to_string(),
            json!({
                "receiver_id": account_id,
                "amount": U128(gift_amount),
            }).to_string().into_bytes(),
            NO_DEPOSIT, // attached deposit
            TRANSFER_GAS,
        );

        player.gift -= 1;
        self.players.insert(&account_id, &player);
    }
}
