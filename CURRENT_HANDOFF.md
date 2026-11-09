# KASCADE public development handoff

Updated November 6, 2026.

## Published checkpoint

Public main begins at reviewed root `62248169f0920277b3ee60cf275080c221a21786`.
Its tree is `50165604d2f1d7ead1efe0b1e9a4d869ebe60753`, with 1,494 files
and no parent commits. Anonymous Git access was verified.

## Current work

This feature branch updates contribution and build instructions for public
development and preserves the existing GitHub Actions checks. It adds manual
workflow dispatch, cancellation of superseded branch runs and the tested
native test resource limits. Runtime source is unchanged by this increment.

The inactive monetary authority and database-readiness work is being prepared
as a separate source migration with its associated regression tests.
Public CI must validate that resulting source before integration.

## Validation and next action

The publication root's Actions run is in progress. This branch requires its
own Actions validation. Documentation paths and repository URLs have been
checked against the current source. No new Rust test result is claimed here.

Review this pull request, establish required checks on main, and continue
the reviewed source migration. Consensus functionality remains inactive.
