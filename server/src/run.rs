use alloc::vec::Vec;
use env::{PieToken, Wait};
use pipe_api::{Reply, Request};
use resource::pile::Pile;
use wire::Message;

use crate::ended::ended;
use crate::ledger::Ledger;
use crate::process::process;

const WAIT: Wait = Wait::AtMost(5000);

pub fn run() -> Result<(), &'static str> {
    let sire = env::unit::sire();
    let _session = ipc::session::Session::open(sire, system_client::operator::BERTH, WAIT)
        .map_err(|_| "pipe operator session")?;
    let entry =
        env::hole::unseal(pipe_api::ENTRY, env::HoleLimits::default()).map_err(|_| "pipe entry")?;
    let capability = system_client::operator::Capability::injected(0, sire, WAIT)
        .map_err(|_| "pipe namespace capability")?;
    let anchor = capability.anchor().map_err(|_| "pipe namespace anchor")?;
    let view =
        system_api::operator::ResourceView::new(env::Access::STORE).ok_or("pipe endpoint view")?;
    capability
        .bind(
            anchor,
            "create",
            entry,
            view,
            system_api::operator::Permit::Public,
            system_api::operator::OperationId(1),
            WAIT,
        )
        .map_err(|_| "pipe endpoint binding")?;
    let _ready = ipc::session::establish::lend(sire, unit::READY_MARK)
        .map_err(|_| "pipe readiness")?;
    let receiver = ipc::rpc::request::Receiver::<pipe_api::Call>::from_raw(
        entry,
        pipe_api::Call::BACK,
        pipe_api::Call::back,
    );
    let pile = Pile::unseal(false).map_err(|_| "pipe wait group")?;
    pile.attach(env::ToleEventSource::Hole {
        token: entry,
        condition: env::HoleCondition::Pull,
    })
    .map_err(|_| "pipe request wait")?;
    let mut ledger = Ledger::default();
    let mut roots = Vec::new();
    let mut bytes = Request::EMPTY;
    loop {
        match pile.await_(Wait::Forever).map_err(|_| "pipe wait")? {
            env::ToleAwaitReply::Source {
                source: env::ToleEventSource::Inspect { task, token },
                ..
            } => {
                ended(
                    &mut ledger,
                    &mut roots,
                    &pile,
                    env::ToleEventSource::Inspect { task, token },
                );
                continue;
            }
            env::ToleAwaitReply::Source { fail: Some(_), .. } => return Err("pipe entry lost"),
            env::ToleAwaitReply::Pending => continue,
            _ => {}
        }
        let incoming = match receiver.receive(&mut bytes, Wait::POLL) {
            Ok(request) => request,
            Err(_) => continue,
        };
        let result = process(
            &mut ledger,
            &mut roots,
            incoming.request,
            incoming.from,
            &pile,
        );
        let reply = match result {
            Ok(reply) => reply,
            Err(error) => Reply {
                status: error.code(),
                id: incoming.request.id,
                capacity: 0,
                seed: PieToken::NONE,
            },
        };
        if incoming.reply.send(reply).is_err() && incoming.request.op == pipe_api::CREATE {
            if let Some(at) = roots.iter().position(|root| root.id == reply.id) {
                roots[at].detach(&pile);
                roots.swap_remove(at);
                ledger.release(reply.id);
            }
        }
    }
}
