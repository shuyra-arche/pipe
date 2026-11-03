#![no_std]
//! Pipe wire vocabulary: marks, opcodes, request/reply frames and failure codes.

pub use stream::{DEFAULT_CAPACITY, Direction, HEADER_SIZE, Header, MAX_CAPACITY, Read, Write};

pub use bit::{DEMAND_BIT, READ_BIT, WRITE_BIT};
pub use call::Call;
pub use direction::{code, direction};
pub use endpoint::Endpoint;
pub use fail::Fail;
pub use mark::{BACK, DATA, DIR, ENTRY, LEASE};
pub use op::{BIND, CLOSE, CREATE, READ, RELEASE, WRITE};
pub use reply::Reply;
pub use request::Request;

mod bit;
mod call;
mod direction;
mod endpoint;
mod fail;
mod mark;
mod op;
mod reply;
mod request;
