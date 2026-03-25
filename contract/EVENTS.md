# Inverse Arena — contract events (indexer reference)

Soroban events are published as `(topics, data)` tuples. Topics are short [`Symbol`](https://developers.stellar.org/docs/build/smart-contracts/getting-started) values (see `symbol_short!` in `contract/arena/src/lib.rs`).

## Operational events (arena)

### Upgrade lifecycle

| Topic symbol | When | Data (examples) |
|--------------|------|-------------------|
| `UP_PROP` | `propose_upgrade` | `(wasm_hash: BytesN<32>, execute_after: u64)` |
| `UP_EXEC` | `execute_upgrade` | `wasm_hash: BytesN<32>` |
| `UP_CANC` | `cancel_upgrade` | `()` |

### Pause policy

| Topic | When | Data |
|-------|------|------|
| `PAUSED` | `pause` | `paused: bool` (`true`) |
| `UNPAUSED` | `unpause` | `paused: bool` (`false`) |

### Rounds and outcomes

| Topic | When | Data (machine-readable) |
|-------|------|-------------------------|
| `R_START` | `start_round` | `(round_number: u32, round_start_ledger: u32, round_deadline_ledger: u32)` |
| `R_TOUT` | `timeout_round` | `(round_number: u32, total_submissions: u32, timed_out: bool)` |
| `WIN_SET` | `set_winner` (admin) | `(player: Address, stake: i128, yield_comp: i128)` |
| `CLAIM` | successful `claim` | `(player: Address, total_payout: i128, round_number: u32)` |
| `G_END` | successful `claim` (game concluded) | `(round_number: u32)` |

### Example: indexer pseudo-code

```text
on topics[0] == "R_START":
  round_id = data[0]
  deadline_ledger = data[2]

on topics[0] == "CLAIM":
  winner = data[0]
  amount = data[1]
```

> **Note:** Topic strings are limited length on-chain; symbols in code use short forms (`R_START`, `R_TOUT`, …).

## Versioning

Adding, removing, or re-shaping event payloads is a **breaking change** for indexers. Bump `schema_version` in `contract/arena/abi_snapshot.json` and document the migration in release notes.
