use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Player {
    pub keys: u32,
    pub keys_per_claim: u32,
    pub chests: u32,
    pub stone: u32,
    pub iron: u32,
    pub wood: u32,
    pub gift: u32,
    pub time_to_next_key_claimable: u64,
    pub time_to_decease: u64,
    pub token_rewarded: u128,
    pub last_token_rewarded: u128,
    pub player_hearth: u64,
    pub player_defense: u64,
    pub player_attack: u64,
    pub player_luck: u64,
    pub player_energy: u64,
    pub player_crit: u64,
    pub player_level: u64,
    pub castle_level: u64,
    pub castle_hearth: u64,
    pub castle_defense: u64,
    pub castle_attack: u64
}

impl Player {
    pub fn new(current_time: u64) -> Self {
        Self {
            keys: 99,
            keys_per_claim: 1,
            chests: 99,
            stone: 0,
            iron: 0,
            wood: 0,
            gift: 20,
            time_to_next_key_claimable: current_time,
            time_to_decease: 0,
            token_rewarded: 0,
            last_token_rewarded: 0,
            player_hearth: 100,
            player_defense: 10,
            player_attack: 15,
            player_luck: 1,
            player_energy: 10,
            player_crit: 5,
            player_level: 1,
            castle_level: 1,
            castle_hearth: 10,
            castle_defense: 1,
            castle_attack: 2,
        }
    }

    pub fn claim_key(&mut self) {
        if env::block_timestamp() > self.time_to_next_key_claimable {
            self.keys += 1;
            //self.time_to_next_key_claimable = env::block_timestamp() + 6 * 60 * 60 * 1_000_000_000 - (self.time_to_decease); // 6 hours in nanoseconds ===> for mainnet
            self.time_to_next_key_claimable = env::block_timestamp() + 20 * 60 * 1_000_000_000 - (self.time_to_decease); //20 mins ===> for testnet
        } else {
            env::panic_str("Key claim is not yet available");
        }
    }

    pub fn exchange_chest(&mut self) {
        if self.wood >= 50 && self.stone >= 50 && self.iron >= 50 {
            self.chests += 1;
            self.wood -= 50;
            self.stone -= 50;
            self.iron -= 50;
        } else if self.keys > 0 {
            self.chests += 1;
            self.keys -= 1;
        } else {
            env::panic_str("Not enough resources or keys to exchange for a chest");
        }
    }

    pub fn open_chest(&mut self) -> String {
        let result: String;
        let extra_result: String;
        let old_val: u32;
        if self.keys > 0 && self.chests > 0 {
            self.keys -= 1;
            self.chests -= 1;

            let random_value = env::random_seed();
            let rand_val: u8 = random_value[0] % 100;

            if rand_val < 32 {
                old_val = self.wood;
                self.wood += Self::random_amount(&[60, 35, 4, 1], &[10, 20, 50, 100]);
                result = format!("Woods: {}", self.wood - old_val);
            } else if rand_val < 64 {
                old_val = self.iron;
                self.iron += Self::random_amount(&[60, 35, 4, 1], &[10, 20, 50, 100]);
                result = format!("Irons: {}", self.iron - old_val);
            } else if rand_val < 96 {
                old_val = self.stone;
                self.stone += Self::random_amount(&[60, 35, 4, 1], &[10, 20, 50, 100]);
                result = format!("Stones: {}", self.stone - old_val);
            } else {
                self.gift += 1;
                result = format!("Gift: {}", 1);
            }
            // Additional 10% chance for an extra gift
            let extra_gift_chance: u8 = random_value[1] % 100;
            if extra_gift_chance < 10 {
                self.gift += 1;
                extra_result = format!("Extra gift: 1");
            }
            else{
                extra_result = format!("");
            }
        } else {
            env::panic_str("Not enough keys or chests to open");
        }
        return format!("{}. {}", result, extra_result);
    }

    fn random_amount(chances: &[u8], amounts: &[u32]) -> u32 {
        let rand_val = env::random_seed()[0] % 100;
        let mut sum = 0;
        for (chance, amount) in chances.iter().zip(amounts.iter()) {
            sum += chance;
            if rand_val < sum {
                return *amount;
            }
        }
        0
    }


    pub fn upgrade(&mut self) {

        let keys_needed = 2 * self.keys_per_claim;
        assert!(self.keys >= keys_needed, "Not enough keys for upgrade");

        self.keys -= keys_needed;
        //for mainnet
        /* if self.time_to_decease < 240 * 60 * 1_000_000_000 {
            self.time_to_decease += 15 * 60 * 1_000_000_000; // 15 minutes in nanoseconds
        }

        if self.time_to_decease % (60 * 60 * 1_000_000_000) == 0 && self.time_to_decease <= 240 * 60 * 1_000_000_000 {
            if self.keys_per_claim < 4 {
                self.keys_per_claim += 1;
            }
            else {
                env::panic_str("Reached max level!");
            }
        } */
       //for testnet
       if self.time_to_decease < 15 * 60 * 1_000_000_000 {
            self.time_to_decease += 5 * 60 * 1_000_000_000; // 5 minutes in nanoseconds
        }

        if self.time_to_decease % (5 * 60 * 1_000_000_000) == 0 && self.time_to_decease <= 15 * 60 * 1_000_000_000 {
            if self.keys_per_claim < 4 {
                self.keys_per_claim += 1;
            }
            else {
                env::panic_str("Reached max level!");
            }
        }
    }
}
