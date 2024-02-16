``` sh
RUST_LOG=info cargo run --bin cark-server

echo -e "server_tcp_addr = \"$(hostname -I | awk '{print $1}'):8080\"\nserver_udp_addr = \"$(hostname -I | awk '{print $1}'):8081\"" > cark.toml
cargo run --bin cark-window --target x86_64-pc-windows-msvc
```
