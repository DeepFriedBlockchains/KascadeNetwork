# Building KASCADE from source

Install Rust 1.91.0 or newer, Git, a C/C++ build toolchain, CMake, Protobuf
compiler and libclang. `Cargo.toml` records the minimum Rust version.
The commands below target Ubuntu 24.04.

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev protobuf-compiler clang libclang-dev cmake
git clone https://github.com/DeepFriedBlockchains/KascadeNetwork.git
cd KascadeNetwork
cargo build --locked --release -p kaspad -p miner -p zkas-walletd -p zkas-api
```

Built binaries are in `target/release/`. The first build compiles RocksDB and
the shielded dependencies and can take substantial time and disk space.
Limit Cargo jobs on machines with restricted memory.

## Tests and checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --tests --benches --examples -- -D warnings
cargo test --locked -p kaspa-consensus --lib
```

GitHub Actions also runs workspace tests, WASM checks and release builds.
Use the targeted command appropriate to the affected crate or test when
developing a fix. Include actual outcomes in the pull request.

The software is pre-release and network configuration remains unfinished.
Use isolated local development data directories.
