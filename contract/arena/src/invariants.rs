//! Round state machine invariants (issue #277). Used by unit tests; keep pure (no `Env`).
//!
//! Violations should fail CI via `#[test]` callers in `test.rs`.

use crate::RoundState;

/// Flags that cannot be true together for a coherent [`RoundState`].
pub fn check_round_flags(round: &RoundState) -> Result<(), &'static str> {
    if round.active && round.timed_out {
        return Err("invariant: active and timed_out are mutually exclusive");
    }
    if round.active && round.finished {
        return Err("invariant: active and finished are mutually exclusive");
    }
    Ok(())
}

/// Round numbers are strictly positive once a round has been started (`start_round` increments from 0).
pub fn check_round_number_monotonic(prev: u32, next: u32) -> Result<(), &'static str> {
    if next != prev && next != prev + 1 {
        return Err("invariant: round_number may only stay the same or increase by 1 per transition");
    }
    Ok(())
}

/// After `timeout_round`, the round is inactive and marked timed out.
pub fn check_timeout_transition(before: &RoundState, after: &RoundState) -> Result<(), &'static str> {
    if before.round_number != after.round_number {
        return Err("invariant: timeout must not change round_number");
    }
    if after.active {
        return Err("invariant: timeout must clear active");
    }
    if !after.timed_out {
        return Err("invariant: timeout must set timed_out");
    }
    Ok(())
}

/// Submission counter must never decrease.
pub fn check_submission_count_monotonic(before: u32, after: u32) -> Result<(), &'static str> {
    if after < before {
        return Err("invariant: total_submissions must be monotonic non-decreasing");
    }
    Ok(())
}
