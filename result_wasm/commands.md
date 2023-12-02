The following commands build and run the example.

```bash
cargo build --target wasm32-wasi --release
wasmedge --env "DATABASE_URL=postgres://postgres:postgres@localhost/postgres" target/wasm32-wasi/release/result_wasm.wasm
```
