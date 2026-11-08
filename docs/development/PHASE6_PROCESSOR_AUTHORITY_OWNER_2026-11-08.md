# Phase 6 processor-owned database authority

November 8, 2026

The inactive virtual processor adapter now constructs and holds one
authority for its database. Callers pass opaque tokens, worker permits
and leases. The processor performs authorization with its own owner,
while retaining pruning session, point and metadata guards in that
order. It can close readiness before a pruning point has been installed.

The guarded UTXO regression creates a second owner against the same
RocksDB handle. Its permit cannot append rows or change the accumulator
through the processor. While that permit is outstanding, the processor
cannot start a competing session. After release, its own session can
complete verification, issue a lease and commit scoped data.

This candidate descends from `c478129fb1027102b601783beb8de6c719cda7f4` and remains inactive.
Hosted validation is pending. The next work is the production import
ownership boundary and verified restart for databases with descendants.
