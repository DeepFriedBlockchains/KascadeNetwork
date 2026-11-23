# Phase 6 resource sample for the paced sustained Simnet run

This fixture continues the existing #33 experiment and adds Linux process snapshots around the measurement window. User and system CPU-time deltas are normalized with the runner's clock tick frequency. Start/end RSS and lifetime peak RSS refer to the shared test process containing all three daemons. The peak includes startup, and the samples cannot attribute memory to individual peers or establish a capacity limit. The existing 1,500-block, rate and final-tip assertions remain. The artifacts add raw resource snapshots and combined CPU and memory measurements.

This run uses simulated PoW and transparent empty-template traffic. A budgeted run separating node processes, fault schedules, and native or representative AuxPoW load remains a separate acceptance gate. CI is staged behind the current #33 source.

## CI trigger

The parent passed all eight jobs on `e48bf578e32eecbd40b6db2f7e596899c0c370e7` in [Actions 45104425195](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45104425195). Trigger this draft’s full matrix on the unchanged staged fixture `89f81b1c2b719c0223d11acbaaf5187613f5691c`.
