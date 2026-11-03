#![no_std]
#![no_main]
//! One declared ELF, two declared UnitFiles: role 0 is the parent probe, role 1
//! is the declared peer. Both are launched by static activation hooks and have
//! their own operator session; they are not runtime threads.

extern crate alloc;

mod parent;
mod peer;
mod wire;

use execution::boot::Report;

#[mold::entry]
fn main() -> Report<'static> {
    let role = execution::boot::args::args().first().copied().unwrap_or(0);
    match role {
        0 => parent::run(),
        1 => peer::run(),
        _ => {
            system_support::debug::put("PIPE: FAIL role");
            Report::note(1, "pipe-probe: failed")
        }
    }
}
