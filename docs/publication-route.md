# Governed personal publication route

The only authorized first-publication command is the pinned controller with the
product publisher as its child:

```bash
node /home/ruv/.local/share/dream-machine/controllers/4d56f2a7cd1df352151ecfdb82fe791c0285e9e8/dream-machine-publication-gate.mjs \
  --publish /absolute/path/to/clean-candidate /absolute/path/to/external-evidence \
  -- bash scripts/publish-via-pr.sh
```

The controller supplies the exact candidate SHA as
`DREAM_MACHINE_SOURCE_SHA`. The publisher refuses a dirty checkout, any other
SHA, a missing real producer identity, or a target owner other than
`nicholas-ruest`.

## First repository shape

The fixed target is
`nicholas-ruest/capability-provenance-mesh-2026-11-26`. The publisher verifies
the authenticated account and target owner before a write. It creates that one
public repository only when a readback proves HTTP 404; an uncertain create is
resolved by reading the same fixed target, never by choosing another name.

`main` receives the repository's original 31-file, approximately 98 KB
foundation commit. That commit contains the workspace contract, real Rust
source, specification, architecture, and pinned verifier set, so it is neither
an empty branch nor a README-only shell. The controller-bound complete source is
pushed to `product/complete-candidate`. Missing refs are pushed atomically.
Existing refs must already match those exact SHAs.

The publisher opens or recovers one non-draft PR from the exact candidate branch
to `main`. It never merges, deploys, changes permissions, provisions paid
resources, or writes to an organization. Remote refs, repository owner/default
branch, PR owner/base/head SHA, draft status, and state are read back into the
external `publication-readback.json`.

## Local proofs

`tests/publication/publisher-route.test.sh` uses local Git and GitHub doubles to
prove first creation, substantive base selection, exact-source PR binding, and
an idempotent retry with no extra writes.

`tests/publication/negative-control.sh` invokes the unchanged real controller
with a one-package candidate and the real publisher command. It requires
`PUBLISH_DENIED`, `publication_attempted: false`, no publisher invocation,
and zero GitHub or Git-push writes. Neither test contacts GitHub.
