# Isolated Simnet AuxPoW relay follow-on

Prepared against draft #21 source `bd4f65851c0267f70aced3602f61391c5fdfc578`. The opt-in Simnet fixture seeds the chain with native work, then submits a block whose own native nonce is above target. Its synthetic one-transaction parent is mined with native kHeavyHash and commits to the exact child header. The test uses the ordinary mining RPC conversion and requires the receiving peer to select the relayed child. It applies no production configuration or activation changes.

This fixture covers the inherited auxiliary proof transport and validation path. KASCADE's final domain, parent interoperability with actual mining pools, and testnet identity are separate acceptance items. GitHub Actions has yet to run this candidate.
