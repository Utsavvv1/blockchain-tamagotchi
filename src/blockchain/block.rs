use sha2:: {Digest, Sha256};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,                     //basically position in chain
    pub timestamp: DateTime<Utc>,       //when did the action occur
    pub action: String,                 // eg. "Fed"
    pub previous_hash: String,          
    pub hash: String,                   //current block fingerprint 
}

impl Block {
    
    pub fn create_genesis_block() -> Block {
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


    pub fn create_block(action: &str, previous_block: &Block) -> Block {
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
}