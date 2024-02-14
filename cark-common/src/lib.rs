pub mod model;
pub mod udp_stat;

pub use postcard::to_io as write;
pub use postcard::to_slice as write_to_slice;

pub fn read<T: serde::de::DeserializeOwned>(buf: &mut Vec<u8>) -> Result<T, postcard::Error> {
    let (x, rest) = postcard::take_from_bytes(buf)?;
    let len = buf.len() - rest.len();
    buf.drain(..len);
    Ok(x)
}

pub fn read_from_slice<T: serde::de::DeserializeOwned>(buf: &[u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(buf)
}

pub type PostcardError = postcard::Error;

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
