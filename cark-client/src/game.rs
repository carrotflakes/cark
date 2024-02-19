use cark_common::field::{Chunk, ChunkId, Field};

pub struct Game {
    field: Field,
    pub characters: Vec<Character>,
    pub player_id: u64,
    pub ups: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            field: Field::new(),
            characters: vec![],
            player_id: 0,
            ups: 0.0,
        }
    }

    pub fn field(&self) -> &Field {
        &self.field
    }

    pub fn set_field(&mut self, field: Field) {
        self.field = field;
    }

    pub fn update_chunk(&mut self, chunk: Chunk) {
        log::debug!("Chunk received: {:?}", chunk);
        self.field.set_existed_chunk(chunk, true);
    }
}

pub struct Character {
    id: u64,
    name: String,
    pub chunk_id: ChunkId,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

impl Character {
    pub fn new(id: u64, name: String, chunk_id: ChunkId, position: [f32; 2]) -> Self {
        Self {
            id,
            name,
            chunk_id,
            position,
            velocity: [0.0, 0.0],
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
