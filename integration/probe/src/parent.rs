//! Parent role (arg 0): declared Unit, real owner of pipe B, adversarial
//! consumer of the peer-owned pipe A.

use core::time::Duration;
use env::{PieToken, TaskId, Wait};
use execution::boot::Report;
use pipe_api::{Endpoint, Fail};
use pipe_client::{Client, Direction, Port, Read};
use pipe_integration_declaration::{
    B_DEAD, BIND_B, COMMAND, CTX_PARENT, Command as PeerCommand, Context, FAIL, PIPE_A,
    RELEASED_B, REPORT, Report as PeerReport,
};
use resource::raw::Hole;
use system_support::debug;
use crate::wire;
use ::wire::Message;

const WAIT: Wait = Wait::AtMost(5000);
const BUDGET_NS: u64 = 30_000_000_000;

fn note(text: &str) {
    debug::put(text);
}

fn fail(code: u8, text: &'static str) -> Report<'static> {
    note(text);
    note("PIPE-INTEGRATION: FAIL");
    Report::note(code as usize, "pipe-probe: failed")
}

pub fn run() -> Report<'static> {
    match inner() {
        Ok(()) => {
            note("PIPE-INTEGRATION: PASS");
            Report::note(env::EXIT_OK, "pipe-probe: pass")
        }
        Err((code, text)) => fail(code, text),
    }
}

fn receive(hole: &Hole, expect: TaskId) -> Option<PeerReport> {
    let mut bytes = <PeerReport as Message>::EMPTY;
    let len = wire::pull(hole, &mut bytes, expect)?;
    PeerReport::fetch(&bytes[..len])
}

fn send(hole: &Hole, command: PeerCommand) -> bool {
    let mut bytes = <PeerCommand as Message>::EMPTY;
    let Some(len) = command.store(&mut bytes) else {
        return false;
    };
    wire::push(hole, &bytes[..len])
}

fn inner() -> Result<(), (u8, &'static str)> {
    let sire = env::unit::sire();
    if sire.get() == 0 {
        return Err((1, "PIPE: FAIL sire"));
    }
    let session = ipc::session::Session::open(sire, system_client::operator::BERTH, WAIT)
        .map_err(|_| (2, "PIPE: FAIL operator-session"))?;
    let tree = system_client::operator::Face::of(session);
    let entry = wire::resolve_entry(&tree, WAIT).map_err(|_| (3, "PIPE: FAIL pipe-entry"))?;
    let client = Client::of(entry).map_err(|_| (4, "PIPE: FAIL pipe-client"))?;
    let me = env::unit::self_id();

    let context = wire::claim(sire, CTX_PARENT, env::Permission::FETCH)
        .ok_or((5, "PIPE: FAIL context"))?;
    let mut bytes = <Context as Message>::EMPTY;
    let len = wire::pull(&context, &mut bytes, sire).ok_or((5, "PIPE: FAIL context frame"))?;
    let frame = Context::fetch(&bytes[..len]).ok_or((5, "PIPE: FAIL context decode"))?;
    if frame.parent != me || frame.peer == me {
        return Err((5, "PIPE: FAIL context identity"));
    }
    let peer = frame.peer;
    let command = wire::claim(sire, COMMAND, env::Permission::STORE)
        .ok_or((6, "PIPE: FAIL command"))?;
    let reports = wire::claim(sire, REPORT, env::Permission::FETCH)
        .ok_or((6, "PIPE: FAIL report"))?;

    let mut round = 0;
    while round < 2 {
        if !matches!(client.create(0, WAIT), Err(Fail::Invalid)) {
            return Err((10, "PIPE: FAIL create-invalid"));
        }
        if !matches!(
            client.create(pipe_api::MAX_CAPACITY + 1, WAIT),
            Err(Fail::Invalid)
        ) {
            return Err((10, "PIPE: FAIL create-invalid"));
        }
        round += 1;
    }
    note("PIPE: create-invalid");
    let pipe = client.create(16, WAIT).map_err(|_| (13, "PIPE: FAIL create"))?;
    note("PIPE: create");
    let actors = alloc::format!("PIPE: actors parent={} child={}", me.get(), peer.get());
    note(&actors);

    let a = receive(&reports, peer).ok_or((11, "PIPE: FAIL peer-a"))?;
    if a.kind == FAIL {
        return Err((11, "PIPE: FAIL peer"));
    }
    if a.kind != PIPE_A {
        return Err((11, "PIPE: FAIL peer-a kind"));
    }
    let a_seed = PieToken::from_bytes(&a.seed).ok_or((11, "PIPE: FAIL peer-a seed"))?;
    let mut challenge = wire::request(pipe_api::BIND, a.id);
    challenge.target = me;
    challenge.direction = pipe_api::WRITE;
    match wire::call(entry, challenge, WAIT) {
        Err(Fail::Denied) => {}
        _ => return Err((12, "PIPE: FAIL ownerauth-denied")),
    }
    note("PIPE: ownerauth-denied distinct-unit");
    let mut a_port = Port::import(Endpoint {
        id: a.id,
        capacity: a.capacity as usize,
        seed: a_seed,
        direction: Direction::Read,
    })
    .map_err(|_| (12, "PIPE: FAIL peer-a import"))?;
    let mut bad_direction = wire::request(pipe_api::BIND, pipe.id);
    bad_direction.direction = 3;
    match wire::call(entry, bad_direction, WAIT) {
        Err(Fail::Invalid) => {}
        _ => return Err((14, "PIPE: FAIL error-invalid direction")),
    }
    let mut bad_target = wire::request(pipe_api::BIND, pipe.id);
    bad_target.direction = pipe_api::READ;
    bad_target.target = TaskId::new(0);
    match wire::call(entry, bad_target, WAIT) {
        Err(Fail::Denied) => {}
        _ => return Err((14, "PIPE: FAIL error-invalid target")),
    }
    note("PIPE: error-invalid");
    match wire::call(entry, wire::request(pipe_api::CLOSE, u64::MAX), WAIT) {
        Err(Fail::Dead) => {}
        _ => return Err((15, "PIPE: FAIL error-dead")),
    }
    note("PIPE: error-dead");

    let read = client
        .bind(&pipe, Direction::Read, me, WAIT)
        .map_err(|_| (16, "PIPE: FAIL bind-read"))?;
    let read_seed = read.seed;
    let mut reader = Port::import(read).map_err(|_| (16, "PIPE: FAIL bind-read"))?;
    if !matches!(client.bind(&pipe, Direction::Read, me, WAIT), Err(Fail::Denied)) {
        return Err((16, "PIPE: FAIL bind-read exclusive"));
    }
    let write = client
        .bind(&pipe, Direction::Write, peer, WAIT)
        .map_err(|_| (17, "PIPE: FAIL bind-write"))?;
    if !matches!(
        client.bind(&pipe, Direction::Write, peer, WAIT),
        Err(Fail::Denied)
    ) {
        return Err((17, "PIPE: FAIL bind-write exclusive"));
    }
    note("PIPE: bind-read");
    note("PIPE: bind-write");
    if !send(
        &command,
        PeerCommand {
            kind: BIND_B,
            id: pipe.id,
            capacity: pipe.capacity as u32,
            seed: write.seed.to_bytes(),
        },
    ) {
        return Err((17, "PIPE: FAIL bind-command"));
    }

    let expected = wire::payload();
    let mut received = [0u8; 40];
    let mut at = 0;
    let until = env::chrono::clock().saturating_add(BUDGET_NS);
    while at < received.len() {
        match reader.read(&mut received[at..], WAIT) {
            Ok(Read::Bytes(step)) if step != 0 => at += step,
            Ok(Read::Pending) => {
                if env::chrono::clock() >= until {
                    return Err((18, "PIPE: FAIL transfer timeout"));
                }
            }
            _ => return Err((18, "PIPE: FAIL transfer")),
        }
    }
    if received != expected {
        return Err((18, "PIPE: FAIL transfer bytes"));
    }
    note("PIPE: transfer");
    let mut sink = [0u8; 1];
    loop {
        match reader.read(&mut sink, WAIT) {
            Ok(Read::Eof) => break,
            Ok(Read::Bytes(0)) => {}
            Ok(Read::Pending) => {
                if env::chrono::clock() >= until {
                    return Err((19, "PIPE: FAIL close timeout"));
                }
            }
            _ => return Err((19, "PIPE: FAIL close")),
        }
    }
    note("PIPE: close");
    reader.close().map_err(|_| (19, "PIPE: FAIL close port"))?;

    let pair = client
        .create(8, WAIT)
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe create"))?;
    let pair_read = client
        .bind(&pair, Direction::Read, me, WAIT)
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe read"))?;
    let pair_write = client
        .bind(&pair, Direction::Write, me, WAIT)
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe write"))?;
    let mut pair_reader = Port::import(pair_read)
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe read import"))?;
    let mut pair_writer = Port::import(pair_write)
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe write import"))?;
    pair_reader
        .close()
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe read close"))?;
    match pair_writer.write(b"x", WAIT) {
        Err(Fail::BrokenPipe) => {}
        _ => return Err((20, "PIPE: FAIL error-broken-pipe")),
    }
    note("PIPE: error-broken-pipe");
    pair_writer
        .close()
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe write close"))?;
    client
        .release(pair, WAIT)
        .map_err(|_| (20, "PIPE: FAIL error-broken-pipe release"))?;

    client
        .release(pipe, WAIT)
        .map_err(|_| (21, "PIPE: FAIL release"))?;
    if resource::raw::alive(read_seed) {
        return Err((21, "PIPE: FAIL release left reader alive"));
    }
    note("PIPE: release");
    if !send(
        &command,
        PeerCommand {
            kind: RELEASED_B,
            id: 0,
            capacity: 0,
            seed: [0; 8],
        },
    ) {
        return Err((21, "PIPE: FAIL release-command"));
    }
    let dead = receive(&reports, peer).ok_or((22, "PIPE: FAIL peer-dead"))?;
    if dead.kind != B_DEAD || dead.code != 0 {
        return Err((22, "PIPE: FAIL error-closed"));
    }
    note("PIPE: error-closed");

    let until = env::chrono::clock().saturating_add(BUDGET_NS);
    loop {
        let seed_dead = !resource::raw::alive(a_seed);
        let record_gone = matches!(
            wire::call(entry, wire::request(pipe_api::CLOSE, a.id), WAIT),
            Err(Fail::Dead)
        );
        if seed_dead && record_gone {
            break;
        }
        if env::chrono::clock() >= until {
            return Err((23, "PIPE: FAIL owner-death"));
        }
        execution::room::park(Duration::from_millis(1))
            .map_err(|_| (23, "PIPE: FAIL owner-death park"))?;
    }
    match a_port.read(&mut sink, WAIT) {
        Err(Fail::Dead) => {}
        _ => return Err((23, "PIPE: FAIL owner-death read")),
    }
    note("PIPE: owner-death");

    let fresh = client
        .create(8, WAIT)
        .map_err(|_| (24, "PIPE: FAIL cleanup create"))?;
    client
        .release(fresh, WAIT)
        .map_err(|_| (24, "PIPE: FAIL cleanup release"))?;
    note("PIPE: cleanup");
    Ok(())
}
