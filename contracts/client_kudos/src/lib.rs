#![no_std]

//! # client_kudos
//!
//! A Soroban smart contract that lets a client award a public, on-chain
//! kudos record to a freelancer after a project is delivered. Each kudos
//! carries a 1..=5 star rating, an off-chain comment hash, and a project
//! reference. The freelancer accumulates an immutable reputation history
//! that anyone can verify — no XLM transfer is involved.

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol, Vec};

/// Maximum allowed star rating.
const MAX_RATING: u32 = 5;

/// Minimum allowed star rating.
const MIN_RATING: u32 = 1;

/// A single kudos record: one client acknowledging one project from one
/// freelancer. Records are append-only and may be soft-revoked by the
/// original client (in which case `active` becomes false).
#[contracttype]
#[derive(Clone, Debug)]
pub struct KudosRecord {
    /// Address of the client who issued this kudos.
    pub client: Address,
    /// Short symbolic reference to the project (e.g. "proj-2025-014").
    pub project_ref: Symbol,
    /// Star rating in the inclusive range `[MIN_RATING, MAX_RATING]`.
    pub rating: u32,
    /// SHA-256 (or similar 32-byte) hash of the off-chain comment text.
    pub comment_hash: BytesN<32>,
    /// Ledger timestamp at the moment the kudos was issued.
    pub timestamp: u64,
    /// True while the record contributes to the freelancer's score.
    /// Set to `false` after a revocation by the original client.
    pub active: bool,
    /// Reason for revocation, or `"active"` while the record stands.
    pub revoke_reason: Symbol,
}

/// Storage keys used by the contract. Grouping all keys in one enum
/// makes future migrations and storage inspection easier.
#[contracttype]
pub enum DataKey {
    /// Full append-only history of kudos for a freelancer.
    Kudos(Address),
    /// Running sum of *active* ratings for a freelancer.
    Score(Address),
    /// Total number of kudos ever issued (active + revoked).
    Count(Address),
}

#[contract]
pub struct ClientKudos;

#[contractimpl]
impl ClientKudos {
    /// Client awards a kudos record to a freelancer for a finished project.
    ///
    /// The client must authorize the call. The rating must be in
    /// `1..=5`. The new record is appended to the freelancer's history
    /// and the running reputation score is updated. Returns the new
    /// total number of kudos the freelancer has received.
    pub fn give_kudos(
        env: Env,
        client: Address,
        freelancer: Address,
        project_ref: Symbol,
        rating: u32,
        comment_hash: BytesN<32>,
    ) -> u32 {
        // The client must sign for this call.
        client.require_auth();

        // Sanity checks: rating range and self-kudos prevention.
        if rating < MIN_RATING || rating > MAX_RATING {
            panic!("rating must be between 1 and 5");
        }
        if client == freelancer {
            panic!("client cannot give kudos to themselves");
        }

        // Build the new record.
        let record = KudosRecord {
            client: client.clone(),
            project_ref,
            rating,
            comment_hash,
            timestamp: env.ledger().timestamp(),
            active: true,
            revoke_reason: Symbol::new(&env, "active"),
        };

        // Append to the freelancer's history.
        let history_key = DataKey::Kudos(freelancer.clone());
        let mut history: Vec<KudosRecord> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or(Vec::new(&env));
        history.push_back(record);
        env.storage().persistent().set(&history_key, &history);

        // Update running total score (sum of active ratings).
        let score_key = DataKey::Score(freelancer.clone());
        let current_score: u32 = env
            .storage()
            .persistent()
            .get(&score_key)
            .unwrap_or(0u32);
        let new_score = current_score + rating;
        env.storage().persistent().set(&score_key, &new_score);

        // Update total kudos count (active + revoked).
        let count_key = DataKey::Count(freelancer);
        let current_count: u32 = env
            .storage()
            .persistent()
            .get(&count_key)
            .unwrap_or(0u32);
        let new_count = current_count + 1;
        env.storage().persistent().set(&count_key, &new_count);

        new_count
    }

    /// Client revokes a previously issued kudos. Only the original client
    /// can revoke, and they must sign the call. The record at `index`
    /// is marked inactive and its rating is subtracted from the
    /// freelancer's running score. The count is left unchanged so the
    /// history remains intact.
    pub fn revoke_kudos(
        env: Env,
        client: Address,
        freelancer: Address,
        index: u32,
        reason: Symbol,
    ) {
        client.require_auth();

        let history_key = DataKey::Kudos(freelancer.clone());
        let mut history: Vec<KudosRecord> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or_else(|| panic!("no kudos history for this freelancer"));

        if index >= history.len() {
            panic!("kudos index out of bounds");
        }

        let mut record = history.get(index).unwrap();
        if record.client != client {
            panic!("only the original client can revoke this kudos");
        }
        if !record.active {
            panic!("kudos is already revoked");
        }

        // Soft-revoke: keep the record in history but mark it inactive.
        record.active = false;
        record.revoke_reason = reason;
        let revoked_rating = record.rating;
        history.set(index, record);
        env.storage().persistent().set(&history_key, &history);

        // Subtract the rating from the running score (saturating to
        // guard against any storage drift).
        let score_key = DataKey::Score(freelancer);
        let current_score: u32 = env
            .storage()
            .persistent()
            .get(&score_key)
            .unwrap_or(0u32);
        let new_score = current_score.saturating_sub(revoked_rating);
        env.storage().persistent().set(&score_key, &new_score);
    }

    /// Returns the total accumulated kudos score (sum of *active* ratings)
    /// for the given freelancer. Zero if the freelancer has no kudos.
    pub fn get_score(env: Env, freelancer: Address) -> u32 {
        let score_key = DataKey::Score(freelancer);
        env.storage()
            .persistent()
            .get(&score_key)
            .unwrap_or(0u32)
    }

    /// Returns the total number of kudos records ever issued to the
    /// freelancer, including revoked ones.
    pub fn get_kudos_count(env: Env, freelancer: Address) -> u32 {
        let count_key = DataKey::Count(freelancer);
        env.storage()
            .persistent()
            .get(&count_key)
            .unwrap_or(0u32)
    }

    /// Returns the average star rating for the freelancer multiplied by
    /// `100` so that two decimal places of precision are preserved as a
    /// `u32` (e.g. `432` means 4.32 stars). Returns `0` when the
    /// freelancer has no recorded kudos.
    pub fn get_average_rating(env: Env, freelancer: Address) -> u32 {
        let count = Self::get_kudos_count(env.clone(), freelancer.clone());
        if count == 0 {
            return 0;
        }
        let score = Self::get_score(env, freelancer);
        // (score / count) * 100 with two-decimal precision
        (score * 100) / count
    }

    /// Returns `true` if the kudos record at position `index` for
    /// `freelancer` was originally issued by `client`. Lets a third
    /// party off-chain verifier confirm the authenticity of a citation.
    pub fn is_authentic(
        env: Env,
        freelancer: Address,
        index: u32,
        client: Address,
    ) -> bool {
        let history_key = DataKey::Kudos(freelancer);
        let history: Vec<KudosRecord> = match env.storage().persistent().get(&history_key) {
            Some(h) => h,
            None => return false,
        };
        if index >= history.len() {
            return false;
        }
        history.get(index).unwrap().client == client
    }
}
