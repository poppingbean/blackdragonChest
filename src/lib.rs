use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use near_sdk::serde::{Deserialize, Serialize};
use rand::Rng;

//const TOKEN_CONTRACT: &str = "blackdragon.tkn.near";
const MIN_GIFT_AMOUNT: u128 = 10_000_000;
const MAX_GIFT_AMOUNT: u128 = 300_000_000;
const INITIAL_TIME_CLAIMABLE_HOURS: u64 = 8;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct Player {
    pub keys: u32,
    pub chests: u32,
    pub stone: u32,
    pub iron: u32,
    pub wood: u32,
    pub gift: u32,
    pub time_to_next_key_claimable: u64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub players: UnorderedMap<AccountId, Player>,
    pub total_supply: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(total_supply: u128) -> Self {
        Self {
            players: UnorderedMap::new(b"p".to_vec()),
            total_supply: total_supply,
        }
    }

    pub fn init_player(&mut self, account_id: AccountId) {
        let player = Player {
            keys: 0,
            chests: 0,
            stone: 0,
            iron: 0,
            wood: 0,
            gift: 0,
            time_to_next_key_claimable: env::block_timestamp() + INITIAL_TIME_CLAIMABLE_HOURS * 60 * 60 * 1_000_000_000,
        };
        self.players.insert(&account_id, &player);
    }

    pub fn claim_key(&mut self, account_id: AccountId) {
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        assert!(env::block_timestamp() > player.time_to_next_key_claimable, "Cannot claim key yet");
        player.keys += 1;
        player.time_to_next_key_claimable = env::block_timestamp() + INITIAL_TIME_CLAIMABLE_HOURS * 60 * 60 * 1_000_000_000;
        self.players.insert(&account_id, &player);
    }

    pub fn open_chest(&mut self, account_id: AccountId) {
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        assert!(player.keys > 0, "Not enough keys");
        assert!(player.chests > 0, "Not enough chests");
        player.keys -= 1;
        player.chests -= 1;

        let mut rng = rand::thread_rng();
        let roll: i32 = rng.gen_range(1,100);
        match roll {
            1..=32 => player.wood += self.random_resource_quantity(),
            33..=64 => player.iron += self.random_resource_quantity(),
            65..=96 => player.stone += self.random_resource_quantity(),
            _ => player.gift += 1,
        }

        self.players.insert(&account_id, &player);
    }

    pub fn exchange_chest(&mut self, account_id: AccountId) {
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        if player.wood >= 50 && player.iron >= 50 && player.stone >= 50 {
            player.wood -= 50;
            player.iron -= 50;
            player.stone -= 50;
            player.chests += 1;
        } else {
            assert!(player.keys > 0, "Not enough resources or keys");
            player.keys -= 1;
            player.chests += 1;
        }
        self.players.insert(&account_id, &player);
    }

    pub fn swap_gift(&mut self, account_id: AccountId) {
        let mut player = self.players.get(&account_id).expect("Player does not exist");
        assert!(player.gift > 0, "Not enough gifts");
        let gift_amount: u128 = self.random_gift_amount();
        assert!(self.total_supply >= gift_amount, "Not enough token supply");
        player.gift -= 1;
        self.total_supply -= gift_amount;
        // Code to transfer token to account_id here
        self.players.insert(&account_id, &player);
    }

    fn random_resource_quantity(&self) -> u32 {
        let mut rng = rand::thread_rng();
        let roll: i32 = rng.gen_range(1,100);
        match roll {
            1..=60 => 10,
            61..=95 => 20,
            96..=99 => 50,
            _ => 100,
        }
    }

    fn random_gift_amount(&self) -> u128 {
        let mut rng = rand::thread_rng();
        rng.gen_range(MIN_GIFT_AMOUNT,MAX_GIFT_AMOUNT)
    }
}
