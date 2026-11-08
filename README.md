# KASCADE

**Status: unfinished. Under active development. No public network. Not audited.**

KASCADE is an experimental Layer-1 BlockDAG blockchain. It is a fork in the lineage
`rusty-kaspa` → `zkas-rusty` (ZKas) → KASCADE. It is **not** a finished product, it is
**not** running a public testnet, and it has **no mainnet**.

If you are looking for a network to use, mine on, or hold value in, this is not it yet.
Nothing in this repository is a release, and no binary built from it should be pointed at
anything of value.

---

## What this repository is

This is the **public source snapshot** of KASCADE's node and tooling: the node, the miner,
the wallet daemon, the explorer API, the shielded-state libraries, and the WASM/SDK
bindings.

It is deliberately **not** the project's development history. The private development
repository carries the full commit history, the engineering handoffs, the internal design
notes, and the security-review material. This snapshot is a curated, history-free export of
the source tree at a single reviewed point, published so that development can be followed
and reviewed without exposing operational material. See
[`PUBLICATION.md`](./PUBLICATION.md) for exactly how the snapshot is produced and how it is
kept in step with private development.

## What KASCADE is trying to be

| | KASCADE (design target) | Kaspa (upstream) |
|---|---|---|
| Consensus | GHOSTDAG BlockDAG | GHOSTDAG BlockDAG |
| Privacy | Optional shielding: transparent transactions by default, with user-selected shielded transactions (proposed Orchard/Halo 2, no trusted setup) | transparent |
| PoW | kHeavyHash, byte-identical to Kaspa | kHeavyHash |
| Merged mining | AuxPoW dual acceptance | — |
| Emission | cascading emission, see `KASCADE-SPEC.md` | fixed cap |

**These are design targets, not claims of implemented or activated features.** KASCADE's intended privacy model is optional shielding, not mandatory or shielded-by-default transactions. `KASCADE-SPEC.md` is a design document describing intent,
not a description of implemented behaviour. Where the specification and the code disagree,
the code is what exists and the specification is what is being argued about. Several
consensus parameters in this tree are still inherited from the predecessor chain and have
not been re-derived for KASCADE; do not read any constant in
`consensus/core/src/config/params.rs` as a KASCADE decision.

## What is known not to work

Stated plainly, because it is the honest description of the current tree:

- The monetary and issuance rules are **not finalised** and are not activated. The supply
  cap in code does not match the cap in the specification.
- The Phase 6 "inactive monetary consensus" work — branch authority, import sessions,
  lease/generation guarding, guarded commits — is **in progress and incomplete**. It is
  present in this snapshot only to the extent it has landed on the exported branch.
- Import-session crash safety is **known to be unresolved**: an interrupted pruning-point
  import does not reliably leave the node fail-closed after a restart.
- The shielded-state invariants under out-of-order GHOSTDAG block acceptance have **not**
  had an independent security review.
- KRN, KCF and the development treasury described in the specification **do not exist in
  this code**.
- No genesis, network magic, DNS seeder, treasury address or AuxPoW chain ID has been
  finalised for KASCADE. Placeholders are marked as such.

## Build

Prerequisites and the full build/test procedure are in
[`docs/BUILDING.md`](./docs/BUILDING.md). In short, a recent stable Rust toolchain, `protoc`,
and `clang`/`cmake` for the RocksDB and shielded dependencies.

```bash
cargo build --release
cargo test --release
```

Expect a long first build. Expect some tests to be skipped. Do not expect the test suite to
constitute a security argument.

## Run (local only)

There is no public network to join. A node built from this tree can be started against a
local regtest/simnet configuration for development:

```bash
./target/release/kaspad --simnet --appdir=./kascade-simnet --rpclisten=127.0.0.1:16810 --utxoindex
```

Keep RPC bound to loopback. See [`docs/NODE.md`](./docs/NODE.md) for node modes and flags,
and [`docs/WALLETD.md`](./docs/WALLETD.md) for the wallet daemon.

## Security

**There has been no security audit of KASCADE.** Not of the consensus changes, not of the
shielded pool, not of the wallet. The predecessor chain's shielded design was also never
independently reviewed.

This is safety-critical financial software written by a small team, in flux. Assume there
are exploitable defects. Do not use it to custody value.

If you find something, please report it privately rather than opening a public issue — see
[`SECURITY.md`](./SECURITY.md).

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Because this repository is a curated snapshot
rather than the development branch, **pull requests against it cannot be merged directly**.
Discussion and patches are welcome; accepted changes are applied in the private development
repository and appear here in a later snapshot. See [`PUBLICATION.md`](./PUBLICATION.md).

## License and provenance

KASCADE is derived from `rusty-kaspa` (Kaspa developers, ISC) via `zkas-rusty` (ZKas).
Upstream copyright notices are retained in [`LICENSE`](./LICENSE), and the full provenance
and attribution statement — including which parts of this tree originate upstream and under
what terms — is in [`NOTICE`](./NOTICE).

KASCADE's own modifications are released under the same ISC terms unless a file states
otherwise.
