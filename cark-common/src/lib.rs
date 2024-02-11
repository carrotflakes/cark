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
pub struct Join {
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Joined {
    pub field: Field,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PublicChatMessage {
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ClientMessage {
    Join(Join),
    PublicChatMessage(PublicChatMessage),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ServerMessage {
    Joined(Joined),
}

pub use postcard::to_io;

pub fn read<T: serde::de::DeserializeOwned>(buf: &mut Vec<u8>) -> Result<T, postcard::Error> {
    let (x, rest) = postcard::take_from_bytes(buf)?;
    let len = buf.len() - rest.len();
    buf.drain(..len);
    Ok(x)
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

#[test]
fn test() {
    let mut buf = postcard::to_allocvec("hello").unwrap();
    buf.extend(&[0, 1, 2, 3]);
    let s: (String, &[u8]) = postcard::take_from_bytes(&buf).unwrap();
    dbg!(s);
    dbg!(&buf);

    let x = read::<String>(&mut buf).unwrap();
    dbg!(x);
    dbg!(buf);
}
