//! Peer role (arg 1): declared Unit that owns pipe A and writes pipe B.

use env::{PieToken, TaskId, Wait};
use execution::boot::Report;
use pipe_api::{Endpoint, Fail};
use pipe_client::{Client, Direction, Port, Write};
use pipe_integration_declaration::{
    B_DEAD, BIND_B, COMMAND, CTX_PEER, Command as PeerCommand, Context, FAIL, PIPE_A, RELEASED_B,
    REPORT, Report as PeerReport,
};
use resource::raw::Hole;
use system_support::debug;
use crate::wire;
use ::wire::Message;

const WAIT: Wait = Wait::AtMost(5000);

fn note(text: &str) {
    debug::put(text);
}

fn receive(hole: &Hole, expect: TaskId) -> Option<PeerCommand> {
    let mut bytes = <PeerCommand as Message>::EMPTY;
    let len = wire::pull(hole, &mut bytes, expect)?;
    PeerCommand::fetch(&bytes[..len])
}

fn send_report(hole: &Hole, report: PeerReport) -> bool {
    let mut bytes = <PeerReport as Message>::EMPTY;
    let Some(len) = report.store(&mut bytes) else {
        return false;
    };
    wire::push(hole, &bytes[..len])
}

pub fn run() -> Report<'static> {
    match inner() {
        Ok(()) => Report::note(env::EXIT_OK, "pipe-peer: done"),
        Err((code, _)) => {
            let line = alloc::format!("PIPE: peer FAIL code={code}");
            note(&line);
            Report::note(code as usize, "pipe-peer: failed")
        }
    }
}

fn inner() -> Result<(), (u8, &'static str)> {
    let sire = env::unit::sire();
    if sire.get() == 0 {
        return Err((1, "PIPE: peer FAIL sire"));
    }
    let session = ipc::session::Session::open(sire, system_client::operator::BERTH, WAIT)
        .map_err(|_| (2, "PIPE: peer FAIL operator-session"))?;
    let tree = system_client::operator::Face::of(session);
    let entry = wire::resolve_entry(&tree, WAIT).map_err(|_| (3, "PIPE: peer FAIL pipe-entry"))?;
    let client = Client::of(entry).map_err(|_| (4, "PIPE: peer FAIL pipe-client"))?;
    let me = env::unit::self_id();

    let context = wire::claim(sire, CTX_PEER, env::Permission::FETCH)
        .ok_or((5, "PIPE: peer FAIL context"))?;
    let mut bytes = <Context as Message>::EMPTY;
    let len = wire::pull(&context, &mut bytes, sire).ok_or((5, "PIPE: peer FAIL context frame"))?;
    let frame = Context::fetch(&bytes[..len]).ok_or((5, "PIPE: peer FAIL context decode"))?;
    if frame.peer != me || frame.parent == me {
        return Err((5, "PIPE: peer FAIL context identity"));
    }
    let parent = frame.parent;
    let command = wire::claim(sire, COMMAND, env::Permission::FETCH)
        .ok_or((6, "PIPE: peer FAIL command"))?;
    let reports = wire::claim(sire, REPORT, env::Permission::STORE)
        .ok_or((6, "PIPE: peer FAIL report"))?;

    let pipe = client.create(32, WAIT).map_err(|_| (10, "PIPE: peer FAIL create"))?;
    let read = client
        .bind(&pipe, Direction::Read, parent, WAIT)
        .map_err(|_| (11, "PIPE: peer FAIL bind"))?;
    if !send_report(
        &reports,
        PeerReport {
            kind: PIPE_A,
            code: 0,
            id: pipe.id,
            capacity: pipe.capacity as u32,
            seed: read.seed.to_bytes(),
        },
    ) {
        return Err((12, "PIPE: peer FAIL report A"));
    }

    let bind = receive(&command, parent).ok_or((13, "PIPE: peer FAIL command A"))?;
    if bind.kind != BIND_B {
        return Err((13, "PIPE: peer FAIL command kind"));
    }
    let seed = PieToken::from_bytes(&bind.seed).ok_or((14, "PIPE: peer FAIL seed"))?;
    let mut port = Port::import(Endpoint {
        id: bind.id,
        capacity: bind.capacity as usize,
        seed,
        direction: Direction::Write,
    })
    .map_err(|_| (14, "PIPE: peer FAIL import"))?;

    let payload = wire::payload();
    let mut at = 0;
    while at < payload.len() {
        match port.write(&payload[at..], WAIT) {
            Ok(Write::Bytes(step)) if step != 0 => at += step,
            _ => return Err((15, "PIPE: peer FAIL write")),
        }
    }
    port.close().map_err(|_| (16, "PIPE: peer FAIL close"))?;
    let closed = matches!(port.write(b"x", WAIT), Err(Fail::Closed));

    let release = receive(&command, parent).ok_or((17, "PIPE: peer FAIL release"))?;
    if release.kind != RELEASED_B {
        return Err((17, "PIPE: peer FAIL release kind"));
    }
    let dead = !resource::raw::alive(seed);
    let code = u8::from(!(closed && dead));
    if !send_report(
        &reports,
        PeerReport {
            kind: B_DEAD,
            code,
            id: 0,
            capacity: 0,
            seed: [0; 8],
        },
    ) {
        return Err((18, "PIPE: peer FAIL report dead"));
    }
    if code != 0 {
        return Err((18, "PIPE: peer FAIL closed/dead"));
    }
    let _ = FAIL;
    core::mem::forget(pipe);
    Ok(())
}
