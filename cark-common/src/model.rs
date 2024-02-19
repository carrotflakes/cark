use crate::udp_stat::Sequence;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Message {
    pub id: u64,
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Field {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Character {
    pub id: u64,
    pub name: String,
    pub position: [f32; 2],
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Join {
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Joined {
    pub user_id: u64,
    pub field: Field,
    pub characters: Vec<JoinedCharacter>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct JoinedCharacter {
    pub id: u64,
    pub name: String,
    pub position: [f32; 2],
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct UpdateField {
    pub position: [u32; 2],
    pub value: u8,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PublicChatMessage {
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Join(Join),
    PublicChatMessage(PublicChatMessage),
    UpdateField(UpdateField),
    Position {
        position: [f32; 2],
        velocity: [f32; 2],
    },
    Leave,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ServerMessage {
    Joined(Joined),
    UpdateField(UpdateField),
    PlayerJoined {
        id: u64,
        name: String,
        position: [f32; 2],
    },
    PlayerLeft {
        user_id: u64,
    },
    Position {
        user_id: u64,
        position: [f32; 2],
        velocity: [f32; 2],
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

impl Field {
    pub fn new(width: u32, height: u32) -> Self {
        let mut data = vec![0; (width * height) as usize];
        let wall = 2;
        for i in 0..width {
            data[i as usize] = wall;
            data[(height * width - i - 1) as usize] = wall;
        }
        for i in 0..height {
            data[(i * width) as usize] = wall;
            data[((i + 1) * width - 1) as usize] = wall;
        }
        Self {
            width,
            height,
            data,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
