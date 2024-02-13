``` sh
hostname -I > cark.txt
RUST_LOG=info cargo run
CARK_SERVER_IP=$(hostname -I | awk '{print $1}') cargo run --target x86_64-pc-windows-msvc
cargo run --bin cark-client --target x86_64-pc-windows-msvc
```
