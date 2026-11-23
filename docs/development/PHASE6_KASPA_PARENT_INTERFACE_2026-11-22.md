# Kaspa parent template interface for Phase 6

November 22, 2026

The first external interoperability fixture should use a pinned Rusty Kaspa release on a separate isolated Simnet. The upstream gRPC contract exposes `GetBlockTemplate` with the parent-network payout address and a UTF-8 `extraData` field. It returns the full block template and the node's sync status. `SubmitBlock` reports a node-level outcome independently of KASCADE's response. This is a direct parent-node baseline. A pool interface needs its own test once a pool implementation is selected. [Kaspa RPC schema](https://github.com/kaspanet/rusty-kaspa/blob/master/rpc/grpc/core/proto/rpc.proto)

## Coinbase budget

The current KASCADE witness embeds a four-byte prefix and the child hash as 64 lowercase ASCII hex characters, for 68 bytes before any pool data. Kaspa's upstream Simnet parameters cap the total coinbase payload at 204 bytes. Its serializer counts 19 fixed bytes before the payout script and extra data. In that profile the remaining budget is therefore `117 - payout_script_bytes - other_extra_data_bytes`. Calculate this from the actual returned template and the pinned parent parameters. Include any node-added bytes in the measurement. Keep the commitment domain decision separate from this budget check. [KASCADE witness source](https://github.com/DeepFriedBlockchains/KascadeNetwork/blob/3aae9a87e353b2a164fadc77528b0871500e34ee/consensus/core/src/auxpow.rs) · [Kaspa coinbase parser](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/src/processes/coinbase.rs) · [Kaspa network parameters](https://github.com/kaspanet/rusty-kaspa/blob/master/consensus/core/src/config/params.rs)

## Next executable fixture

1. Obtain one KASCADE child template and one parent block template carrying its commitment via the upstream RPC. Assert that the marker and child hash survive the template response. Record total payload bytes and test the reject boundary with the chosen payout script.
2. Build the leaf-zero witness branch from the returned parent transactions using the parent's matching transaction Merkle algorithm. Require a fixture with multiple transactions and compare the reconstructed root to the original header. Preserve the parent template's compact target and consensus fields.
3. Verify the mined work independently against the child and parent targets. Exercise a child-only result and a dual result under the isolated profiles. Require the expected submit result on each node and a selected-tip check where acceptance is expected.
4. Rotate templates while mining and check that results from an expired pair cannot be submitted as the current job. Record the parent release, both network configurations, template identities, and exact job and test results.

The isolated fixture still needs implementation and validation. Keep the Phase 6 preview default-off and Simnet-only, and keep shielded consensus activation disabled.
