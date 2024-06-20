use std::collections::HashMap;
use near_sdk::{ AccountId, Gas, PanicOnDefault};
use near_sdk::near;

//const TOKEN_CONTRACT: &str = "blackdragon.tkn.near";
const MIN_GIFT_AMOUNT: u128 = 10_000_000;
const MAX_GIFT_AMOUNT: u128 = 300_000_000;
const INITIAL_TIME_CLAIMABLE_HOURS: u64 = 8;
const FT_BALANCE_OF_GAS: Gas = Gas::from_gas(10_000_000_000_000);
const FOUR_HOURS: u64 = 4 * 60 * 60 * 1_000_000_000; 
const NO_DEPOSIT: u128 = 0;


#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct GameContract {
    players: HashMap<AccountId, Player>,
    token_contract: AccountId,
}

#[near(serializers = [borsh, json])]
pub struct Player {
    keys: u32,
    number_keys_per_claims: u32,
    chests: u32,
    stone: u32,
    iron: u32,
    wood: u32,
    gift: u32,
    time_decrease_from_default: u64,
    time_to_next_key_claimable: u64,
}

#[near]
impl GameContract {
    #[init]
    pub fn new(token_contract: AccountId) -> Self {
        Self {
            players: HashMap::new(),
            token_contract,
        }
    }

    pub fn get_player(&self, account_id: AccountId) -> Option<Player> {
        self.players.get(&account_id)
    }

    fn create_player(&mut self, account_id: AccountId) -> Player {
        let player = Player {
            keys: 0,
            number_keys_per_claims: 1,
            chests: 0,
            stone: 0,
            iron: 0,
            wood: 0,
            gift: 0,
            time_decrease_from_default: 0,
            time_to_next_key_claimable: env::block_timestamp() + FOUR_HOURS,
        };
        self.players.insert(&account_id, &player);
        player
    }

    pub fn sign_in(&mut self) {
        let account_id = env::predecessor_account_id();
        if self.players.get(&account_id).is_none() {
            self.create_player(account_id);
        }
    }

    pub fn claim_key(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut player = self.players.get(&account_id).expect("Player not found");
        let current_time = env::block_timestamp();
        assert!(current_time > player.time_to_next_key_claimable, "Cannot claim key yet");
        player.keys += 1;
        player.time_to_next_key_claimable = current_time + FOUR_HOURS;
        self.players.insert(&account_id, &player);
    }

    pub fn open_chest(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut player = self.players.get(&account_id).expect("Player not found");
        assert!(player.keys > 0, "Not enough keys");
        assert!(player.chests > 0, "Not enough chests");

        player.keys -= 1;
        player.chests -= 1;

        let mut rng: thread_rng();

        // Open the chest and determine what the player gets
        let reward_type = rng.gen_range(0..100);
        if reward_type < 32 {
            // Wood
            let amount = Self::get_random_amount();
            player.wood += amount;
        } else if reward_type < 64 {
            // Iron
            let amount = Self::get_random_amount();
            player.iron += amount;
        } else if reward_type < 96 {
            // Stone
            let amount = Self::get_random_amount();
            player.stone += amount;
        } else {
            // Gift
            player.gift += 1;
        }

        self.players.insert(&account_id, &player);
    }

    pub fn exchange_chest(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut player = self.players.get(&account_id).expect("Player not found");

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

    #[private]
    pub fn callback_swap_gift(&mut self, account_id: AccountId, amount: U128) {
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let balance: U128 = serde_json::from_slice::<U128>(&result).unwrap();
                let balance: u128 = balance.0;

                let amount_to_transfer = if balance >= amount.0 {
                    amount.0
                } else {
                    balance
                };

                if amount_to_transfer == 0 {
                    env::panic_str("Insufficient balance to transfer tokens");
                }

                Promise::new(self.token_contract.clone()).function_call(
                    "ft_transfer".to_string(),
                    json!({
                        "receiver_id": account_id,
                        "amount": U128(amount_to_transfer)
                    }).to_string().into_bytes(),
                    NO_DEPOSIT,
                    FT_TRANSFER_GAS,
                );

                let mut player = self.players.get(&account_id).expect("Player not found");
                player.gift -= 1;
                self.players.insert(&account_id, &player);
            }
            _ => env::panic_str("Failed to get balance from the token contract"),
        }
    }

    #[payable]
    pub fn swap_gift(&mut self) {
        let account_id = env::predecessor_account_id();
        let player = self.players.get(&account_id).expect("Player not found");
        assert!(player.gift > 0, "Not enough gifts");

        let mut rng = thread_rng();
        let amount: u128 = rng.gen_range(10_000_000..300_000_000);

        Promise::new(self.token_contract.clone()).function_call(
            "ft_balance_of".to_string(),
            json!({
                "account_id": self.token_contract.clone()
            }).to_string().into_bytes(),
            NO_DEPOSIT,
            FT_TRANSFER_GAS,
        ).then(
            Self::ext(env::current_account_id())
                .with_static_gas(FT_TRANSFER_GAS)
                .callback_swap_gift(account_id, U128(amount)),
        );
    }

    fn get_random_amount() -> u32 {
        let mut rng = thread_rng();
        let chance = rng.gen_range(0..100);
        match chance {
            0..=59 => 10,
            60..=94 => 20,
            95..=98 => 50,
            _ => 100,
        }
    }
}