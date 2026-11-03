#![no_std]
//! Pipe service: the ledger, live roots and the resident request loop.

extern crate alloc;

pub mod ledger;
pub mod record;

mod ended;
mod grant;
mod lease;
mod process;
mod root;
mod run;

pub use run::run;
