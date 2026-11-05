# Dynamic Unit and Rust roundtrip case

`server/unit.toml` retains the native Pipe namespace, permissions, create mark,
ready signal and resident policy while identifying the actual `pipe` Cargo bin.
The `test.pipe-roundtrip` declaration selects Pipe via requires/after and ends
transiently. Unit declarations and real bin metadata supply the necessary image
set without central registration.

The Rust guest creates capacity 16, exercises two payloads across ring wraparound,
checks EOF after writer close, closes/drops mapped ports and explicitly releases
the Pipe. Failure returns native reason 17. Pass/fail is determined by System's
typed native exit and reclamation receipts.

Validation used nightly-2026-10-09. The actual guest bin built for RV64 from the
real manifest and formal Cargo.lock without local patches:

```sh
cargo build --manifest-path tests/roundtrip/Cargo.toml --target riscv64gc-unknown-none-elf --locked
```

A host CLI build has also packed actual System, Pipe and guest ELFs into initrd.
No guest, SMP or runtime integration acceptance was run. Existing tests/scenarios
are unchanged and are not evidence for this new runtime path.
