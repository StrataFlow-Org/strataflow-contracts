# Changelog

All notable changes to the `strataflow-contracts` repository are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] - 2026-11-09

### Security Hardening
- **Collusion Guard**: Enforced strict tripartite uniqueness (`client != provider && client != arbiter && provider != arbiter`) in `create_agreement`.
- **Dispute Fee Deduction**: Added protocol fee calculation and deduction on `resolve_dispute`, aligning fee collection with normal release workflows.
- **Anti-Freeze Fallback (`resolve_dispute_timeout`)**: Added fallback resolution mechanism triggering if an arbiter fails to act within the dispute window (default 7 days / 100,800 ledgers).
- **Two-Step Admin Transfer**: Added `nominate_admin` and `accept_admin` functions to eliminate single-step admin transfer risks.
- **Bilateral Cancellation (`mutual_cancel`)**: Added consensus cancellation allowing client and provider to agree on mutual refund prior to milestone completion.
- **Milestone Rejection Flow (`reject_milestone`)**: Enabled clients to reject unsatisfactory deliverables with on-chain feedback hashes without raising a full dispute.

### Code Quality & Static Analysis
- **Zero-Warning Clippy Enforcement**: Refactored state transition matching to `matches!` macro and cleaned up boolean assertions in test suites, passing `cargo clippy --all-targets -- -D warnings` with zero warnings.
- **Test Suite Expansion**: Added property-based invariant tests covering escrow solvency, conservation of funds, admin nomination resets, and dispute conservation, reaching **37 passing unit tests (100% pass rate)**.
- **Governance Documentation**: Added `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, and updated `README.md`.

---

## [1.0.0] - 2026-11-08

### Added
- **Initial Protocol Contract**: Milestone-based programmable escrow contract deployed on Stellar Soroban Testnet at `CDXVD3AA43SE2D52ZBJSH3RDKAPOYG6OCDAZGSQD6M5TCBQIG3HTLY6J`.
- **Core Operations**: Implemented `initialize`, `create_agreement`, `fund_agreement`, `submit_milestone`, `release_milestone`, `claim_timeout_refund`, `raise_dispute`, `resolve_dispute`, `cancel_agreement`.
- **Emergency Circuit Breaker**: Added `set_pause` and `is_paused` to safeguard against unexpected upstream anomalies.
- **Fee Management**: Configurable protocol fee basis points (capped at 500 bps / 5%).
- **Storage Strategy**: Partitioned agreement state into `storage().persistent()` entries with automatic TTL extension on state mutation.
