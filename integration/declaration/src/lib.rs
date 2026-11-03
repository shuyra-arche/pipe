#![no_std]
//! Explicit independent integration declaration: three images, four UnitFiles,
//! three started leaves. No Source catalog or path is consulted.

use env::Mark;
use unit::{
    Bootstrap, ControlConfig, Dependency, Deployment, Ending, Identity, ImageBuild, LoaderConfig,
    OperatorConfig, Ready, SupplyConfig, UnitFile, scene_target,
};

mod command;
mod context;
mod report;

pub use command::{BIND_B, Command, RELEASED_B};
pub use context::Context;
pub use report::{B_DEAD, FAIL, PIPE_A, Report};

/// Marked Root-created Hole channels between the two declared leaf Units.
pub const CTX_PARENT: Mark = Mark::of("pipe-it/ctx-parent");
pub const CTX_PEER: Mark = Mark::of("pipe-it/ctx-peer");
pub const REPORT: Mark = Mark::of("pipe-it/report");
pub const COMMAND: Mark = Mark::of("pipe-it/command");

pub static PIPE_ROOT: UnitFile = UnitFile {
    name: "pipe-root",
    loader: LoaderConfig::supervisor("pipe-root"),
    operator: OperatorConfig::DEFAULT,
    identity: Identity::DEFAULT,
    control: ControlConfig {
        supervision: None,
        after: None,
        ending: Some(Ending::Resident),
        ready: Ready::OnSpawn,
        construction: &[],
        retirement: &[],
    },
    supply: SupplyConfig::DEFAULT,
};

/// Parent role (arg 0) and peer role (arg 1) share one declared ELF.
pub static PIPE_PROBE: UnitFile = UnitFile {
    name: "pipe-probe",
    loader: LoaderConfig {
        image: "pipe-probe",
        space: env::ProgramKind::User,
        args: &[0],
        stack: 0,
    },
    operator: OperatorConfig::DEFAULT,
    identity: Identity::DEFAULT,
    control: ControlConfig {
        supervision: None,
        after: Some(&[Dependency::Unit("pipe")]),
        ending: Some(Ending::Transient),
        ready: Ready::OnSpawn,
        construction: &[],
        retirement: &[],
    },
    supply: SupplyConfig::DEFAULT,
};

pub static PIPE_PEER: UnitFile = UnitFile {
    name: "pipe-peer",
    loader: LoaderConfig {
        image: "pipe-probe",
        space: env::ProgramKind::User,
        args: &[1],
        stack: 0,
    },
    operator: OperatorConfig::DEFAULT,
    identity: Identity::DEFAULT,
    control: ControlConfig {
        supervision: None,
        after: Some(&[Dependency::Unit("pipe")]),
        ending: Some(Ending::Transient),
        ready: Ready::OnSpawn,
        construction: &[],
        retirement: &[],
    },
    supply: SupplyConfig::DEFAULT,
};

pub static DECLARATIONS: &[&UnitFile] = &[
    &pipe_declaration::UNIT,
    &PIPE_ROOT,
    &PIPE_PROBE,
    &PIPE_PEER,
];

pub static DEPLOYMENT: Deployment = Deployment {
    name: "pipe-independent",
    entry: "pipe-root",
    start: &["pipe", "pipe-probe", "pipe-peer"],
    guarded: &[],
    resources: &[],
    targets: &[scene_target(&["pipe", "pipe-probe", "pipe-peer"])],
    bootstrap: &[Bootstrap::Operator, Bootstrap::Identity],
    sources: &[],
};

pub static BUILDS: &[ImageBuild] = &[
    pipe_declaration::BUILD,
    ImageBuild {
        image: "pipe-root",
        manifest: "integration/root/Cargo.toml",
        package: "pipe-root",
        bin: "pipe-root",
        features: &[],
        target: "riscv64gc-unknown-none-elf",
        profile: "release",
        default_features: true,
    },
    ImageBuild {
        image: "pipe-probe",
        manifest: "integration/probe/Cargo.toml",
        package: "pipe-probe",
        bin: "pipe-probe",
        features: &[],
        target: "riscv64gc-unknown-none-elf",
        profile: "release",
        default_features: true,
    },
];
