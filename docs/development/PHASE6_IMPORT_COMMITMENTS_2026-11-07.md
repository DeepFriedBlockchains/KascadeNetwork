# Phase 6 pruning import commitments · November 7, 2026

This inactive branch continues guarded import verification at `23fe1993616231da6099d6e8bd6fdec88bb8aa35`.
It uses the same pruning guards and the current processor's network parameters
to verify the committed SMT state at an active pruning point. A read-only
lane scan independently rebuilds the root and count, corroborates the branch
root, and checks the result against the pruning header's sequence commitment
and its parent and inactivity ancestry. Pre-Toccata points are distinguished
from missing evidence at an active point.

A separate shielded-state regression authenticates the selected child header
and body and checks the committed frontier and actual nullifier rows in an
isolated receiver database. The test covers altered body data and missing
membership rows. This is validation of the inactive seam.

The existing stack remains open. This branch requires CI. The next work is
to supply proof-authenticated points and child commitments to the guarded
processor path, bind shielded verification to that path, and thread the
authority token through every direct and staged import caller. Cancellation
and staging cleanup must preserve worker exclusion and release database
handles before deletion. Consensus activation remains gated.
