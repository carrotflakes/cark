
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
