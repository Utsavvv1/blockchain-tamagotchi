use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tamagotchi {
    pub name: String,
    pub hunger: u32,
    pub happiness: u32,
    pub cleanliness: u32,
    pub last_interaction: DateTime<Utc>,
}

impl Tamagotchi {
    pub fn new(name: &str) -> Self {
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

    pub fn feed(&mut self) {
        self.hunger = self.hunger.saturating_sub(10).min(100); 
        self.happiness = self.happiness.saturating_add(5).min(100); 
        self.update_time();
    }

    pub fn play(&mut self) {
        self.happiness = self.happiness.saturating_add(10).min(100);
        self.cleanliness = self.cleanliness.saturating_sub(5).min(100);
        self.update_time();
    }

    pub fn clean(&mut self) {
        self.cleanliness = self.cleanliness.saturating_add(100).min(100); 
    }

    pub fn update_stats(&mut self) {
        let now = Utc::now();
        let secs_passed = (now - self.last_interaction).num_seconds() as u32;
        self.hunger = self.hunger.saturating_add(secs_passed / 10).min(100);
        self.happiness = self.happiness.saturating_sub(secs_passed / 15).min(100);
        self.cleanliness = self.cleanliness.saturating_sub(secs_passed / 10).min(100);
        self.last_interaction = now;
    }
}
