use super::*;
use anyhow::Result;
use libp2p::futures::prelude::*;
use libp2p::swarm::SwarmBuilder;
use std::time::Duration;

impl NetworkService {
    //creates a new network service
    pub async fn new(initial_blocks: Vec<Block>, initial_pet:Option<Tamagotchi>) -> Result<Self> {
        //create random key
        let keyapir = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keyapir.public_key());
        println!("Local peer id: {}", peer_id);

        //channel for communication between network behavior and service
        let (response_sender, event_receiver) = mpsc::unbounded_channel();

        // Set up topics for different message types
        let block_topic = Topic::new(BLOCK_TOPIC);
        let chain_topic = Topic::new(CHAIN_TOPIC);
        let pet_topic = Topic::new(PET_TOPIC);

        // /setting transport with encryption n multiplexing
        let transport = tcp::async_io::Transport::new(tcp::Config::default());
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(keypair)?)
            .multiplex(yamux::Config::default())
            .boxed();


        //setting up our network behavior
        let mut behaviour = TamagotchiBehavior {
            floodsub: Floodsub::new(peer_id),
            mdns: mdns::async_io::Behaviour::new(mdns::Config::default(), peer_id)?,
            response_sender,
        };

        //subscr
        // Subscribe to the topics
        behaviour.floodsub.subscribe(block_topic);
        behaviour.floodsub.subscribe(chain_topic);
        behaviour.floodsub.subscribe(pet_topic);

        // Build the Swarm that manages our network connection
        let swarm = SwarmBuilder::with_async_std_executor(
            transport,
            behaviour, 
            peer_id,
        ).build();

        Ok(Self {
            swarm, event_receiver, peers: HashSet::new(), blocks: initial_blocks, pet: initial_pet,
        })

    }

    //starts the network service
    pub async fn run(&mut self, listen_addr: String) -> Result<()> {
        //listen for incoming connections
        self.swarm.listen_on(listen_addr.parse()?)?;

        //main event loop
        loop {
            tokio::select! {
                //handle network events
                event = self.swarm.select_next_some() => match event{
                    //new connection established
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("New connection established: {}", peer_id);
                        self.peers.insert(peer_id);
                    }
                    // Connection closed
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        println!("Disconnected from: {}", peer_id);
                        self.peers.remove(&peer_id);
                    }
                    //hanfle behaviour events 
                    SwarmEvent::Behaviour(event) => self.handle_behaviour_event(event).await?,
                    _ => {}
                },


                //handle events from our channel 
                event = self.event_receiver.recv() => match event {
                    Some(NetworkEvent::NewBlock(block)) => self.broadcast_new_block(block).await?,
                    Some(NetworkEvent::SyncRequest) => self.request_chain_sync().await?,
                    Some(NetworkEvent::ChainResponse(blocks)) => self.handle_chain_response(blocks).await?,
                    Some(NetworkEvent::PetUpdate(pet)) => self.broadcast_pet_update(pet).await?,
                    None => break,
                }
            }
        }
    }

    //handles behaviour events
    async fn broadcast_new_block(&mut self, block: Block) -> Result<()> {
        let message = NetworkMessage::NewBlock{block }
        let data = serde_json::to_string(&message)?;
        self.swarm.behaviour_mut().floodsub.publish(
            Topic::new(BLOCK_TOPIC),
            data.as_bytes(),
        );
        Ok(())
    }

    // request chain synchronization from peers
    async fn request_chain_sync(&mut self) -> Result<()> {
        let latest_index = match self.blocks.last() {
            Some(block) => block.index,
            None => 0,
        };


        let message = NetworkMessage::SyncRequest { latest_block_idx: latest_index };
        let data = serde_json::to_string(&message)?;
        self.swarm.behaviour_mut().floodsub.publish(
            Topic::new(CHAIN_TOPIC),
            data.as_bytes(),
        );
        Ok(())
    }

      // Broadcast pet state update
      async fn broadcast_pet_update(&mut self, pet: Tamagotchi) -> Result<()> {
        let message = NetworkMessage::PetUpdate { pet };
        let data = serde_json::to_string(&message)?;
        self.swarm.behaviour_mut().floodsub.publish(
            Topic::new(PET_TOPIC),
            data.as_bytes(),
        );
        Ok(())
    }

    // Handle chain response from peers
    async fn handle_chain_response(&mut self, blocks: Vec<Block>) -> Result<()> {
        // Simple validation - if the received chain is longer, replace ours
        // In a real blockchain, you'd validate the entire chain
        if blocks.len() > self.blocks.len() {
            // Validate chain integrity
            if self.validate_chain(&blocks) {
                println!("Received a longer valid chain. Replacing local chain.");
                self.blocks = blocks;
            }
        }
        
        Ok(())
    }

     // Simple chain validation
     fn validate_chain(&self, blocks: &[Block]) -> bool {
        // Check if it's empty
        if blocks.is_empty() {
            return false;
        }
        
        // Validate each block
        for i in 1..blocks.len() {
            let current = &blocks[i];
            let previous = &blocks[i - 1];
            
            // Check if indexes are sequential
            if current.index != previous.index + 1 {
                return false;
            }
            
            // Check if previous hash matches
            if current.previous_hash != previous.hash {
                return false;
            }
            
            // Could add more validation like checking hash validity
        }
        
        true
    }
        
