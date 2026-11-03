#![no_std]
//! Pure pipe unit declaration and the image target that builds its server.
//!
//! This crate publishes facts only: no catalog, no registration, no Source
//! composition. The consumer chooses where `UNIT` is listed.

use unit::{Ending, ImageBuild, UnitFile};

pub static UNIT: UnitFile = UnitFile {
    name: "pipe",
    loader: unit::LoaderConfig::user("pipe"),
    operator: unit::OperatorConfig {
        namespace: &[unit::operator::Namespace {
            key: system_api::namespace::SERVICE,
            placement: unit::operator::Placement::Shared,
            path: "pipe",
            scope: unit::operator::Scope::Subtree,
            ops: system_api::operator::Ops::PART
                .union(system_api::operator::Ops::UNPART)
                .union(system_api::operator::Ops::BIND)
                .union(system_api::operator::Ops::UNBIND)
                .union(system_api::operator::Ops::FIND),
            permit: Some(system_api::operator::Permit::Public),
            source_mark: None,
            bindings: &[unit::operator::Binding {
                name: "create",
                mark: pipe_api::ENTRY,
                view: env::Access::STORE,
                permit: system_api::operator::Permit::Public,
            }],
        }],
    },
    identity: unit::Identity {
        aliases: &[],
        grants: &[],
    },
    control: unit::ControlConfig {
        supervision: None,
        after: Some(&[
            unit::Dependency::Bootstrap(unit::Bootstrap::Operator),
            unit::Dependency::Bootstrap(unit::Bootstrap::Identity),
        ]),
        ready: unit::Ready::Signal(unit::READY_MARK),
        ending: Some(Ending::Resident),

        construction: &[],
        retirement: &[],
    },
    supply: unit::SupplyConfig { bindings: &[] },
};

pub const BUILD: ImageBuild = ImageBuild {
    image: "pipe",
    manifest: "server/Cargo.toml",
    package: "pipe-server",
    bin: "pipe",
    features: &[],
    target: "riscv64gc-unknown-none-elf",
    profile: "release",
    default_features: true,
};
