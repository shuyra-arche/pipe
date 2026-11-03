//! Host-only capsule validator/builder for the independent integration.
//!
//! It never runs QEMU, never builds the Kernel, never downloads firmware and
//! never reads a Source path. All three images and the deployment are the
//! provider's explicit declarations.

use std::path::{Path, PathBuf};

use pipe_integration_declaration::{BUILDS, DECLARATIONS, DEPLOYMENT};

struct Options {
    command: String,
    root: PathBuf,
    work: Option<PathBuf>,
    out: Option<PathBuf>,
}

fn usage() -> String {
    "usage: pipe-integration-host check --root <repo> | \
     pipe-integration-host build --root <repo> --work <dir> --out <initrd>"
        .to_string()
}

fn parse() -> Result<Options, String> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).ok_or_else(usage)?.clone();
    let mut root = None;
    let mut work = None;
    let mut out = None;
    let mut at = 2;
    while at < args.len() {
        let value = args.get(at + 1).ok_or_else(|| format!("{} needs a value", args[at]))?;
        match args[at].as_str() {
            "--root" => root = Some(PathBuf::from(value)),
            "--work" => work = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            other => return Err(format!("unknown argument {other}")),
        }
        at += 2;
    }
    Ok(Options {
        command,
        root: root.ok_or("--root is required")?,
        work,
        out,
    })
}

fn check_root(root: &Path) -> Result<(), String> {
    for build in BUILDS {
        let manifest = root.join(build.manifest);
        if !manifest.is_file() {
            return Err(format!("missing image manifest {}", manifest.display()));
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("pipe-integration-host: {error}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<(), String> {
    let options = parse()?;
    check_root(&options.root)?;
    match options.command.as_str() {
        "check" => {
            let images = image_builder::image_inventory(DECLARATIONS, &DEPLOYMENT, BUILDS)?;
            let order = image_builder::startup_order(DECLARATIONS, &DEPLOYMENT)?;
            println!("deployment: {}", DEPLOYMENT.name);
            println!("entry: {}", DEPLOYMENT.entry);
            println!("startup: {}", order.join(" -> "));
            println!("images: {}", images.len());
            for (image, kind) in images {
                println!("image: {image} {kind:?}");
            }
            Ok(())
        }
        "build" => {
            let work = options.work.ok_or("--work is required for build")?;
            let out = options.out.ok_or("--out is required for build")?;
            let capsule = image_builder::build_with(
                &options.root,
                &work,
                &out,
                DECLARATIONS,
                &DEPLOYMENT,
                BUILDS,
            )?;
            println!("capsule: {}", capsule.display());
            Ok(())
        }
        other => Err(format!("unknown command {other}\n{}", usage())),
    }
}
