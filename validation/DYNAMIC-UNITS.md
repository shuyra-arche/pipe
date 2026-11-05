# Dynamic Unit and Rust roundtrip case

`server/unit.toml` retains the native Pipe namespace, permissions, create mark,
ready signal and resident policy while identifying the actual `pipe` Cargo bin.
The `test.pipe-roundtrip` declaration selects Pipe via requires/after and ends
transiently. Unit declarations and real bin metadata supply the necessary image
set without central registration.

The Rust guest creates capacity 16 and exercises ring wraparound with a payload
larger than capacity, requiring partial I/O. It checks EOF, closes directions,
explicitly releases the Pipe while endpoint references remain held, verifies
both native capabilities are revoked, then drops ports. Failure returns
`0x500000 | (step << 8) | Fail::code()`; `src/step.rs` names each stage.
Pass/fail is determined by System's
typed native exit and reclamation receipts.

Validation used nightly-2026-10-09. The actual guest bin built for RV64 from the
real manifest and formal Cargo.lock without local patches:

```sh
cargo build --manifest-path tests/roundtrip/Cargo.toml --target riscv64gc-unknown-none-elf --locked
```

A host CLI build has also packed actual System, Pipe and guest ELFs into initrd.
The minimal three-Unit case passed single-core QEMU 10.0.13 / OpenSBI 1.6 with
native Reap/zero exit, acknowledged retirement and VM exit zero. The initial
test dropped endpoints before explicit release and received Dead after normal
automatic collection; cleanup order is now corrected without ignoring failure.
UART CRLF decoding was corrected in the host serial adapter. The final run's
verdict binds success to initrd SHA-256
`7dccbac8c48596533ca1a05cc520e30496caf28db1e1f879fc4998aba32af3ba`.
No SMP or performance acceptance was run. Existing scenarios remain unchanged.
