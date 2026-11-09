# Security Policy

## 🔒 Reporting Security Vulnerabilities

The StrataFlow team takes smart contract security, cryptographic validation, and non-custodial asset custody seriously. If you discover a vulnerability or security flaw in the smart contracts:

**Please DO NOT open a public issue.**

Instead, report it responsibly to:
- **Email**: `security@strataflow.io`
- **Subject**: `[Smart Contract Vulnerability]: StrataFlow Contracts`

### Please Include in Your Report:
1. **Proof of Concept**: Minimal test case reproducing the vulnerability using `soroban_sdk::Env`.
2. **Impact Assessment**: Evaluation of potential impact (e.g. fund drain, state freezing, unauthorized release, replay attack).
3. **Mitigation**: Any suggested algorithmic or architectural fixes.

### Response SLA
- **Initial Acknowledgment**: Within 24-48 hours.
- **Triage & Remediation Timeline**: Within 3-5 business days for critical issues.
- **Public Disclosure**: Coordinated following patch deployment.

---

## 🛡️ Built-in Protocol Safeguards

1. **Exact Capital Conservation**: Invariant verification ensures `total_released + remaining_balance == total_funded` across all states.
2. **Two-Step Admin Transfer**: Eliminates single-step administrative key compromise.
3. **Emergency Circuit Breaker**: Allows contract administrators to temporarily halt deposits during upstream network emergencies without locking user withdrawals.
4. **Dispute Window Timeout**: Prevents permanent fund trapping in the event of an unresponsive arbiter.
