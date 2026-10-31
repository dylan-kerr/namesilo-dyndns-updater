# Namesilo Dynamic DNS Updater

## Development setup

Install dependencies:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash
rustup default nightly
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall cargo-llvm-cov
cargo binstall watchexec
cargo binstall cargo-mutants
```

Copy `.cargo/config.example.toml` to `.cargo/config.toml` and customise the environment variables.

Watch for file changes and run a lint-test-build-run loop:

```bash
./dev.sh
```

Run mutation tests:

```bash
cargo mutants
```

Build for release:

```bash
cargo build --release
```
