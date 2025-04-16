use libp2p::{
    core::upgrade, floodsub::{self, Floodsub, FloodsubEvent, Topic}, identity, mdns, noise, swarm::{behaviour, NetworkBehaviourEventProcess, SwarmEvent}, tcp, yamux, NetworkBehaviour, PeerId, Swarm
};


use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::{io, sync::mpsc};
use futures::StreamExt;
use std::error::Error;
use crate::blockchain::{Block, create_block};
use crate::tamagotchi::pet::Tamagotchi;


//we generate a random peer id
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| {
    let keypair = identity::Keypair::generate_ed25519();
    PeerId::from(keypair.public())
});


// //type to represent the events that can occur in the network
// pub enum NetworkEvent {
//     //a new peer has connected to the network
//     PeerConnected(PeerId),
//     //a new peer has disconnected from the network
//     PeerDisconnected(PeerId),
// }

pub enum NetworkEvent {
    NewBlock(Block),
    SyncRequest, 
    ChainResponse(Vec<Block>),
    PetUpdate(Tamagotchi),
}


// Message types for the network communications 
#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    //a new block to be added to the blockchain
    NewBlock { block:Block },
    SyncRequest { latest_block_idx: u64},
    ChainResponse {blocks: Vec<Block>},
    PetUpdate { pet: Tamagotchi},
}


//network behaviour which combines multiple protocols
#[derive(NetworkBehaviour)]
pub struct TamagotchiBehavior {
    floodsub: Floodsub,
    mdns: libp2p::Mdns::behaviour,
    #[behavior(ignore)]
    response_sender: mpsc::UnboundedSender<NetworkEvent>
    // Purpose: Bridge between networking code and your game logic
    // How it works:
    // When network events happen (new peer, message received), they're sent through this channel
    // our game code can react to these events (like adding a new block to the chain)
}


//Main network service that handles P2P communication
pub struct NetworkService {
    swarm: Swarm<TamagotchiBehavior>,
    event_receiver: mpsc::UnboundedReceiver<NetworkEvent>,
    peers: HashSet<PeerId>,
    blocks: Vec<Block>,
    pet: Option<Tamagotchi>,
}


//define topic names
const BLOCK_TOPIC: &str = "blocks";
const CHAIN_TOPIC: &str = "chain";
const PET_TOPIC: &str = "pet";

//impl
        



pub mod service;

