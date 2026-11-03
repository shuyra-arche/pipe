use alloc::vec::Vec;
use env::{PieToken, TaskId};
use pipe_api::{Direction, Fail, Reply, Request};
use resource::pile::Pile;

use crate::grant::Grant;
use crate::lease::lease;
use crate::ledger::Ledger;
use crate::root::Root;

pub(crate) fn process(
    ledger: &mut Ledger,
    roots: &mut Vec<Root>,
    request: Request,
    from: TaskId,
    pile: &Pile,
) -> Result<Reply, Fail> {
    if request.op == pipe_api::CREATE {
        let capacity = usize::try_from(request.capacity).map_err(|_| Fail::Invalid)?;
        lease(request.lease, from)?;
        roots.try_reserve(1).map_err(|_| Fail::Full)?;
        let id = ledger.create(from.get(), capacity)?;
        match Root::create(id, capacity, request.lease) {
            Ok(root) => {
                if pile
                    .attach(env::ToleEventSource::Inspect {
                        task: env::unit::self_id(),
                        token: root.owner_lease,
                    })
                    .is_err()
                {
                    ledger.release(id);
                    return Err(Fail::Full);
                }
                roots.push(root);
            }
            Err(error) => {
                ledger.release(id);
                return Err(error);
            }
        }
        return Ok(Reply {
            status: 0,
            id,
            capacity: capacity as u64,
            seed: PieToken::NONE,
        });
    }
    let at = roots
        .iter()
        .position(|root| root.id == request.id)
        .ok_or(Fail::Dead)?;
    ledger.authorize(request.id, from.get())?;
    match request.op {
        pipe_api::BIND => {
            let direction = pipe_api::direction(request.direction).ok_or(Fail::Invalid)?;
            let target = ledger.target(request.id, direction)?;
            if target.is_some() {
                return Err(Fail::Denied);
            }
            let root = &mut roots[at];
            if root.header().closed(direction) {
                return Err(Fail::Closed);
            }
            let old = match direction {
                Direction::Read => &root.read,
                Direction::Write => &root.write,
            };
            let seed = if let Some(grant) = old {
                grant.seed
            } else {
                let seed = env::pie::accord(
                    root.token,
                    request.target,
                    env::Permission::FETCH | env::Permission::STORE,
                    pipe_api::DATA,
                )
                .map_err(|_| Fail::Denied)?;
                let source = env::ToleEventSource::Inspect {
                    task: request.target,
                    token: seed,
                };
                if pile.attach(source).is_err() {
                    let _ = env::pie::revoke(request.target, seed);
                    return Err(Fail::Denied);
                }
                if let Err(error) =
                    ledger.bind(request.id, from.get(), direction, request.target.get())
                {
                    let _ = pile.detach(source);
                    let _ = env::pie::revoke(request.target, seed);
                    return Err(error);
                }
                let grant = Grant {
                    seed,
                    target: request.target,
                };
                match direction {
                    Direction::Read => root.read = Some(grant),
                    Direction::Write => root.write = Some(grant),
                }
                seed
            };
            Ok(Reply {
                status: 0,
                id: root.id,
                capacity: root.capacity as u64,
                seed,
            })
        }
        pipe_api::CLOSE => {
            let direction = pipe_api::direction(request.direction).ok_or(Fail::Invalid)?;
            roots[at].close(direction);
            Ok(Reply {
                status: 0,
                id: request.id,
                capacity: 0,
                seed: PieToken::NONE,
            })
        }
        pipe_api::RELEASE => {
            roots[at].detach(pile);
            roots.swap_remove(at);
            ledger.release(request.id);
            Ok(Reply {
                status: 0,
                id: request.id,
                capacity: 0,
                seed: PieToken::NONE,
            })
        }
        _ => Err(Fail::Invalid),
    }
}
