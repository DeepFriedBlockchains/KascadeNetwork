# Phase 6 development continuation · November 6, 2026

The public validation stack is PR #1 (`ba4ce3072bd7785e1b3061647f2dacbba044d8e6`),
PR #2 (`91da4d8f728fe147529420dbaaa5e628765fd8f3`) and
PR #3 (`ed4abfe3a105b45145e7449b91149b52544f6262`). PR #2 is based on PR #1 and PR #3 is based on PR #2.

Tests run [42225379635](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42225379635)
passed Check, Lints, Check no_std, Check WASM32, Test WASM32, Linux Release and
Build WASM32 SDK. The native test job stopped with 1,129 passes, three goref
coinbase payload failures, 24 skipped and 386 unrun. Run
[42215101257](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/42215101257) on PR #2
has the same seven passing job categories and a failed native test job.

The isolated regeneration candidate reuses the real block-template builder
with a test-only transparent coinbase profile and explicit DAG recipes.
The Python verifier independently hashes serialized headers and checks
parents, payload layout, spend and merge counts, DAA variation and the
pruning span. Rust replay checks consensus acceptance, DAA, canonical
genesis, pruning advancement and UTXO-index consistency. The archived
upstream fixtures remain inputs. The fixture names retain their historic
paths. The 265-block case contains 224 spends; the two 5,000-block cases
contain zero and 4,790 spends, respectively.

The first hosted regeneration and release replay are pending. Verify the
run, archive digests and targeted replay, then promote the candidate source
and generated fixtures to PR #3 and run the full stack CI. After that,
continue the database-bound authority wrapper on a public feature branch.
Consensus activation stays gated.
