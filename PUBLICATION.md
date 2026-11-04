# How this public snapshot is produced

This repository is a **curated export**, not a development branch. It has no shared Git
history with the private development repository. That is deliberate: the private history
contains credentials that were committed and later removed, internal security-review
documents, operator runbooks, incident notes and build logs. None of that can be published,
and none of it can be reliably removed from a history that is already deep — so the public
tree is built fresh instead of being filtered.

## The rule

> The public repository is an **export artefact**. It is generated from a private commit by
> a reviewed, repeatable process. It is never edited directly and never merged into.

## Why not just filter the private history

`git filter-repo` on the private repository would have to rewrite every commit, invalidating
every clone and every piece of evidence the private repo currently holds. The project's
recovery evidence — including the record of what was exposed and when — lives in that
history. Rewriting it destroys the audit trail. A fresh export keeps both: the private
history stays intact and forensically complete, and the public tree is clean by
construction.

## Export procedure

Performed from a private workstation with read access to the private repository.

1. **Freeze a source commit.** Choose the private commit to export. It must be a commit on
   the approved branch whose CI is green. Record the SHA.

2. **Export the tree only.** No history, no remotes, no reflog:

   ```bash
   git archive <PRIVATE_SHA> | tar -x -C /tmp/kascade-export
   ```

3. **Apply the exclusion list.** Delete the paths named in the exclusion section of the
   private `PUBLICATION_SANITIZATION_MANIFEST.md`. That list is deliberately not published:
   the names of the excluded files are themselves information.

4. **Apply the sanitization rules.** Run the sanitization pass, which is deterministic and
   idempotent. It replaces operator filesystem paths, live infrastructure hostnames and
   example key material with placeholders or environment-variable lookups. Every rule is
   recorded in the private `PUBLICATION_SANITIZATION_MANIFEST.md` with a rule ID, a
   description and a match count.

5. **Regenerate the public-only files.** `README.md`, `NOTICE`, `SECURITY.md` and this file
   are authored for the public tree and are not present in the private tree.

6. **Verify before committing.** Scan the staged tree *and* the resulting history for
   credentials, internal documents, infrastructure details and confidential files. A clean
   working-tree scan is not sufficient — the commit itself must be scanned, because the
   commit is what gets published.

7. **Commit as a single root commit.** The public repository's history starts at one commit
   with no parents. Its message names the private source SHA so the export is traceable
   without publishing the private history:

   ```
   KASCADE public source snapshot

   Exported from private commit <PRIVATE_SHA>.
   No shared history with the private development repository.
   ```

8. **Push to the private publication repository first.** It stays private until the project
   owner approves publication. A fresh clone of the pushed repository is then scanned again,
   because what matters is what is actually reachable on the remote, not what is on the
   exporting machine.

## Keeping the public repository up to date

Updates are **whole-snapshot replacements**, not incremental merges. This is intentional:
incremental merges would reintroduce shared history and make it possible for a private
commit to leak by accident.

For each release of the snapshot:

1. Pick the next approved private commit.
2. Re-run steps 2–6 above from scratch into an empty directory.
3. Commit the result on the public repository's `main` as an ordinary commit whose message
   names the new private SHA and summarises the user-visible changes.
4. Verify the new commit and its tree (step 6) before pushing.
5. Tag the snapshot if it corresponds to a milestone, e.g. `snapshot-2026-11-04`.

Because each snapshot is generated from a clean export rather than rebased onto the previous
one, the public history accumulates one commit per snapshot and never contains private
commits. Reviewers can diff consecutive snapshot commits to see what changed between
exports.

## What must never appear in a snapshot

- Private keys, seed phrases, wallet material, API tokens, credentials of any kind.
- Internal security documents, audit findings, threat models, review notes.
- Engineering handoffs, agent instructions, status files, research notes.
- Operator runbooks, incident write-ups, capacity notes, recovery scripts.
- Build logs, CI logs, or any file containing a machine-specific path.
- Live infrastructure: node IP addresses, seeder hostnames, explorer/wallet/pool hostnames,
  internal service names.
- Anything that identifies a person, a machine, or a hosting provider.

## If something leaks anyway

Do not `git commit --amend` and do not force-push. Treat the leaked value as compromised:
rotate it first, then decide how to clean the history. A rewritten history does not
un-publish a secret that was already fetched.

## Reviewing a snapshot

A reviewer should be able to answer, from the public repository alone:

- What is this project, and what state is it in? (`README.md`)
- Where did the code come from, and under what licence? (`NOTICE`, `LICENSE`)
- How do I build and test it? (`docs/BUILDING.md`)
- How do I report a vulnerability? (`SECURITY.md`)
- Is this the development history? (No — this file)
