//! Hard caps on storage-heavy collections (participants, per-round submissions).
//!
//! Documented in `contract/BOUNDS.md`. Production limits use the `not(test)` values.
//! Unit tests compile with **lower** caps so boundary cases (N−1, N, N+1) stay fast in CI.

/// Maximum registered survivors (`DataKey::Survivor` entries + `S_COUNT`).
#[cfg(test)]
pub const MAX_ARENA_PARTICIPANTS: u32 = 64;
/// Maximum registered survivors (`DataKey::Survivor` entries + `S_COUNT`).
#[cfg(not(test))]
pub const MAX_ARENA_PARTICIPANTS: u32 = 10_000;

/// Maximum `Submission(round, player)` records for a single round (`RoundState::total_submissions`).
#[cfg(test)]
pub const MAX_SUBMISSIONS_PER_ROUND: u32 = 32;
/// Maximum `Submission(round, player)` records for a single round (`RoundState::total_submissions`).
#[cfg(not(test))]
pub const MAX_SUBMISSIONS_PER_ROUND: u32 = 10_000;
