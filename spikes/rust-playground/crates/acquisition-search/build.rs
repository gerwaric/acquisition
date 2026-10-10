//! The build a basis names (C98; B1, `search/LEDGER.md`): a digest of the
//! checked-in sources an answer is computed by, injected as
//! `ACQ_SEARCH_BUILD` (`corpus.rs`, `Basis::build`). Two builds that
//! could answer one request differently carry different names, so a
//! token is refused across them and a route read back reports both
//! counts; the table versions and `DERIVATION` beside it still say
//! which part moved, where one did.
//!
//! Inputs, whole files as the contract revision's are (C84; the routine
//! is the protocol crate's `digest.rs`, included): this crate's manifest,
//! sources and reference tables; the planner's, which resolves the price
//! a query reads; the store's, whose read fixes the order a
//! continuation's place is a place in; and the lock, for the matcher. It
//! is conservative the way the contract revision is: any edit to one of
//! them is another build, and the cost of a false one is a search asked
//! again from its start. Features, rustc, profile and target are not
//! inputs: a debug and a release build of one tree answer alike and
//! share the name.

include!("../acquisition-protocol/digest.rs");

fn main() {
    let build = digest(
        b"acq-search-build/1",
        &[
            "Cargo.lock",
            "crates/acquisition-search/Cargo.toml",
            "crates/acquisition-search/src",
            "crates/acquisition-search/reference",
            "crates/acquisition-plan/Cargo.toml",
            "crates/acquisition-plan/src",
            "crates/acquisition-plan/reference",
            "crates/acquisition-store/Cargo.toml",
            "crates/acquisition-store/src",
        ],
    );
    println!("cargo:rustc-env=ACQ_SEARCH_BUILD={build}");
}
