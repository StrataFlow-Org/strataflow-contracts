# Contributing to StrataFlow Contracts

Thank you for your interest in contributing to the StrataFlow Soroban smart contract repository! This project participates in the **Stellar Drips Wave** and welcomes open-source developers of all skill levels.

---

## 🛠️ Prerequisites

1. **Rust & Cargo**:
   Ensure you have Rust 1.79+ or newer installed:
   ```bash
   rustup default stable
   rustup target add wasm32-unknown-unknown
   ```
2. **Soroban CLI** (optional, recommended for local deployment):
   ```bash
   cargo install --locked soroban-cli
   ```

---

## 🚀 Development Workflow

1. **Fork & Clone**:
   ```bash
   git clone https://github.com/winningtalker-commits/strataflow-contracts.git
   cd strataflow-contracts
   ```
2. **Create a Feature Branch**:
   ```bash
   git checkout -b feat/my-new-feature
   ```
3. **Format & Lint**:
   Always run formatting and clippy before committing:
   ```bash
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   ```
4. **Run Tests**:
   ```bash
   cargo test
   ```
5. **Build Wasm Target**:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

---

## 🏷️ Issue & PR Taxonomy for Drips Wave

When claiming tasks during Drips Wave sprints:
- **Low / Trivial**: Error handling improvements, code comments, localized unit tests.
- **Medium**: New contract functions, event index improvements, gas optimization.
- **High**: Architecture extensions, multi-sig upgrades, security hardening.

Please reference the issue number in your PR description: e.g. `Fixes #12`.

