# Contributing to StrataFlow Contracts

Thank you for your interest in contributing to the **StrataFlow Soroban Smart Contracts**! This project welcomes open-source community contributions.

---

## 🛠️ Prerequisites

1. **Rust & Cargo**:
   Install the stable Rust toolchain:
   ```bash
   rustup default stable
   rustup target add wasm32-unknown-unknown
   ```
2. **Soroban CLI** (recommended for local simulation and deployment):
   ```bash
   cargo install --locked soroban-cli
   ```

---

## 🧪 Local Testing & Verification

Before submitting any pull request, ensure that all tests and linter checks pass with zero warnings:

```bash
# Run full unit and invariant test suite (37 tests)
cargo test

# Run Clippy with strict zero-warning enforcement
cargo clippy --all-targets -- -D warnings

# Build the release Wasm target
cargo rustc --target wasm32-unknown-unknown --release --crate-type cdylib
```

---

## 🛡️ Smart Contract Development Standards

- **Checks-Effects-Interactions (CEI)**: Always mutate storage and update internal balances before issuing token transfers.
- **Resource Consciousness**: Minimize memory footprints. Dynamic structures must be partitioned into `storage().persistent()` entries rather than instance storage.
- **Arithmetic Safety**: Always use checked arithmetic (`checked_add`, `checked_sub`, `checked_mul`, `checked_div`) to prevent overflows.
- **Explicit Invariant Tests**: Any new contract feature must include accompanying invariant and property tests in `src/test.rs`.

---

## 🌿 Contribution Workflow

1. Fork or branch from `main`:
   ```bash
   git checkout -b feat/your-contract-improvement
   ```
2. Follow Conventional Commits:
   - `feat(contract): ...`
   - `fix(security): ...`
   - `test(invariant): ...`
   - `docs(readme): ...`
3. Ensure no local snapshot files with line-ending churn are committed.
4. Open a pull request with an explanation of your modifications.

---

## 📜 Code of Conduct

All contributors are expected to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).
