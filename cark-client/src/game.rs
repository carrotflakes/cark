pub struct Game {
    field: Field,
    pub characters: Vec<Character>,
    pub player_id: u64,
    pub ups: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            field: Field::new(20, 20),
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
}

#[derive(Clone)]
pub struct Field {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Field {
    pub fn new(width: u32, height: u32) -> Self {
        let mut data = vec![0; (width * height) as usize];
        for i in 0..width {
            data[i as usize] = 1;
            data[(height * width - i - 1) as usize] = 1;
        }
        for i in 0..height {
            data[(i * width) as usize] = 1;
            data[((i + 1) * width - 1) as usize] = 1;
        }
        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_data(width: u32, height: u32, data: Vec<u8>) -> Self {
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

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

pub struct Character {
    id: u64,
    name: String,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

impl Character {
    pub fn new(id: u64, name: String, position: [f32; 2]) -> Self {
        Self {
            id,
            name,
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
