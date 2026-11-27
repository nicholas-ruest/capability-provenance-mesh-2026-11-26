#!/usr/bin/env bash
set -euo pipefail

readonly owner='nicholas-ruest'
readonly repo_name='capability-provenance-mesh-2026-11-26'
readonly owner_repo="$owner/$repo_name"
readonly feature_branch='product/complete-candidate'
readonly remote_url="https://github.com/$owner_repo.git"

deny() { printf 'PUBLISH_DENIED: %s\n' "$*" >&2; exit 1; }

root="$(git rev-parse --show-toplevel)"
cd "$root"
: "${DREAM_MACHINE_SOURCE_SHA:?controller-bound source SHA required}"
: "${DREAM_MACHINE_PRODUCER_RUN_ID:?actual producer run id required}"
: "${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"

if [[ -n "${DREAM_MACHINE_PUBLISHER_INVOCATION_LOG:-}" ]]; then
  printf 'publisher-invoked\n' >> "$DREAM_MACHINE_PUBLISHER_INVOCATION_LOG"
fi

head_sha="$(git rev-parse HEAD)"
[[ "$head_sha" =~ ^[0-9a-f]{40}$ ]] || deny 'invalid candidate source SHA'
[[ "$head_sha" == "$DREAM_MACHINE_SOURCE_SHA" ]] || deny 'candidate differs from controller-bound source'
[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || deny 'candidate source must be clean'

base_sha="$(git rev-list --max-parents=0 HEAD)"
[[ "$base_sha" =~ ^[0-9a-f]{40}$ ]] || deny 'candidate must have exactly one source root'
[[ "$base_sha" != "$head_sha" ]] || deny 'review base cannot equal the complete candidate'
git merge-base --is-ancestor "$base_sha" "$head_sha" || deny 'review base is not an ancestor'
required_base_paths=(
  README.md
  Cargo.toml
  crates/cpm-contracts/src/lib.rs
  docs/specification.md
  docs/architecture.md
  scripts/verify-prototype.sh
)
for path in "${required_base_paths[@]}"; do
  git cat-file -e "$base_sha:$path" 2>/dev/null || deny "review base lacks substantive path: $path"
done
base_files="$(git ls-tree -r --name-only "$base_sha" | wc -l)"
base_bytes="$(git ls-tree -rl "$base_sha" | awk '{ total += $4 } END { print total + 0 }')"
(( base_files >= 20 && base_bytes >= 50000 )) || deny 'review base is an empty or README-only shell'

authenticated_owner="$(gh api user --jq '.login')" || deny 'unable to verify authenticated GitHub account'
[[ "$authenticated_owner" == "$owner" ]] || deny "authenticated account must be $owner"
verified_owner="$(gh api "users/$owner" --jq '.login')" || deny 'unable to verify target owner'
[[ "$verified_owner" == "$owner" ]] || deny 'target owner readback mismatch'

repo_exists=false
if repo_response="$(gh api --include --method GET "repos/$owner_repo" 2>&1)"; then
  repo_exists=true
else
  repo_status="$(printf '%s\n' "$repo_response" | sed -nE 's/.*HTTP ([0-9]{3}).*/\1/p' | tail -1)"
  [[ "$repo_status" == 404 ]] || deny 'cannot prove the fixed target repository is absent'
fi

if [[ "$repo_exists" == false ]]; then
  if ! gh api --method POST user/repos -f "name=$repo_name" \
      -f 'description=Source-bound capability admission and provenance workbench' \
      -F private=false -F has_issues=true -F auto_init=false >/dev/null; then
    gh api --method GET "repos/$owner_repo" >/dev/null 2>&1 ||
      deny 'repository creation failed and readback cannot resolve its state'
  fi
fi
repo_meta="$(gh api --method GET "repos/$owner_repo" \
  --jq '[.owner.login,.name,.visibility,.archived,.is_template]|@tsv')" ||
  deny 'repository readback failed'
IFS=$'\t' read -r repo_owner actual_name visibility archived is_template <<< "$repo_meta"
[[ "$repo_owner" == "$owner" && "$actual_name" == "$repo_name" ]] ||
  deny 'repository owner or name readback mismatch'
[[ "$visibility" == public && "$archived" == false && "$is_template" == false ]] ||
  deny 'repository must be an active public personal repository'

if origin_url="$(git remote get-url origin 2>/dev/null)"; then
  [[ "$origin_url" == "$remote_url" ]] || deny 'existing origin does not match the fixed personal target'
else
  git remote add origin "$remote_url"
fi

remote_ref() {
  git ls-remote --heads "$remote_url" "refs/heads/$1" |
    awk -v ref="refs/heads/$1" '$2 == ref { print $1 }'
}
remote_main="$(remote_ref main)"
remote_feature="$(remote_ref "$feature_branch")"
[[ -z "$remote_main" || "$remote_main" == "$base_sha" ]] ||
  deny 'remote main is not the substantive source foundation'
if [[ -n "$remote_feature" && "$remote_feature" != "$head_sha" ]]; then
  git merge-base --is-ancestor "$remote_feature" "$head_sha" ||
    deny 'remote candidate branch is not an ancestor of the repaired candidate'
fi

refspecs=()
[[ -n "$remote_main" ]] || refspecs+=("$base_sha:refs/heads/main")
[[ "$remote_feature" == "$head_sha" ]] || refspecs+=("$head_sha:refs/heads/$feature_branch")
if (( ${#refspecs[@]} )); then
  git push --atomic --porcelain "$remote_url" "${refspecs[@]}"
fi

remote_main="$(remote_ref main)"
remote_feature="$(remote_ref "$feature_branch")"
[[ "$remote_main" == "$base_sha" ]] || deny 'main ref readback mismatch'
[[ "$remote_feature" == "$head_sha" ]] || deny 'candidate ref readback mismatch'
default_branch="$(gh api --method GET "repos/$owner_repo" --jq '.default_branch')" ||
  deny 'default branch readback failed'
if [[ "$default_branch" != main ]]; then
  gh api --method PATCH "repos/$owner_repo" -f default_branch=main >/dev/null ||
    deny 'could not establish main as the review base'
fi
[[ "$(gh api --method GET "repos/$owner_repo" --jq '.default_branch')" == main ]] ||
  deny 'default branch is not main after readback'

find_pr() {
  gh api --method GET "repos/$owner_repo/pulls" -f state=open \
    -f "head=$owner:$feature_branch" -f base=main --jq '.[0].number // empty'
}
pr_number="$(find_pr)"
if [[ -z "$pr_number" ]]; then
  gh pr create --repo "$owner_repo" --head "$owner:$feature_branch" --base main \
    --title 'Capability Provenance Mesh: complete product' \
    --body "Producer run: $DREAM_MACHINE_PRODUCER_RUN_ID
Exact tested source: $head_sha
Review base: $base_sha
No merge or deployment is authorized." >/dev/null || true
  pr_number="$(find_pr)"
fi
[[ "$pr_number" =~ ^[0-9]+$ ]] || deny 'non-draft product PR was not created or recovered'

pr_meta="$(gh api --method GET "repos/$owner_repo/pulls/$pr_number" \
  --jq '[.html_url,.draft,.state,.head.sha,.head.ref,.head.repo.owner.login,.base.ref]|@tsv')" ||
  deny 'PR readback failed'
IFS=$'\t' read -r pr_url pr_draft pr_state pr_head pr_head_ref pr_owner pr_base <<< "$pr_meta"
[[ "$pr_draft" == false && "$pr_state" == open ]] || deny 'PR must be open and non-draft'
[[ "$pr_head" == "$head_sha" && "$pr_head_ref" == "$feature_branch" ]] ||
  deny 'PR head is not the exact controller-bound source'
[[ "$pr_owner" == "$owner" && "$pr_base" == main ]] ||
  deny 'PR owner or review base readback mismatch'
readback_path="$DREAM_MACHINE_EVIDENCE_DIR/publication-readback.json"
PUBLICATION_READBACK_PATH="$readback_path" \
PUBLICATION_OWNER_REPO="$owner_repo" \
PUBLICATION_BASE_SHA="$base_sha" \
PUBLICATION_SOURCE_SHA="$head_sha" \
PUBLICATION_FEATURE_BRANCH="$feature_branch" \
PUBLICATION_PR_NUMBER="$pr_number" \
PUBLICATION_PR_URL="$pr_url" \
node --input-type=module -e '
  import { writeFileSync } from "node:fs";
  const e = process.env;
  const receipt = {
    schema_version: 1,
    owner_repo: e.PUBLICATION_OWNER_REPO,
    base: { ref: "main", sha: e.PUBLICATION_BASE_SHA },
    candidate: { ref: e.PUBLICATION_FEATURE_BRANCH, sha: e.PUBLICATION_SOURCE_SHA },
    pull_request: { number: Number(e.PUBLICATION_PR_NUMBER), url: e.PUBLICATION_PR_URL, draft: false, state: "open" }
  };
  writeFileSync(e.PUBLICATION_READBACK_PATH, JSON.stringify(receipt, null, 2) + "\n");
'
cat "$readback_path"
