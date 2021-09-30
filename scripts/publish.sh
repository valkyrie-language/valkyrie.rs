# shellcheck disable=SC2164
# shellcheck disable=SC2103
cd projects || cd ../projects

cd valkyrie-types
cargo publish
cd ..

cd valkyrie-compiler
cargo publish
cd ..

cd valkyrie-interpreter
cargo publish
cd ..

cd valkyrie
cargo publish
cd ..
