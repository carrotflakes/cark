use crate::{
    direction::Direction,
    field::{Chunk, ChunkId},
    udp_stat::Sequence,
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Message {
    pub id: u64,
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub chunk_id: ChunkId,
    pub position: [f32; 2],
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Join {
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Joined {
    pub user_id: u64,
    pub chunk: Chunk,
    pub characters: Vec<JoinedCharacter>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct JoinedCharacter {
    pub id: u64,
    pub name: String,
    pub chunk_id: ChunkId,
    pub position: [f32; 2],
}

// #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
// pub struct UpdateField {
//     pub position: [u32; 2],
//     pub value: u8,
// }

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PublicChatMessage {
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Join(Join),
    PublicChatMessage(PublicChatMessage),
    // UpdateField(UpdateField),
    Position {
        chunk_id: ChunkId,
        position: [f32; 2],
        velocity: [f32; 2],
    },
    Leave,
    RequestChunk {
        id: ChunkId,
        direction: Direction,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ServerMessage {
    Joined(Joined),
    // UpdateField(UpdateField),
    PlayerJoined {
        id: u64,
        name: String,
        chunk_id: ChunkId,
        position: [f32; 2],
    },
    PlayerLeft {
        user_id: u64,
    },
    Position {
        user_id: u64,
        chunk_id: ChunkId,
        position: [f32; 2],
        velocity: [f32; 2],
    },
    Chunk {
        chunk: Chunk,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ClientUdpMessage {
    Init {
        id: u64,
    },
    Message {
        sequence: Sequence,
        message: ClientMessage,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ServerUdpMessage {
    Init,
    Message {
        sequence: Sequence,
        message: ServerMessage,
    },
}
