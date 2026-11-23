# Phase 6 issuance checkpoint recovery after process exit

November 23, 2026

This isolated fixture starts the same test executable as a child process. The child opens a temporary RocksDB and prepares the replacement batch. In one case it exits before submitting the batch. In the other it exits immediately after RocksDB returns success. The worker calls process exit without returning through RocksDB's normal shutdown. The parent checks the expected independent commitment and complete readiness/checkpoint pair after recovery. It also requires a unique exit code so a failed or undiscovered child test cannot masquerade as the intended boundary.

This tests process exit before and after a database call. It does not claim power-loss durability, crash during a database write, or real production pruning and settlement. It builds on #39 and is staged pending its validation. No production consensus or issuance feature is wired.

## CI gate

Parent `db7d221116af81a2b7cbba9f9646d760052a56f2` passed all eight jobs in [Actions 45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500). Validate the staged test source `e87401b6bca55a5684f513beb0fa893fed862d24` with the existing focused step and full matrix.
