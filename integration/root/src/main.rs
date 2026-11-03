#![no_std]
#![no_main]
//! Independent integration root: running Supervisor entry. It mints/embarks the
//! pipe service first, then mints both declared leaf Units, publishes their real
//! TaskIds, ships Root-owned typed Hole test channels, and embarks them.

extern crate alloc;

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use env::{Permission, TaskId, Wait};
use execution::boot::Report;
use pipe_integration_declaration::{
    COMMAND, CTX_PARENT, CTX_PEER, Context, REPORT,
};
use resource::port::{self, Access, Policy};
use resource::raw::Hole;
use system_server::boot;
use system_server::debug;
use system_server::test_support::{Fixture, FixtureState, SupplyHook};
use wire::Message;

const PLIC_CLASS: &str = "sifive,plic-1.0.0";
const BUDGET_NS: u64 = 90_000_000_000;

static PARENT: AtomicUsize = AtomicUsize::new(0);
static PEER: AtomicUsize = AtomicUsize::new(0);
static SUPPLIED: AtomicBool = AtomicBool::new(false);

fn channel(mark: env::Mark) -> Result<Hole, &'static str> {
    let token =
        env::hole::unseal(mark, env::HoleLimits::default()).map_err(|_| "pipe channel hole")?;
    Ok(Hole::from_raw(token))
}

fn context(
    mark: env::Mark,
    target: TaskId,
    parent: TaskId,
    peer: TaskId,
) -> Result<(), &'static str> {
    let hole = channel(mark)?;
    let frame = Context { parent, peer };
    let mut bytes = <Context as Message>::EMPTY;
    let len = frame.store(&mut bytes).ok_or("pipe context frame")?;
    hole.push(&bytes[..len], Wait::POLL)
        .map_err(|_| "pipe context push")?;
    port::ship(hole.token(), target, Access::FETCH, Policy::NONE)
        .map_err(|_| "pipe context ship")?;
    env::pie::narrow(hole.token(), Permission::STORE).map_err(|_| "pipe context narrow")?;
    Ok(())
}

fn setup() -> Result<(), &'static str> {
    let parent = TaskId::new(PARENT.load(Ordering::Acquire));
    let peer = TaskId::new(PEER.load(Ordering::Acquire));
    if parent.get() == 0 || peer.get() == 0 || parent == peer {
        return Err("pipe declared ids");
    }
    context(CTX_PARENT, parent, parent, peer)?;
    context(CTX_PEER, peer, parent, peer)?;

    let report = channel(REPORT)?;
    port::ship(report.token(), peer, Access::STORE, Policy::NONE)
        .map_err(|_| "pipe report peer")?;
    port::ship(report.token(), parent, Access::FETCH, Policy::NONE)
        .map_err(|_| "pipe report parent")?;

    let command = channel(COMMAND)?;
    port::ship(command.token(), parent, Access::STORE, Policy::NONE)
        .map_err(|_| "pipe command parent")?;
    port::ship(command.token(), peer, Access::FETCH, Policy::NONE)
        .map_err(|_| "pipe command peer")?;
    Ok(())
}

fn supply<'a>(
    _: Option<TaskId>,
    unit: &'a system_server::unit::UnitFile,
    _: TaskId,
) -> Result<(), &'static str> {
    if unit.name() == "pipe-probe" {
        if SUPPLIED.swap(true, Ordering::AcqRel) {
            return Err("pipe duplicate supply");
        }
        setup()?;
    }
    Ok(())
}

fn failed(code: u8, note: &'static str) -> Report<'static> {
    debug::put(note);
    Report::note(code as usize, "pipe-root: failed")
}

#[mold::entry]
fn main() -> Report<'static> {
    let boot = match boot::take(PLIC_CLASS) {
        Ok(boot) => boot,
        Err(_) => return failed(1, "PIPE-ROOT: FAIL boot"),
    };
    if Fixture::scene(&boot, pipe_integration_declaration::DECLARATIONS, &pipe_integration_declaration::DEPLOYMENT).is_err() {
        return failed(2, "PIPE-ROOT: FAIL scene");
    }
    let hook: SupplyHook = supply;
    let mut fixture = match Fixture::new(
        boot,
        pipe_integration_declaration::DECLARATIONS,
        &pipe_integration_declaration::DEPLOYMENT,
        hook,
    ) {
        Ok(fixture) => fixture,
        Err(_) => return failed(3, "PIPE-ROOT: FAIL install"),
    };
    if fixture.mint("pipe").is_err() || fixture.embark("pipe").is_err() {
        return failed(4, "PIPE-ROOT: FAIL pipe");
    }
    let parent = match fixture.mint("pipe-probe") {
        Ok(task) => task,
        Err(_) => return failed(5, "PIPE-ROOT: FAIL mint probe"),
    };
    let peer = match fixture.mint("pipe-peer") {
        Ok(task) => task,
        Err(_) => return failed(6, "PIPE-ROOT: FAIL mint peer"),
    };
    PARENT.store(parent.get(), Ordering::Release);
    PEER.store(peer.get(), Ordering::Release);
    if fixture.embark("pipe-probe").is_err() || fixture.embark("pipe-peer").is_err() {
        return failed(7, "PIPE-ROOT: FAIL embark leaves");
    }

    let until = env::chrono::clock().saturating_add(BUDGET_NS);
    loop {
        if fixture.progress().is_err() {
            return failed(8, "PIPE-ROOT: FAIL progress");
        }
        match env::unit::join_task(parent, env::Wait::POLL) {
            Ok(true) => break,
            Ok(false) => {}
            Err(_) => return failed(9, "PIPE-ROOT: FAIL parent-join"),
        }
        if env::chrono::clock() >= until {
            return failed(10, "PIPE-ROOT: FAIL parent-timeout");
        }
        if execution::room::park(core::time::Duration::from_millis(1)).is_err() {
            return failed(11, "PIPE-ROOT: FAIL park");
        }
    }

    fixture.settle();
    if fixture.supervise().is_err() {
        return failed(12, "PIPE-ROOT: FAIL supervise");
    }
    let all_terminal = fixture.instances().is_ok_and(|views| {
        views.iter().all(|view| {
            matches!(
                view.state,
                FixtureState::NeverStarted | FixtureState::Dead | FixtureState::Debarked
            )
        })
    });
    if !all_terminal {
        return failed(13, "PIPE-ROOT: FAIL terminal");
    }
    debug::put("PIPE-ROOT: SHUTDOWN_OK");
    Report::note(env::EXIT_OK, "pipe-root: shutdown completed")
}
