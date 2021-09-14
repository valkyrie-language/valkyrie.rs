cargo component build --release --target wasm32-wasip2 --package valkyrie-wit
cp target/wasm32-wasip2/release/valkyrie_wit.wasm projects/valkyrie-wasm32-wasi/valkyrie-wasm32-wasi.wasm
jco transpile projects/valkyrie-wasm32-wasi/valkyrie-wasm32-wasi.wasm -o projects/valkyrie-wasm32-wasi/src --name index
