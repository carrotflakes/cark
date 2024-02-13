``` sh
CARK_SERVER_IP=$(hostname -I | awk '{print $1}') cargo run --target x86_64-pc-windows-msvc
```
