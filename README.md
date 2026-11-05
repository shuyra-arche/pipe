# Pipe

The dynamic Vanllia model discovers `server/unit.toml` directly. A new Rust
`tests/roundtrip` guest and its `test.toml` exercise ring wraparound, EOF and
explicit endpoint/pipe release. Its successful result is native exit zero;
System and the host runner verify exit receipts and reclamation. It prints no
PASS marker and does not depend on kernel event logging. Existing scenarios are
not migrated. See [validation/DYNAMIC-UNITS.md](validation/DYNAMIC-UNITS.md).

Public pipe declaration, wire API, client and resident server extracted from
Shuyra Source. See [PROVENANCE.md](PROVENANCE.md) for the frozen source, ownership
map, attribution and license absence.

| Package | Responsibility |
| --- | --- |
| `pipe-declaration` | Pure `UNIT` and owned image `BUILD` coordinates |
| `pipe-api` | Existing marks, opcodes, failure codes and exact wire frames |
| `pipe-client` | Client requests, owner leases and mapped stream ports |
| `pipe-server` | Resident server; binary `pipe` |

Consumers own their actual composition and selected Deployment. Integration
consumers prefix `BUILD.manifest` with the provider's checkout location; image,
Cargo package, binary, features, target and profile remain separate coordinates.
The server uses public execution/mold for its single startup, panic, allocator
and linker implementation. Stream remains a public fixed-revision dependency.

A pipe's creator owns its lease and authorizes endpoint binding. Read/write
reservations last for the pipe lifetime. CLOSE is sticky half-close, not RELEASE
or permission to rebind a reserved direction. Endpoint death closes its
direction; explicit release or owner-lease death revokes the root through the
existing Kernel mechanism. An admitted request with a lost reply may already
have committed; BIND has no new idempotence guarantee. The caller retains
cleanup responsibility for its owned lease on failure.

Build the server with the pinned toolchain:

```sh
cargo build --locked -p pipe-server --bin pipe --target riscv64gc-unknown-none-elf --release
cargo test --locked --manifest-path tests/Cargo.toml --target x86_64-unknown-linux-gnu
```

The host tests project only this provider's own pure ledger sources; no Source
path or second implementation is used. They include the original Shell ledger
case, wire golden bytes and declaration facts. After normal dependency cache
preparation, repeat with `--offline`.

The separate [real integration](integration/README.md) owns its explicit
declarations, selected deployment, Root/probe ELFs and host image CLI. It uses
fixed public Kernel/System/image-builder versions and a hashed public firmware
input. Kernel is a direct fixed gitlink, so clone with `--recurse-submodules`.
Host tests alone do not establish runtime cleanup: acceptance requires actual
semantic and native-exit evidence, natural shutdown and frame equality at both
4 and 8 Harts. Validation results belong to the associated Source migration
record; no success is implied by these instructions.
