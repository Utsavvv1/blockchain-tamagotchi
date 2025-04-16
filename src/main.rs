use std::fs;
use dialoguer::{Select, Input};
use tamagotchi::pet::Tamagotchi;


mod tamagotchi;
mod blockchain;
use blockchain::{Block, create_block, create_genesis_block};

mod network;
use network::{NetworkService, NetworkEvent, NetworkServiceExt};

fn save_state(pet: &Tamagotchi, blockchain: &Vec<Block>) -> std::io::Result<()> {

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