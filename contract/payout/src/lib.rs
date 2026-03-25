#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const SCHEMA_VERSION_KEY: Symbol = symbol_short!("S_VER");
const TOPIC_PAYOUT_EXECUTED: Symbol = symbol_short!("PAYOUT");

/// Current schema version. Bump this when storage layout changes.
const CURRENT_SCHEMA_VERSION: u32 = 1;

// ── Error codes ───────────────────────────────────────────────────────────────
//
// All public write entrypoints return `Result<_, PayoutError>` so callers
// receive a machine-readable error code instead of an opaque panic string.

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PayoutError {
    /// Contract was already initialised; `initialize` may only be called once.
    AlreadyInitialized = 1,
    /// Contract has not been initialised yet.
    NotInitialized = 2,
    /// Caller is not the admin and lacks permission for this operation.
    Unauthorized = 3,
    /// Payout amount is zero or negative.
    InvalidAmount = 4,
    /// A payout for this `(idempotency_key, winner)` pair was already processed.
    AlreadyProcessed = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Payout(u32, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Unauthorized = 1,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PayoutData {
    pub winner: Address,
    pub amount: i128,
    pub currency: Symbol,
    pub paid: bool,
}

#[contract]
pub struct PayoutContract;

#[contractimpl]
impl PayoutContract {
    /// Placeholder function — returns a fixed value for contract liveness checks.
    ///
    /// # Authorization
    /// None — open to any caller.
    pub fn hello(_env: Env) -> u32 {
        789
    }

    /// Initialise the payout contract, setting the admin address.
    /// Must be called exactly once after deployment.
    ///
    /// # Errors
    /// * [`PayoutError::AlreadyInitialized`] — contract has already been initialised.
    ///
    /// # Authorization
    /// None — permissionless; must be called immediately after deploy.
    pub fn initialize(env: Env, admin: Address) -> Result<(), PayoutError> {
        if env.storage().instance().has(&ADMIN_KEY) {
            return Err(PayoutError::AlreadyInitialized);
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage()
            .instance()
            .set(&SCHEMA_VERSION_KEY, &CURRENT_SCHEMA_VERSION);
        Ok(())
    }

    // ── Schema versioning ────────────────────────────────────────────────────

    /// Return the persisted schema version (0 if never set).
    pub fn schema_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&SCHEMA_VERSION_KEY)
            .unwrap_or(0u32)
    }

    /// Migrate storage from the current persisted version to
    /// `CURRENT_SCHEMA_VERSION`. Admin-only.
    ///
    /// Each version bump should have its own migration block inside
    /// this function. The version is written atomically at the end so
    /// a failed transaction leaves the old version in place.
    ///
    /// Calling `migrate` when already at the current version is a no-op.
    pub fn migrate(env: Env) -> Result<(), PayoutError> {
        let admin = require_admin(&env)?;
        admin.require_auth();

        let stored: u32 = env
            .storage()
            .instance()
            .get(&SCHEMA_VERSION_KEY)
            .unwrap_or(0u32);

        if stored >= CURRENT_SCHEMA_VERSION {
            return Ok(()); // already up to date
        }

        // -- v0 -> v1: initial version stamp (no data changes) ------
        // Future migrations go here as sequential if-blocks:
        //   if stored < 2 { /* v1 -> v2 migration logic */ }

        env.storage()
            .instance()
            .set(&SCHEMA_VERSION_KEY, &CURRENT_SCHEMA_VERSION);
        Ok(())
    }

    /// Return the current admin address.
    ///
    /// # Errors
    /// * [`PayoutError::NotInitialized`] — `initialize` has not been called.
    ///
    /// # Authorization
    /// None — read-only, open to any caller.
    pub fn admin(env: Env) -> Result<Address, PayoutError> {
        require_admin(&env)
    }

    /// Distribute winnings to a winner. Admin-only.
    ///
    /// Uses an `idempotency_key` to prevent double-pays: calling this function
    /// twice with the same `(idempotency_key, winner)` pair is rejected.
    ///
    /// # Arguments
    /// * `caller` - Must be the admin address.
    /// * `idempotency_key` - Unique key scoped to `winner` preventing duplicate payouts.
    /// * `winner` - Recipient address.
    /// * `amount` - Amount to pay; must be > 0.
    /// * `currency` - Currency symbol (e.g. `XLM`, `USDC`).
    ///
    /// # Errors
    /// * [`PayoutError::NotInitialized`] — contract not initialised.
    /// * [`PayoutError::Unauthorized`] — `caller` is not the admin.
    /// * [`PayoutError::InvalidAmount`] — `amount` is zero or negative.
    /// * [`PayoutError::AlreadyProcessed`] — payout already recorded for this key.
    ///
    /// # Events
    /// Emits `PayoutExecuted(winner, amount, currency)`.
    pub fn distribute_winnings(
        env: Env,
        caller: Address,
        idempotency_key: u32,
        winner: Address,
        amount: i128,
        currency: Symbol,
    ) -> Result<(), PayoutError> {
        let admin = require_admin(&env)?;

        if caller != admin {
            return Err(PayoutError::Unauthorized);
        }

        if amount <= 0 {
            return Err(PayoutError::InvalidAmount);
        }

        let payout_key = DataKey::Payout(idempotency_key, winner.clone());
        if env
            .storage()
            .instance()
            .get::<_, PayoutData>(&payout_key)
            .is_some()
        {
            return Err(PayoutError::AlreadyProcessed);
        }

        let payout_data = PayoutData {
            winner: winner.clone(),
            amount,
            currency: currency.clone(),
            paid: true,
        };
        env.storage().instance().set(&payout_key, &payout_data);

        env.events()
            .publish((TOPIC_PAYOUT_EXECUTED,), (winner, amount, currency));

        Ok(())
    }

    /// Return whether a payout for the given key has been processed.
    ///
    /// # Authorization
    /// None — read-only, open to any caller.
    pub fn is_payout_processed(env: Env, idempotency_key: u32, winner: Address) -> bool {
        let payout_key = DataKey::Payout(idempotency_key, winner);
        env.storage()
            .instance()
            .get::<_, PayoutData>(&payout_key)
            .map(|p| p.paid)
            .unwrap_or(false)
    }

    /// Return the stored payout data, or `None` if not yet processed.
    ///
    /// # Authorization
    /// None — read-only, open to any caller.
    pub fn get_payout(env: Env, idempotency_key: u32, winner: Address) -> Option<PayoutData> {
        let payout_key = DataKey::Payout(idempotency_key, winner);
        env.storage().instance().get(&payout_key)
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Return the stored admin address, or `PayoutError::NotInitialized` if absent.
fn require_admin(env: &Env) -> Result<Address, PayoutError> {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .ok_or(PayoutError::NotInitialized)
}

#[cfg(test)]
mod test;
