# Phase 6 selected-child issuance commitment fixture

November 23, 2026

This test composes the real selected-child body commitment verifier with #39's RocksDB reopen fixture. It checks the authenticated selected-parent relationship and body Merkle root, then uses the verified commitment to restore the persisted issuance fixture. A wrong parent, changed child body, and a changed paid/deferred split each fail the corresponding check.

The test stores an illustrative commitment in the existing shielded commitment slot solely to exercise the verification path. It adds no production commitment field, issuance metadata, network behavior or activation. The trusted child header and GHOSTDAG row are seeded explicitly. Deriving that trust from pruning ancestry, committing the future versioned issuance field, and linking it to real settlement and promotion remain separate production gates. This draft is staged pending #39 validation.

## CI gate

Parent `db7d221116af81a2b7cbba9f9646d760052a56f2` passed all eight jobs in [Actions 45136232500](https://github.com/DeepFriedBlockchains/KascadeNetwork/actions/runs/45136232500). Validate the staged test source `b997d82bc6e72384a5ba747485812698cce425dd` with the existing focused step and full matrix.
