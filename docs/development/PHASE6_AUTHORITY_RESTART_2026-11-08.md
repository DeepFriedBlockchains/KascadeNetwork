# Phase 6 authority restart and session exclusion

November 8, 2026

The reviewed PR stack remains open. PR #3 at `f64839c` passed all eight jobs
in [run 42348096245](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42348096245).
The inactive authority milestone at `a7618bf` and guarded import code at
`23fe199` also passed all eight jobs. This branch continues the existing
guarded UTXO candidate at `4c24c70`. The short recovery ref `1dc979a`
did not resolve in KascadeNetwork.

## Changes

- A persisted READY marker grants authority only after successful
  corroboration for the current open database and the exact committed
  revision. The same check rejects a replacement marker for the same point.
- A failed import write invalidates its token and keeps the session reserved
  until the outstanding token or worker is released. The marker stays closed.
- The shielded import regression derives the expected nullifiers from every
  action in the actual accepted Orchard bundle, including padding, and checks
  the committed set at the selected point. The missing-row check is retained.
  The fixed-count assertion was the only failure in
  [run 42405344527](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42405344527).

The authority and import integration remain test-only. Proof authentication, guarded shielded completion,
and end-to-end caller migration remain open.

## Verified validation

[Run 42489324499, attempt 2](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42489324499/attempts/2)
completed successfully on `433c10f7d65cac9d4ee6ec861c4e777fb6554107`. All eight jobs
are green. Native nextest passed 1,549 tests with 25 skipped, and 32 no-asm
tests passed. Attempt 1 had one abrupt-writer subprocess exit without
the expected status. The identical code passed that test on retry and
on the later authenticated import and staging drain candidates.
