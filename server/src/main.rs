#![no_std]
#![no_main]

#[mold::entry]
fn main() -> Result<(), env::Reason> {
    pipe_server::run().map_err(|message| {
        system_support::debug::put(message);
        85
    })
}
