#![no_std]
#![no_main]

use env::Wait;
use pipe_api::Fail;
use pipe_client::{Client, Direction, Port, Read, Write};

const WAIT: Wait = Wait::AtMost(5000);

#[mold::entry]
fn main() -> Result<(), env::Reason> {
    roundtrip().map_err(|_| 17)
}

fn roundtrip() -> Result<(), Fail> {
    let session =
        ipc::session::Session::open(env::unit::sire(), system_client::operator::BERTH, WAIT)
            .map_err(|_| Fail::Dead)?;
    let tree = system_client::operator::Face::of(session);
    let client = Client::find(&tree, WAIT)?;
    let pipe = client.create(16, WAIT)?;
    let me = env::unit::self_id();
    let mut reader = Port::import(client.bind(&pipe, Direction::Read, me, WAIT)?)?;
    let mut writer = Port::import(client.bind(&pipe, Direction::Write, me, WAIT)?)?;
    for payload in [b"first-pass".as_slice(), b"wrap-around".as_slice()] {
        if writer.write(payload, WAIT)? != Write::Bytes(payload.len()) {
            return Err(Fail::Protocol);
        }
        let mut bytes = [0; 16];
        if reader.read(&mut bytes, WAIT)? != Read::Bytes(payload.len())
            || &bytes[..payload.len()] != payload
        {
            return Err(Fail::Protocol);
        }
    }
    writer.close()?;
    if reader.read(&mut [0; 1], WAIT)? != Read::Eof {
        return Err(Fail::Protocol);
    }
    reader.close()?;
    drop(writer);
    drop(reader);
    client.close(&pipe, Direction::Write, WAIT)?;
    client.close(&pipe, Direction::Read, WAIT)?;
    client.release(pipe, WAIT)
}
