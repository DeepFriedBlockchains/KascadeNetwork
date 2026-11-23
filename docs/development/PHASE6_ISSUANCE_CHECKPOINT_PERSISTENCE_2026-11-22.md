# Phase 6 test-only issuance checkpoint persistence

This extension of #38 writes a checkpoint and its readiness commitment to one RocksDB batch in an isolated temporary database. It checks a clean close/reopen before and after committing a replacement checkpoint. The old anchor remains valid after an unsubmitted batch. After replacement, the new anchor selects the complete committed checkpoint. A logically balanced modification of the paid/deferred split and a missing readiness record both fail to restore against the independently supplied anchor.

The fixture also checks canonical record length and bounded claim-count decoding. It is staged behind #38's focused tests and does not alter production storage, the wire format, consensus activation, or the emission schedule. Dropping an unsubmitted batch models the pre-commit case. Crash injection during a database write and authenticated pruning proof/child binding remain separate production gates. CI is deferred until the parent model completes.

## CI trigger

The parent passed all eight jobs on `82dfd474f5ce1e31112bc7f7eab00db42db42d21` in [Actions 45105132180](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45105132180). Trigger this draft’s full matrix on the unchanged staged fixture `cc5d73dd3628c3b1bb1cc96c901dbdf3722fd40d`.
