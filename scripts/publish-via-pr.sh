#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"; cd "$root"
: "${DREAM_MACHINE_EXPECTED_SOURCE_SHA:?exact tested source required}"
: "${DREAM_MACHINE_PRODUCER_RUN_ID:?producer run id required}"
test "$(git rev-parse HEAD)" = "$DREAM_MACHINE_EXPECTED_SOURCE_SHA" || { echo 'source SHA mismatch' >&2; exit 1; }
test -z "$(git status --porcelain --untracked-files=all)" || { echo 'source must be clean' >&2; exit 1; }
remote="$(git remote get-url origin)"; owner_repo="$(printf '%s' "$remote"|sed -E 's#(git@github.com:|https://github.com/)##;s#\.git$##')"; owner="${owner_repo%%/*}"
test "$owner" = nicholas-ruest || { echo "PUBLISH_DENIED: personal owner required; got $owner" >&2; exit 1; }
branch="$(git branch --show-current)"; test "$branch" != main && test "$branch" != master || { echo 'PUBLISH_DENIED: feature branch required' >&2; exit 1; }
git push --porcelain origin "HEAD:refs/heads/$branch"
gh pr create --repo "$owner_repo" --head "$branch" --base main --title 'Capability Provenance Mesh: complete product' --body "Producer run: $DREAM_MACHINE_PRODUCER_RUN_ID
Exact tested source: $DREAM_MACHINE_EXPECTED_SOURCE_SHA
No merge or deployment is authorized."
gh pr view --repo "$owner_repo" "$branch" --json url,isDraft,headRefOid,state
