cargo component build --release --target wasm32-wasip2
cp target/wasm32-wasip2/release/legion_wit.wasm projects/legion-wasm32-wasi/legion-wasm32-wasi.wasm
jco transpile projects/legion-wasm32-wasi/legion-wasm32-wasi.wasm -o projects/legion-wasm32-wasi/src --name index
