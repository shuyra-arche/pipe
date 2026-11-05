#![no_std]
#![no_main]

use env::Wait;
use pipe_api::Fail;
use pipe_client::{Client, Direction, Port, Read, Write};
mod step;
use step::Step;

const WAIT: Wait = Wait::AtMost(5000);

#[mold::entry]
fn main() -> Result<(), env::Reason> {
    roundtrip()
}

fn roundtrip() -> Result<(), env::Reason> {
    let session =
        ipc::session::Session::open(env::unit::sire(), system_client::operator::BERTH, WAIT)
            .map_err(|_| Step::Session.fail(Fail::Dead))?;
    let tree = system_client::operator::Face::of(session);
    let client = Client::find(&tree, WAIT).map_err(|fail| Step::Find.fail(fail))?;
    let pipe = client
        .create(16, WAIT)
        .map_err(|fail| Step::Create.fail(fail))?;
    let me = env::unit::self_id();
    let read = client
        .bind(&pipe, Direction::Read, me, WAIT)
        .map_err(|fail| Step::BindRead.fail(fail))?;
    let write = client
        .bind(&pipe, Direction::Write, me, WAIT)
        .map_err(|fail| Step::BindWrite.fail(fail))?;
    let mut reader = Port::import(read).map_err(|fail| Step::ImportRead.fail(fail))?;
    let mut writer = Port::import(write).map_err(|fail| Step::ImportWrite.fail(fail))?;
    for payload in [
        b"first-pass".as_slice(),
        b"wrap-around-over-capacity".as_slice(),
    ] {
        let mut written = 0;
        let mut received = 0;
        let mut bytes = [0; 32];
        while written < payload.len() {
            match writer
                .write(&payload[written..], WAIT)
                .map_err(|fail| Step::Write.fail(fail))?
            {
                Write::Bytes(n) if n > 0 => written += n,
                _ => return Err(Step::Write.fail(Fail::Protocol)),
            }
            while received < written {
                match reader
                    .read(&mut bytes[received..written], WAIT)
                    .map_err(|fail| Step::Read.fail(fail))?
                {
                    Read::Bytes(n) if n > 0 => received += n,
                    _ => return Err(Step::Read.fail(Fail::Protocol)),
                }
            }
        }
        if &bytes[..payload.len()] != payload {
            return Err(Step::Contents.fail(Fail::Protocol));
        }
    }
    writer.close().map_err(|fail| Step::CloseWrite.fail(fail))?;
    if reader
        .read(&mut [0; 1], WAIT)
        .map_err(|fail| Step::Eof.fail(fail))?
        != Read::Eof
    {
        return Err(Step::Eof.fail(Fail::Protocol));
    }
    reader.close().map_err(|fail| Step::CloseRead.fail(fail))?;
    client
        .close(&pipe, Direction::Write, WAIT)
        .map_err(|fail| Step::ReleaseWrite.fail(fail))?;
    client
        .close(&pipe, Direction::Read, WAIT)
        .map_err(|fail| Step::ReleaseRead.fail(fail))?;
    let read_seed = reader.token();
    let write_seed = writer.token();
    client
        .release(pipe, WAIT)
        .map_err(|fail| Step::Release.fail(fail))?;
    if resource::raw::alive(read_seed) || resource::raw::alive(write_seed) {
        return Err(Step::Revocation.fail(Fail::Protocol));
    }
    drop(writer);
    drop(reader);
    Ok(())
}
