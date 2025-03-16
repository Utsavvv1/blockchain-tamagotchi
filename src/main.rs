use sha2:: {Digest, Sha256};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use dialoguer::{Select, Input};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tamagotchi {
    name: String,
    hunger: u32,
    happiness: u32,
    cleanliness: u32,
    last_interaction: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,                     //basically position in chain
    timestamp: DateTime<Utc>,       //when did the action occur
    action: String,                 // eg. "Fed"
    previous_hash: String,          
    hash: String,                   //current block fingerprint 
}


impl Tamagotchi {
    fn new(name: &str) -> Self {
        Tamagotchi {
            name: name.to_string(),
            hunger: 50,          
            happiness: 50,       
            cleanliness: 50,     
            last_interaction: Utc::now(),
        }
    }

    fn update_time(&mut self) {
        self.last_interaction = Utc::now();
    }

    fn feed(&mut self) {
        self.hunger = self.hunger.saturating_sub(10).min(100); 
        self.happiness = self.happiness.saturating_add(5).min(100); 
        self.update_time();
    }

    fn play(&mut self) {
        self.happiness = self.happiness.saturating_add(10).min(100);
        self.cleanliness = self.cleanliness.saturating_sub(5).min(100);
        self.update_time();
    }

    fn clean(&mut self) {
        self.cleanliness = self.cleanliness.saturating_add(100).min(100); 
    }

    fn update_stats(&mut self) {
        let now = Utc::now();
        let secs_passed = (now - self.last_interaction).num_seconds() as u32;
        self.hunger = self.hunger.saturating_add(secs_passed / 10).min(100);
        self.happiness = self.happiness.saturating_sub(secs_passed / 15).min(100);
        self.cleanliness = self.cleanliness.saturating_sub(secs_passed / 10).min(100);
        self.last_interaction = now;
    }
}




fn create_genesis_block() -> Block {
    let mut hasher = Sha256::new();
    hasher.update(b"genesis");
    let hash = format!("{:x}", hasher.finalize());          //converts the byte array returned by hasher.finalise to lowercase hexadecimal string

    Block {
        index: 0,
        timestamp: Utc::now(),
        action: "GENESIS BLOCK" .to_string(),
        previous_hash: "0".to_string(),
        hash,
    }
}


fn create_block(action: &str, previous_block: &Block) -> Block {
    let mut hasher = Sha256::new();

    let input = format!("{}{}{}", previous_block.hash, action, Utc::now());
    hasher.update(input);
    let hash = format!("{:x}", hasher.finalize());

    Block {
        index: previous_block.index + 1,
        timestamp: Utc::now(),
        action: action.to_string(),
        previous_hash: previous_block.hash.clone(),     //creates and immutable link
        hash,
    }
}



fn save_state(pet: &Tamagotchi, blockchain: &[Block]) -> std::io::Result<()> {

    fs::write("pet_state.json", serde_json::to_string(pet)?)?;
    fs::write("blockchain.json", serde_json::to_string(blockchain)?)?;
    Ok(())
}


fn load_state() -> Option<(Tamagotchi, Vec<Block>)> {
    let pet = fs::read_to_string("pet_state.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    let chain = fs::read_to_string("blockchain.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    match(pet, chain) {
        (Some(p), Some(c)) => Some ((p, c)),
        _ => None,
    }
}

fn main() {

    let (mut pet, mut blockchain) = match load_state() {
        Some((p,c)) => (p, c),          //success
        None => {
            let name: String = Input::new()
                .with_prompt("Name your Tamagotchi")
                .interact()
                .unwrap();
            let pet = Tamagotchi::new(&name);
            let blockchain = vec![create_genesis_block()];

            (pet, blockchain)
        }
    };


    loop {
        pet.update_stats();

        println!("\n=== {} ===", pet.name);
        println!("Hunger: {}", pet.hunger);
        println!("Happiness: {}", pet.happiness);
        println!("Cleanliness: {}", pet.cleanliness);
        println!("\nBlockchain length: {}", blockchain.len());

        let selection = Select::new()
            .items(&["Feed", "Play", "Clean", "View Blockchain", "Quit"])
            .interact()
            .unwrap();


        match selection {
            0..=2 => {
                let action = match selection {
                    0 => { pet.feed(); "Fed Pet"},
                    1 => { pet.play(); "Played with pet"},
                    _ => { pet.clean(); "Pet cleaned!"},
                };

                let new_block = create_block(action, blockchain.last().unwrap());
                blockchain.push(new_block);
            }

            3 => {
                println!("\nBlockchain Ledger:");
                for block in &blockchain {
                    println!(
                        "[{}] {} - Previous: {}... Current: {}...",
                        block.index,
                        block.action,
                        if block.previous_hash.len() >= 6 {
                            &block.previous_hash[0..6]
                        } else {
                            &block.previous_hash
                        }, // truncated
                        if block.hash.len() >= 6 {
                            &block.hash[0..6]
                        } else {
                            &block.hash
                        }, // truncated
                    );
                }
            }
            _ => {
                save_state(&pet, &blockchain).unwrap();
                break;
            }
        }
    }
}