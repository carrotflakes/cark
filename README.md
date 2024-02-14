``` sh
RUST_LOG=info cargo run --bin cark-server

hostname -I > cark.txt
cargo run --bin cark-client --target x86_64-pc-windows-msvc
```
