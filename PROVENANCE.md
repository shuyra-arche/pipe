# Source snapshot and ownership

This repository extracts the pipe declaration, API, client, server, and owned
tests from https://github.com/anreonyr/sqware at
`f82ee2a9751b09563ca0f21577c212ab4470c8e4`.
The source authorship remains in that public Git history; this extraction does
not rewrite it or assert a new license. No project LICENSE file was present in
the source snapshot, and none is invented here.

| Source path | Provider ownership |
| --- | --- |
| `programs/src/service/pipe/api/` | `api/` |
| `programs/src/service/pipe/client/` | `client/` |
| `programs/src/service/pipe/core.rs` | server ledger and record modules |
| `programs/src/service/pipe/serve.rs` | server runtime and resource ownership |
| `programs/src/service/pipe/mod.rs` | server library entry |
| `programs/src/service/pipe/main.rs` | server ELF entry adapter |
| `crates/composition/src/units/pipe.rs` | `declaration/`, explicit `UNIT` |
| Shell `pipe_registry_authorizes_owner_and_keeps_roles_exclusive` | provider-owned host test, exact original assertions |

The initial byte-for-byte snapshot was nine files. Subsequent splitting keeps
one data structure per file, re-exports the existing public paths, and adapts
only repository boundaries, manifests and the single public execution entry.
Source owns the actual machine composition and selected deployment. This
provider does not read a Source global registry or depend on Source packages.

Stream wire and ring behavior remain supplied by the public stream provider at
`ebd1c6db4823e28591650d967d7e8c8dd1939687`; no second implementation is copied.
Existing opcodes, failure codes, codec quirks, limits, ownership, sticky role
reservations, half-close and recursive release semantics are preserved.

The independent integration fixture is newly authored test code. It uses the
fixed public Kernel, System, unit, execution and image-builder contracts. Its
firmware is an explicit, hashed public test input with source provenance,
not a new license assertion or a dependency on a local Source checkout.
