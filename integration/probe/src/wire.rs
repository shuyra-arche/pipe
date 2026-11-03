//! Shared fixture helpers: public pipe RPC, marked Hole channels and the
//! deterministic 40-byte payload.

use env::{PieToken, TaskId, Wait};
use pipe_api::{Call, Fail, Reply, Request};
use resource::raw::Hole;

pub const WAIT: Wait = Wait::AtMost(5000);

pub fn resolve_entry(
    tree: &system_client::operator::Face,
    wait: Wait,
) -> Result<PieToken, Fail> {
    let base = tree
        .resolve(system_api::namespace::SERVICE, wait)
        .map_err(|_| Fail::Denied)?;
    tree.tile_from(base, system_api::operator::Path::new(pipe_api::DIR), wait)
        .and_then(|tile| tile.token(wait))
        .map_err(|_| Fail::Denied)
}

pub fn request(op: u8, id: u64) -> Request {
    Request {
        op,
        id,
        capacity: 0,
        target: env::unit::self_id(),
        direction: 0,
        lease: PieToken::NONE,
        back: PieToken::NONE,
    }
}

pub fn call(entry: PieToken, mut request: Request, wait: Wait) -> Result<Reply, Fail> {
    let sender = ipc::rpc::request::Sender::<Call>::from_raw(entry, Call::BACK)
        .map_err(|_| Fail::Denied)?;
    let reply = sender
        .call(ipc::time::Deadline::new(wait), |back| {
            request.back = back;
            request
        })
        .map_err(|_| Fail::Dead)?;
    if reply.status != 0 {
        return Err(Fail::of(reply.status).unwrap_or(Fail::Protocol));
    }
    Ok(reply)
}

/// Claim one unique Root-shipped Hole and verify its actual source/permission.
pub fn claim(
    sire: TaskId,
    mark: env::Mark,
    permission: env::Permission,
) -> Option<Hole> {
    let token = ipc::session::establish::claim(sire, mark, WAIT).ok()?;
    let info = resource::raw::inspect(token).ok()?;
    if !info.alive
        || info.owner != sire
        || info.vestor != sire
        || !info.permission.contains(permission)
    {
        return None;
    }
    Some(Hole::from_raw(token))
}

pub fn pull(hole: &Hole, bytes: &mut [u8], expect: TaskId) -> Option<usize> {
    let (len, from) = hole.pull(bytes, WAIT).ok()?;
    (from == expect).then_some(len)
}

pub fn push(hole: &Hole, bytes: &[u8]) -> bool {
    hole.push(bytes, WAIT).is_ok()
}

pub fn payload() -> [u8; 40] {
    let mut bytes = [0u8; 40];
    let mut at = 0;
    while at < bytes.len() {
        bytes[at] = (at as u8).wrapping_mul(3).wrapping_add(1);
        at += 1;
    }
    bytes
}
