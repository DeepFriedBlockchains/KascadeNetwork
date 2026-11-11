# Phase 6 nonempty imports and abrupt restart validation

November 11, 2026

## Verified checkpoint

Draft [PR #9](https://github.com/DeepFriedBlockchains/KascadeNetwork/pull/9)
is stacked on PR #8 at `4f23ff9f77bed788085efe03441420e522ad9595`.
Implementation `110d1625763c9fecd2cf2a226e5e7fb25add99bf` on
`feature/phase6-payload-crash-recovery-20261111` passed
[all eight jobs in run 43068789717](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/43068789717).
The documentation-only continuation preserves that tested implementation.

## Completed implementation

The shielded fixture reconstructs a wallet from canonical accepted-chain
coinbase data, proves a real Orchard payment at a mature anchor after
Simnet's existing anchor fork, submits it through production RPC
admission and mines it below the pruning point. Source and recovered
receiver must export identical nonempty spent-nullifier sets.

A separate transparent coinbase fixture has nonempty pruning UTXOs.
Ten valid signed user lanes are refreshed every 50 mined blocks and
asserted present inside the actual pruning point's finality window.

Test-only gates pause after a real SMT batch write, nonempty authorized
UTXO append or verified atomic shielded seed. An explicitly armed SMT
case uses one-entry batches to expose partial RocksDB state before
root/metadata completion. Normal batching is unchanged. No database or
authority lock is acquired by the rendezvous. The UTXO done frame is
withheld until disconnect, so buffered frames cannot complete the
intended interrupted import.

Cancellation retains the outstanding native SMT writer's reservation,
drains workers, leaves the old active receiver at genesis, reopens the
same directory and requires a complete verified retry.

Three Unix parent tests run actual daemon services in a child process,
wait for the receiver's write checkpoint, send SIGKILL and check signal
9. The parent owns both persistent directories and reopens them after
the killed process exits. Before network start the receiver must have
no restored worker reservation, staging role or promotion marker and
must remain at genesis. Complete fresh authentication, verified
promotion, closed active staging role and post-IBD block acceptance are
then required. The child performs no graceful shutdown or DB close.

Shielded nullifiers are currently buffered and seeded atomically by
the existing importer. Its crash checkpoint is after that atomic seed
and before component completion, while overall import authority is
still incomplete. SMT exposes a true partially written multi-batch
tree; UTXO exposes an authorized append before the done frame.
These are process-termination tests, not machine power-loss tests.

## Observed validation

| Check | Result |
| --- | --- |
| Focused authority regressions | 21 passed; 1,583 excluded by filter |
| Regular workspace | 1,568 passed; 25 existing skips |
| No-asm suite | 32 passed |
| Formatting and strict Clippy | Passed |
| Workspace/no-asm doc tests, cargo doc, kip10 example | Passed |
| Check, Check WASM32, Check no_std | Passed |
| Linux release, WASM SDK release, WASM tests | Passed |

The six new cases all passed:
- `authority_integration_preview_interrupts_spent_nullifier_payload`
- `authority_integration_preview_interrupts_nonempty_utxo_payload`
- `authority_integration_preview_cancels_partial_smt_write`
- `authority_integration_preview_recovers_abrupt_smt_payload`
- `authority_integration_preview_recovers_abrupt_shielded_payload`
- `authority_integration_preview_recovers_abrupt_utxo_payload`

No assertion was removed or relaxed. The original descriptor and
restart regressions remain. Focused execution uses two test workers to
bound simultaneous real proof generation and daemon processes.

## Source and CI history

| Source | Change | Validation |
| --- | --- | --- |
| `ccb68579d2cdb4b4770bcf679e8776f45418dddf` | Fixtures, hooks and SIGKILL harness | Runs 43066407288 / 43066415215 superseded |
| `1c1042147e4b68b17460a46a85a55d7b961f71d0` | Hold UTXO done until disconnect | Check/WASM/no-std passed; run 43067275773 reported 13 Rustfmt hunks |
| `13a1e3de896eed9deb59e535f2431209be18b663` | Apply exact CI formatter diff | Formatting/Clippy and five jobs passed in 43067753790 before the fixture follow-up |
| `110d1625763c9fecd2cf2a226e5e7fb25add99bf` | Refresh and assert active fixture lanes | Run 43068789717 passed all eight jobs |

Push run 43068779387 was superseded by the green PR run. No
implementation run remains pending for this milestone.

## Next executable checkpoint

Continue from the latest PR #9 head, preserving tested implementation
`110d1625763c9fecd2cf2a226e5e7fb25add99bf`. Extend the production
tests to retain an authenticated proof receipt across promotion and
require actual `begin_verified_staging_import` calls on ordinary active
receivers to reject it before import, after activation and after
reopen. Keep the closed-role and durable-promotion assertions.

Further production work includes adversarial nonempty stream rejection
and bounded-memory shielded nullifier ingestion; the existing importer
still collects the full declared set before its atomic seed. Preserve
the count cap, duplicate detection, authenticated root and session
ownership if that implementation is changed.

All development and validation use GitHub and GitHub Actions. Local
Windows, WSL and existing checkouts are outside the workflow. No
private history or project-reference material was imported.
The preview remains default-off and Simnet-only. PRs #1–#9 remain
unmerged. No release or consensus activation is authorized.
