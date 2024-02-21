use std::{collections::HashMap, num::NonZeroU32};

pub type ChunkId = NonZeroU32;
pub type OptChunkId = u32;

pub const CHUNK_SIZE: usize = 16;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Chunk {
    pub id: ChunkId,
    // Related chunk ids: [left, right, top, bottom]
    pub related: [OptChunkId; 4],
    #[serde(with = "serde_big_array::BigArray")]
    pub data: [u8; CHUNK_SIZE * CHUNK_SIZE],
}

impl Chunk {
    pub fn new(id: ChunkId) -> Self {
        use rand::Rng;
        let mut data = [1; CHUNK_SIZE * CHUNK_SIZE];
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(id.get() as u64);
        data[0] = 3;
        for i in 0..8 {
            data[i + 1] = if id.get() >> i & 1 == 1 { 4 } else { 2 };
        }
        for i in 9..data.len() {
            data[i] = if rng.gen_bool(0.025) {
                1
            } else {
                if rng.gen_bool(0.2) {
                    3
                } else {
                    2
                }
            };
        }
        Self {
            id,
            related: [0, 0, 0, 0],
            data,
        }
    }
}

pub struct Field {
    pub new_id: ChunkId,
    pub chunks: HashMap<ChunkId, Chunk>,
}

impl Field {
    pub fn new() -> Self {
        let id = ChunkId::MIN;
        Self {
            new_id: id.checked_add(1).unwrap(),
            chunks: [(id, Chunk::new(id))].into_iter().collect(),
        }
    }

    pub fn chunk(&self, id: ChunkId) -> Option<&Chunk> {
        self.chunks.get(&id)
    }

    pub fn chunks_around(&self, id: ChunkId) -> [([i32; 2], Option<&Chunk>); 9] {
        let mut chunks = [
            ([-1, -1], None),
            ([0, -1], None),
            ([1, -1], None),
            ([-1, 0], None),
            ([0, 0], self.chunk(id)),
            ([1, 0], None),
            ([-1, 1], None),
            ([0, 1], None),
            ([1, 1], None),
        ];
        chunks[1].1 = chunks[4].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Top.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[3].1 = chunks[4].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Left.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[5].1 = chunks[4].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Right.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[7].1 = chunks[4].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Bottom.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[0].1 = chunks[1].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Left.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[2].1 = chunks[1].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Right.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[6].1 = chunks[7].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Left.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks[8].1 = chunks[7].1.and_then(|c| {
            ChunkId::new(c.related[Direction::Right.to_number()]).and_then(|id| self.chunk(id))
        });
        chunks
    }

    // Generate a new chunk next to the given chunk
    pub fn generate_chunk(&mut self, id: ChunkId, direction: Direction) -> Option<ChunkId> {
        if self.chunk(id).unwrap().related[direction.to_number()] != 0 {
            // Chunk already exists
            return None;
        }

        let new_id = self.new_id;
        self.new_id = new_id.checked_add(1).unwrap();
        let mut new_chunk = Chunk::new(new_id);
        new_chunk.related[direction.opposite().to_number()] = id.get();

        self.chunks.insert(new_id, new_chunk);

        self.compute_related_chunks(new_id);

        Some(new_id)
    }

    fn compute_related_chunks(&mut self, new_id: ChunkId) {
        let mut open = vec![(new_id, [0, 0])];
        let mut closed = vec![new_id];
        while let Some((id, pos)) = open.pop() {
            if pos == [-1, 0] {
                if let Some(chunk) = self.chunks.get_mut(&new_id) {
                    chunk.related[Direction::Left.to_number()] = id.get();
                }
                if let Some(chunk) = self.chunks.get_mut(&id) {
                    chunk.related[Direction::Right.to_number()] = new_id.get();
                }
            }
            if pos == [1, 0] {
                if let Some(chunk) = self.chunks.get_mut(&new_id) {
                    chunk.related[Direction::Right.to_number()] = id.get();
                }
                if let Some(chunk) = self.chunks.get_mut(&id) {
                    chunk.related[Direction::Left.to_number()] = new_id.get();
                }
            }
            if pos == [0, -1] {
                if let Some(chunk) = self.chunks.get_mut(&new_id) {
                    chunk.related[Direction::Top.to_number()] = id.get();
                }
                if let Some(chunk) = self.chunks.get_mut(&id) {
                    chunk.related[Direction::Bottom.to_number()] = new_id.get();
                }
            }
            if pos == [0, 1] {
                if let Some(chunk) = self.chunks.get_mut(&new_id) {
                    chunk.related[Direction::Bottom.to_number()] = id.get();
                }
                if let Some(chunk) = self.chunks.get_mut(&id) {
                    chunk.related[Direction::Top.to_number()] = new_id.get();
                }
            }
            if let Some(chunk) = self.chunk(id) {
                for (i, &related_id) in chunk.related.iter().enumerate() {
                    if let Some(chunk_id) = ChunkId::new(related_id) {
                        if closed.contains(&chunk_id) {
                            continue;
                        }
                        open.push((chunk_id, Direction::from_number(i).move_pos(pos)));
                        closed.push(chunk_id);
                    }
                }
            }
        }
    }

    pub fn set_existed_chunk(&mut self, chunk: Chunk, compute_related: bool) {
        let id = chunk.id;
        self.chunks.insert(chunk.id, chunk);

        if compute_related {
            self.compute_related_chunks(id);
        }
    }

    pub fn view(&self, chunk_id: ChunkId, rect: [i32; 4]) -> Vec<u8> {
        let mut view = vec![0; (rect[2] - rect[0]) as usize * (rect[3] - rect[1]) as usize];
        for cy in rect[1].div_euclid(CHUNK_SIZE as i32)..=rect[3].div_euclid(CHUNK_SIZE as i32) {
            let mut chunk_id = Some(chunk_id);
            if cy < 0 {
                for _ in 0..-cy {
                    chunk_id = chunk_id.and_then(|id| self.chunk(id)).and_then(|chunk| {
                        NonZeroU32::new(chunk.related[Direction::Top.to_number()])
                    });
                }
            } else {
                for _ in 0..cy {
                    chunk_id = chunk_id.and_then(|id| self.chunk(id)).and_then(|chunk| {
                        NonZeroU32::new(chunk.related[Direction::Bottom.to_number()])
                    });
                }
            }
            for cx in rect[0].div_euclid(CHUNK_SIZE as i32)..=rect[2].div_euclid(CHUNK_SIZE as i32)
            {
                let mut chunk_id = chunk_id;
                if cx < 0 {
                    for _ in 0..-cx {
                        chunk_id = chunk_id.and_then(|id| self.chunk(id)).and_then(|chunk| {
                            NonZeroU32::new(chunk.related[Direction::Left.to_number()])
                        });
                    }
                } else {
                    for _ in 0..cx {
                        chunk_id = chunk_id.and_then(|id| self.chunk(id)).and_then(|chunk| {
                            NonZeroU32::new(chunk.related[Direction::Right.to_number()])
                        });
                    }
                }
                if let Some(chunk) = chunk_id.and_then(|id| self.chunk(id)) {
                    for dy in 0..CHUNK_SIZE as i32 {
                        for dx in 0..CHUNK_SIZE as i32 {
                            let x = cx * CHUNK_SIZE as i32 + dx;
                            let y = cy * CHUNK_SIZE as i32 + dy;
                            if x < rect[0] || x >= rect[2] || y < rect[1] || y >= rect[3] {
                                continue;
                            }
                            view[(y - rect[1]) as usize * (rect[2] - rect[0]) as usize
                                + (x - rect[0]) as usize] =
                                chunk.data[dy as usize * CHUNK_SIZE + dx as usize];
                        }
                    }
                }
            }
        }
        view
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }

    pub fn turn_left(&self) -> Self {
        match self {
            Self::Left => Self::Bottom,
            Self::Right => Self::Top,
            Self::Top => Self::Left,
            Self::Bottom => Self::Right,
        }
    }

    pub fn to_number(&self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Top => 2,
            Self::Bottom => 3,
        }
    }

    pub fn from_number(n: usize) -> Self {
        match n {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Top,
            3 => Self::Bottom,
            _ => panic!("Invalid number"),
        }
    }

    pub fn move_pos(&self, pos: [i32; 2]) -> [i32; 2] {
        match self {
            Self::Left => [pos[0] - 1, pos[1]],
            Self::Right => [pos[0] + 1, pos[1]],
            Self::Top => [pos[0], pos[1] - 1],
            Self::Bottom => [pos[0], pos[1] + 1],
        }
    }
}
