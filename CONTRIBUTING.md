# Contributing to KASCADE

KASCADE is pre-release, unaudited, and under active development. Contributions are welcome,
with one structural caveat that you should read before you spend time on a patch.

## This repository is a snapshot, not the development branch

The public repository is a **curated export** of the private development tree. It has no
shared Git history with it. See [`PUBLICATION.md`](./PUBLICATION.md) for the mechanics.

The practical consequences:

- **Pull requests opened against this repository cannot be merged directly.** There is no
  common ancestor to merge from.
- A patch you send is reviewed, and if accepted it is applied in the private development
  repository, tested there, and appears in a later snapshot with the fix attributed to you.
- Your commit will therefore not appear in this repository's history under your name. If
  that matters to you, say so in the issue and we will credit you in the snapshot commit
  message and in `NOTICE` where appropriate.
- Force-pushing, rewriting this repository's history, or adding commits to it directly is
  not possible for outside contributors and should not be attempted.

If you want to work on something substantial, **open an issue first**. It is much cheaper
for both of us to agree on the approach before the code exists.

## What is useful right now

The project is not short of features to imagine. It is short of correctness work on what
already exists:

- **Consensus invariant tests.** The shielded state — nullifier set, note-commitment tree,
  anchor, turnstile supply accounting — has to stay consistent under GHOSTDAG's
  out-of-order block acceptance and under reorgs. Tests that try to break that are the
  single most valuable contribution available.
- **Crash-consistency work.** The pruning-point import path is known not to be reliably
  fail-closed after a restart. Reproductions and analysis are wanted; see the "known not to
  work" section of the README.
- **Differential testing against upstream.** KASCADE is a fork of `rusty-kaspa`. Where
  behaviour is supposed to be identical (proof-of-work, transaction mass, script
  evaluation), a test that proves it is still identical is worth having.
- **Documentation that is true.** Several documents in `docs/` describe a predecessor
  deployment rather than KASCADE. Corrections are welcome.
- **Build and packaging friction.** Windows cross-compilation, musl static builds, and the
  GPU backend all have rough edges.

## What is not useful right now

- New consensus features, new monetary rules, or new tokenomics. Those are being designed
  elsewhere and a patch here will not be considered.
- Rebranding. The `zkas`/`ZKMM` identifiers are consensus-visible; renaming them is a
  consensus change, not a cosmetic one.
- Dependency bumps with no stated reason.
- Large reformatting-only diffs.

## Ground rules for patches

1. **Do not change consensus-visible behaviour without saying so explicitly** in the issue
   and the patch description. That includes address prefixes, network magic, genesis,
   difficulty adjustment, emission, mass rules, and AuxPoW binding.
2. **Do not invent consensus constants.** If a value is unresolved, it stays unresolved and
   the code says so. Guessing a GHOSTDAG `k`, a pruning depth, a genesis hash or a treasury
   address is worse than leaving it marked as a placeholder.
3. **Add a test that fails before your fix.** For consensus and database code this is not
   optional.
4. **Keep the diff reviewable.** Separate mechanical changes from semantic ones.
5. **Match the surrounding style.** `cargo fmt` and `cargo clippy` before sending.

## Reporting bugs

Ordinary bugs: open an issue. Include the snapshot commit, what you ran, what happened, and
what you expected.

Security problems: **do not open a public issue.** See [`SECURITY.md`](./SECURITY.md).

## Licensing of contributions

By contributing you agree that your contribution is licensed under the ISC licence, the same
terms as the rest of the repository. See [`LICENSE`](./LICENSE) and [`NOTICE`](./NOTICE) for
the provenance of the upstream code your patch will sit next to.
