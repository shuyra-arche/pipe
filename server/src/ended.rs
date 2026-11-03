use alloc::vec::Vec;
use pipe_api::Direction;
use resource::pile::Pile;

use crate::ledger::Ledger;
use crate::root::Root;

pub(crate) fn ended(
    ledger: &mut Ledger,
    roots: &mut Vec<Root>,
    pile: &Pile,
    source: env::ToleEventSource,
) {
    let env::ToleEventSource::Inspect { task, token } = source else {
        return;
    };
    let mut at = 0;
    while at < roots.len() {
        let root = &mut roots[at];
        if task == env::unit::self_id() && token == root.owner_lease {
            root.close(Direction::Read);
            root.close(Direction::Write);
            root.detach(pile);
            let id = root.id;
            roots.swap_remove(at);
            ledger.release(id);
            continue;
        }
        for direction in [Direction::Read, Direction::Write] {
            let grant = match direction {
                Direction::Read => &root.read,
                Direction::Write => &root.write,
            };
            if grant
                .as_ref()
                .is_some_and(|g| g.target == task && g.seed == token)
            {
                root.close(direction);
                let _ = pile.detach(source);
                match direction {
                    Direction::Read => root.read = None,
                    Direction::Write => root.write = None,
                }
            }
        }
        if root.header().closed(Direction::Read)
            && root.header().closed(Direction::Write)
            && root.read.is_none()
            && root.write.is_none()
        {
            root.detach(pile);
            let id = root.id;
            roots.swap_remove(at);
            ledger.release(id);
        } else {
            at += 1;
        }
    }
}
