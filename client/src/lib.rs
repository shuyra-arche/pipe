#![no_std]
//! Pipe client: pipe creation/binding calls and imported byte ports.

pub use pipe_api::{Direction, Read, Write};

pub use client::Client;
pub use creating::Creating;
pub use pipe::Pipe;
pub use port::Port;

mod client;
mod creating;
mod pipe;
mod port;
