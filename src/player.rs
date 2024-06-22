use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};

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
    pub time_to_decease: u64
}

impl Player {
    pub fn new(current_time: u64) -> Self {
        Self {
            keys: 0,
            keys_per_claim: 1,
            chests: 0,
            stone: 0,
            iron: 0,
            wood: 0,
            gift: 0,
            time_to_next_key_claimable: current_time,
            time_to_decease: 0
        }
    }

    pub fn claim_key(&mut self) {
        if env::block_timestamp() > self.time_to_next_key_claimable {
            self.keys += 1;
            self.time_to_next_key_claimable = env::block_timestamp() + 6 * 60 * 60 * 1_000_000_000 - (self.time_to_decease * 1_000_000_000); // 6 hours in nanoseconds
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

    pub fn open_chest(&mut self) {
        if self.keys > 0 && self.chests > 0 {
            self.keys -= 1;
            self.chests -= 1;

            let random_value = env::random_seed();
            let rand_val: u8 = random_value[0] % 100;

            if rand_val < 32 {
                self.wood += Self::random_amount(&[60, 35, 4, 1], &[10, 20, 50, 100]);
            } else if rand_val < 64 {
                self.iron += Self::random_amount(&[60, 35, 4, 1], &[10, 20, 50, 100]);
            } else if rand_val < 96 {
                self.stone += Self::random_amount(&[60, 35, 4, 1], &[10, 20, 50, 100]);
            } else {
                self.gift += 1;
            }
        } else {
            env::panic_str("Not enough keys or chests to open");
        }
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


    pub fn upgrade(&mut self, account_id: AccountId) {

        let keys_needed = 2 * self.keys_per_claim;
        assert!(self.keys >= keys_needed, "Not enough keys for upgrade");

        self.keys -= keys_needed;
        if self.time_to_decease < 240 * 60 * 1_000_000_000 {
            self.time_to_decease += 15 * 60 * 1_000_000_000; // 15 minutes in nanoseconds
        }

        if self.time_to_decease % (60 * 60 * 1_000_000_000) == 0 && self.time_to_decease <= 240 * 60 * 1_000_000_000 {
            if self.keys_per_claim < 4 {
                self.keys_per_claim += 1;
            }
            else {
                env::panic_str("Reached max level!");
            }
        }
    }
}
