# Pipe independent integration

Provider-owned, reproducible integration fixture for the extracted pipe service.
It is a separate Cargo workspace (`declaration`, `root`, `probe`, `host`) and
does not read Source composition, runtime or paths. The Kernel is the fixed
public gitlink `components/kernel` (`3aa0a155...`), excluded from this workspace.

## Images and deployment

Three images: `pipe` (User), `pipe-root` (Supervisor entry), `pipe-probe`
(User). The probe image is reused by two explicit UnitFiles:

| UnitFile | image | role arg |
| --- | --- | --- |
| `pipe-probe` | `pipe-probe` | `0` parent |
| `pipe-peer` | `pipe-probe` | `1` peer |

`DECLARATIONS` therefore has four UnitFiles; `DEPLOYMENT` start is
`["pipe", "pipe-probe", "pipe-peer"]`, with the same three members in the scene
target and bootstrap `[Operator, Identity]`. Both leaves are declared User
Transient Units after the pipe service's ready signal; `pipe-root` is the
already-running Supervisor and is never assembled.

Every ELF uses public `execution`/`mold` startup and linker support; no local
runtime, Source ELF or private linker is used. `BUILDS` still has three image
coordinates.

## Host CLI, RV, firmware

`integration/host` is std-only and never runs QEMU, builds the Kernel or
downloads firmware. `check` validates manifests plus public
`image_inventory`/`startup_order`; `build` calls public `image_builder::build_with`
with `DECLARATIONS`, `DEPLOYMENT` and `BUILDS`. Native commands remain Root-owned;
the RV target is `riscv64gc-unknown-none-elf`, profile `release`.

`firmware-input.json` records the Root-supplied public firmware source
(`SBI.bin` at Source `f82ee2a9...`), sha256
`c53d58aa9ab17f95fa1af159f0bf05fad5c5622ebc0f1084463435fa6e5752b6`, 277880
bytes, PLIC class `sifive,plic-1.0.0`. The source snapshot has no license file;
no license is invented or granted.

## Declared Units and typed test channels

Both leaves are launched by static activation hooks from the running Root/System;
`env::unit::sire()` is that trusted task, and each leaf requires a nonzero sire
and opens its own public operator session. No runtime-thread harness is used.

Root's public `Fixture` `SupplyHook` creates marked Hole resources and ships
least-privilege `Policy::NONE` views: a FETCH-only context per leaf (typed
`Context{parent,peer}`), report writer STORE to peer / reader FETCH to parent,
and command writer STORE to parent / reader FETCH to peer. Each receiver claims
its unique token and verifies actual owner/vestor/permission, exact frame length
and `sender == sire`. `Context`, `Report` and `Command` live one per file in the
declaration crate. Actual endpoint seeds are still installed by the server in
their target's table; messages carry only descriptors.

## Coverage

Invalid and repeated failed create then success; peer creates pipe A and binds
Read to the parent; parent challenges A's unreserved WRITE direction by public
typed RPC and requires exact `Denied`; parent imports its legitimate A Read seed.
Parent owns pipe B, binds Read to self and Write to peer, with exclusive-role
denials; peer writes exact 40 bytes at capacity 16, closes, and requires exact
`Closed` on a further write; parent reads exact payload/wrap/EOF. A separate real
same-task pair proves exact `BrokenPipe`. Invalid-direction `Invalid`,
unknown-id `Dead`, invalid-target denial, and the fresh create/release checks
remain. Parent releases B and checks its seed dead; peer checks its B write seed
dead and reports before exiting named zero. Peer intentionally abandons A's owner
lease; parent retains its A endpoint and proves Kernel `raw::alive` dead, server
typed RPC exact `Dead` (pre-death wrong-owner was `Denied`) and Port read exact
`Dead`, before a fresh create/release.

### Markers (serial, one line each)

```text
PIPE: create-invalid
PIPE: create
PIPE: actors parent=<decimal> child=<decimal>
PIPE: ownerauth-denied distinct-unit
PIPE: error-invalid
PIPE: error-dead
PIPE: bind-read
PIPE: bind-write
PIPE: transfer
PIPE: close
PIPE: error-broken-pipe
PIPE: release
PIPE: error-closed
PIPE: owner-death
PIPE: cleanup
PIPE-INTEGRATION: PASS
```

Only the parent prints PASS, strictly last on success. Peer failure prints
`PIPE: peer FAIL code=<n>`, sends an explicit failure report and returns a
nonzero named Report; it never prints PASS. Parent failure prints
`PIPE: FAIL <step>` and `PIPE-INTEGRATION: FAIL`.

## Completion proof split

Root mints/embarks `pipe` first (ready), then mints both leaves, publishes their
real TaskIds, and embarks both; the hook ships channels before execution. Root
observes the parent's native `join_task` result (`Err` fails), then
`settle`/`supervise` and the supplementary typed terminal view. It prints only
`PIPE-ROOT: SHUTDOWN_OK` with a named zero Report.

The independent judge uses Kernel3aa's existing semihosting Spawn/Exit/Reap/Fault
records to account for every task. Root/parent/peer have named zero exits, and
the two System bootstrap tasks have native zero Exit records. The resident
pipe follows the existing System-controlled Ruin retirement: require its native
Reap and explicit System retirement, rather than pretending it naturally
returned zero. That path does not emit a numeric exit reason; report it as
unobserved. No production shutdown behavior is changed for the test.

The semantic PASS marker must be final, typed shutdown must complete, Kernel
must halt naturally, and nonzero before==after frame counts must match within
each 4/8 run. Faults, timeout/kill, missing task accounting or failed semantic
checks fail acceptance. Serial alone omits unnamed successful exits and cannot
establish the complete task accounting.

## Limits

The 5 s client calls are unchanged SDK waits. The parent's 30 s owner-death phase
bound and Root's 90 s observation budget are new harness bounds; qtest keeps its
unchanged 60 s per-test cap. No timeout was increased and no kill/timeout counts
as cleanup success.
