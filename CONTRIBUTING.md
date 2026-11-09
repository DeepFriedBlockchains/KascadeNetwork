# Contributing to KASCADE

KascadeNetwork is the primary public development repository. Contributions use
feature branches and pull requests.

## Send a change

1. Fork this repository and branch from the current public `main`.
2. Keep each change focused. Discuss changes to consensus, monetary policy or
   import and pruning behavior with maintainers before implementation.
3. Add regression coverage for consensus or database behavior that changes.
4. Run the relevant tests, formatting and Clippy. Include the commands and
   results in the pull request description.
5. Open a pull request. Maintainers review the code and the checks before merging.

A substantial proposal should begin with an issue describing the concrete
problem and expected behavior. Report security findings privately using
[SECURITY.md](./SECURITY.md).

## Validation

Install the dependencies in [docs/BUILDING.md](./docs/BUILDING.md), then run
the checks relevant to your change.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --tests --benches --examples -- -D warnings
cargo test --locked -p kaspa-consensus --lib
```

GitHub Actions runs the workspace tests, compilation, formatting, Clippy,
WASM checks and release builds. Document any skipped or failed tests.
Consensus and persistence changes also need targeted tests that exercise
the affected entry points, including failure and retry behavior.

## Review boundaries

Maintainers control repository administration, merging and releases.
Outside contributors can fork the public repository and submit pull requests.
Consensus-critical functionality stays inactive until validated and explicitly
authorized for activation.

Preserve existing monetary policy, network and cryptographic parameters unless
a maintainer explicitly approves a policy change. Keep unresolved parameters
unresolved. Separate mechanical changes from semantic changes.

Commit source and public development evidence. Credentials, wallet material,
private review reports and operational details belong in restricted channels.

## Licensing

Contributions use the repository's ISC license. Retain upstream attribution.
See [LICENSE](./LICENSE) and [NOTICE](./NOTICE).
