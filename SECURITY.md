# Security policy

## Status

KASCADE is **pre-release, unaudited, and under active development**. There is no public
network, no mainnet, and no supported deployment. The consensus, shielded-pool and wallet
code in this repository has not had an independent security review.

Assume the software is exploitable. Do not use it to custody value.

## Reporting a vulnerability

**Please do not open a public issue for a security problem.**

Report privately through either channel:

1. GitHub's private vulnerability reporting on this repository
   (*Security* → *Report a vulnerability*), or
2. Email the maintainers at the address listed in the repository's GitHub organisation
   profile.

Please include:

- the affected component and, where possible, the file and function;
- the commit or snapshot you tested;
- a description of the impact — what an attacker gains;
- reproduction steps or a proof of concept, if you have one;
- whether you intend to disclose publicly, and on what timeline.

## What to expect

This is a small project without a formal security team or an on-call rotation. Realistic
expectations:

- **Acknowledgement:** best effort, typically within a week.
- **Assessment:** we will tell you whether we believe the report is valid, and why.
- **Fix:** depends entirely on severity and on whether the affected code is active. Large
  parts of the consensus tree are currently inactive or incomplete; a bug in inactive code
  is still a bug, but it will not be fixed on an emergency schedule.
- **Credit:** we will credit you in the fix commit and in the release notes unless you ask
  us not to.

We will not pay a bounty. There is no bug bounty programme and no funding for one. Please
do not spend effort on the assumption that there is.

## Scope

In scope:

- consensus validation, block acceptance, and reorg handling;
- monetary issuance, reward calculation and supply accounting;
- the shielded pool: note commitments, nullifiers, anchors, and the turnstile invariant;
- proof-of-work validation, including AuxPoW / merge-mining binding;
- the pruning-point import path and its crash-consistency properties;
- wallet key handling, signing, and the wallet daemon's REST surface;
- the p2p layer's message parsing and resource limits.

Out of scope:

- anything requiring physical access to a machine;
- denial of service by sheer volume against a node you are not running;
- issues in third-party dependencies that are already publicly known and unfixed upstream
  (tell us anyway if KASCADE's usage makes the impact worse);
- the placeholder values in `consensus/core/src/config/params.rs`, which are known not to
  be KASCADE's final parameters.

## Known issues

The project does not maintain a public list of known security issues, because this
repository is a source snapshot rather than the development tracker. The `README.md`
"what is known not to work" section lists the currently acknowledged functional gaps,
including the unresolved import-session crash-safety problem. That list is not exhaustive
and its absence of an item is not evidence that the item is fixed.

## Safe harbour

We will not pursue legal action against researchers who:

- test only against their own local deployment built from this repository;
- do not access, modify or exfiltrate data belonging to anyone else;
- do not degrade any service that anyone else relies on;
- report privately and give us a reasonable opportunity to respond before disclosing.

There is no public network to test against, so in practice this means: build it locally,
break it locally, tell us what you found.
