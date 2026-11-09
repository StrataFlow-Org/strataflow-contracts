# StrataFlow Contracts 📜

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Soroban](https://img.shields.io/badge/Stellar-Soroban%20v21+-blueviolet.svg)](https://stellar.org)
[![CI](https://github.com/StrataFlow-Org/strataflow-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/StrataFlow-Org/strataflow-contracts/actions)

> **Part of the [StrataFlow Protocol](https://github.com/StrataFlow-Org) suite.**  
> Trustless milestone escrows and programmable settlement on the Stellar network. Built for Stellar Drips Wave.

---

## 🌟 Overview

`strataflow-contracts` contains the Soroban smart contract logic powering the StrataFlow protocol. It provides non-custodial milestone-based escrow contracts where clients lock Stellar Asset Contract (SAC) tokens (such as USDC or native XLM), service providers submit completion evidence on-chain or via IPFS hashes, and clients release payments upon milestone verification.

In the event of disagreement, a neutral third-party arbiter can resolve disputes by apportioning remaining balances on-chain.

---

## 🛡️ Security Architecture & Invariants

1. **Persistent Storage with Automatic TTL Extension**: Dynamic agreement vectors are partitioned into `storage().persistent()` entries rather than instance storage, ensuring contract size never approaches Soroban 64KB limits.
2. **Capital Trapping Remediation (`claim_timeout_refund`)**: If a designated provider fails to deliver milestones before the deadline, the client can reclaim 100% of remaining escrowed funds.
3. **Emergency Circuit Breaker (`set_pause` / `is_paused`)**: Contract administrator can pause deposits and state changes in the event of an upstream or network anomaly.
4. **Checks-Effects-Interactions (CEI)**: State mutations and balance decrements occur strictly prior to token transfer invocations.
5. **Exact Dispute Apportionment**: Arbiters must apportion the exact remaining balance (`client_amount + provider_amount == balance`), preventing residual capital freeze.
6. **Protocol Fee Ceiling**: Admin fees are capped at a strict maximum of 500 basis points (5%), protecting users against fee gouging.

---

## 🏗️ State Machine

```mermaid
stateDiagram-v2
    [*] --> Created: create_agreement()
    Created --> Funded: fund_agreement()
    Created --> Cancelled: cancel_agreement() [Pre-funding]
    Funded --> InProgress: submit_milestone()
    InProgress --> Funded: release_milestone() [Partial Approval]
    Funded --> Completed: release_milestone() [All Milestones Approved]
    Funded --> Disputed: raise_dispute()
    InProgress --> Disputed: raise_dispute()
    Disputed --> Resolved: resolve_dispute()
    Funded --> Cancelled: claim_timeout_refund() [Post-Deadline]
    Completed --> [*]
    Resolved --> [*]
    Cancelled --> [*]
```

---

## ⚡ Contract Interface

| Function | Access | Description |
| :--- | :--- | :--- |
| `initialize(admin)` | Admin | Initializes protocol admin and counters. |
| `create_agreement(...)` | Client | Creates a new escrow agreement with milestones. |
| `fund_agreement(id, funder)` | Funder / Client | Transfers required SAC tokens to contract storage. |
| `submit_milestone(id, idx, proof)`| Provider | Submits IPFS deliverable proof for verification. |
| `reject_milestone(id, idx, feedback)`| Client | Rejects submitted milestone with feedback hash. |
| `release_milestone(id, idx)` | Client | Releases milestone payout minus protocol fee. |
| `claim_timeout_refund(id)` | Client | Refunds remaining balance if deadline has passed. |
| `raise_dispute(id, idx, caller)` | Client / Provider | Freezes remaining balance into dispute status. |
| `resolve_dispute(id, client, provider)`| Arbiter | Distributes contested balance between parties minus fee. |
| `resolve_dispute_timeout(id)` | Client / Provider | Resolves stale dispute if arbiter window expired. |
| `cancel_agreement(id)` | Client | Cancels an unfunded draft agreement. |
| `mutual_cancel(id, caller)` | Client / Provider | Initiates or confirms mutual escrow cancellation and refund. |
| `reassign_arbiter(id, new_arbiter)`| Admin | Emergency reassignment if an arbiter is unresponsive. |
| `set_pause(paused)` | Admin | Triggers emergency pause circuit breaker. |
| `set_fee_config(fee_bps, recipient)` | Admin | Updates protocol fee (capped at 500 bps / 5%). |
| `nominate_admin(new_admin)` | Admin | Two-step ownership transfer nomination. |
| `accept_admin()` | Nominee | Two-step ownership transfer acceptance. |
| `get_agreement(id)` | Public Read | Queries agreement details and milestone progress. |
| `get_contract_version()` | Public Read | Returns semver tuple `(major, minor, patch)`. |
| `get_protocol_metrics()` | Public Read | Returns aggregate protocol volume and escrow counts. |

---

## 🚀 Stellar Testnet Deployment Details

The StrataFlow core protocol contract is officially deployed and live on Stellar Testnet:

| Parameter | Value |
|---|---|
| **Contract ID** | `CDXVD3AA43SE2D52ZBJSH3RDKAPOYG6OCDAZGSQD6M5TCBQIG3HTLY6J` |
| **Network** | Stellar Testnet (`Test SDF Network ; September 2015`) |
| **RPC Endpoint** | `https://soroban-testnet.stellar.org` |
| **Admin Public Key** | `GBWFZHBYEKF5MR2MHW7JAZX6ZRI2PKKBL6IALEMJSBMU4JMDSIB34KP5` |
| **WASM Hash** | `3cf260524d911a16c0012bf650f5d1453793ddf4362710d602e869956e5147fb` |
| **Contract Version** | `1.0.0` |
| **Circuit Breaker Status** | `Unpaused (Active)` |
| **Deployment Tx** | [`25bf399a...8988`](https://stellar.expert/explorer/testnet/tx/25bf399a52be5f942e9f62c965f0e8d6a0800db0673f79d779bc79f3b4378988) |
| **Initialization Tx** | [`cf2cfe25...b6ee`](https://stellar.expert/explorer/testnet/tx/cf2cfe25015e36bd7740f4fc9d13947d2425bbd83be23c88b574e7b8eae1b6ee) |
| **Stellar Expert** | [Explore Contract on StellarExpert](https://stellar.expert/explorer/testnet/contract/CDXVD3AA43SE2D52ZBJSH3RDKAPOYG6OCDAZGSQD6M5TCBQIG3HTLY6J) |
| **Stellar Lab** | [Inspect in Stellar Lab](https://lab.stellar.org/r/testnet/contract/CDXVD3AA43SE2D52ZBJSH3RDKAPOYG6OCDAZGSQD6M5TCBQIG3HTLY6J) |

---

## 🧪 Local Testing & Verification

Compile the test harness and execute unit tests (37 passing tests):

```bash
cargo test
```

Execute Clippy linter with strict zero-warning enforcement:

```bash
cargo clippy --all-targets -- -D warnings
```

Build the optimized Wasm binary:

```bash
cargo rustc --target wasm32-unknown-unknown --release --crate-type cdylib
```

---

## 📄 License

This repository is licensed under the [MIT License](LICENSE).
