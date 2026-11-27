#!/usr/bin/env bash
set -euo pipefail

product_root="$(git rev-parse --show-toplevel)"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
candidate="$fixture/candidate"
evidence="$fixture/evidence"
fake_bin="$fixture/fake-bin"
state="$fixture/state"
mkdir -p "$candidate/crates/cpm-contracts/src" "$candidate/docs" \
  "$candidate/scripts" "$evidence" "$fake_bin" "$state"

printf '%s\n' '# Capability Provenance Mesh foundation' > "$candidate/README.md"
printf '%s\n' '[workspace]' 'members = ["crates/cpm-contracts"]' > "$candidate/Cargo.toml"
printf '%s\n' '[package]' 'name = "cpm-contracts"' 'version = "0.1.0"' \
  'edition = "2024"' > "$candidate/crates/cpm-contracts/Cargo.toml"
printf '%s\n' 'pub struct Contract;' > "$candidate/crates/cpm-contracts/src/lib.rs"
printf '%s\n' '# Specification' > "$candidate/docs/specification.md"
printf '%s\n' '# Architecture' > "$candidate/docs/architecture.md"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' > "$candidate/scripts/verify-prototype.sh"
chmod +x "$candidate/scripts/verify-prototype.sh"
for index in $(seq 1 18); do
  head -c 3000 /dev/zero | tr '\0' x > "$candidate/docs/reference-$index.md"
done

git -C "$candidate" init -q -b candidate
git -C "$candidate" config user.name 'Publication route test'
git -C "$candidate" config user.email 'publication-route@example.invalid'
git -C "$candidate" add .
git -C "$candidate" commit -q -m 'substantive review foundation'
base_sha="$(git -C "$candidate" rev-parse HEAD)"

cp "$product_root/scripts/publish-via-pr.sh" "$candidate/scripts/publish-via-pr.sh"
printf '%s\n' 'complete candidate' > "$candidate/product.txt"
git -C "$candidate" add scripts/publish-via-pr.sh product.txt
git -C "$candidate" commit -q -m 'complete product candidate'
source_sha="$(git -C "$candidate" rev-parse HEAD)"
real_git="$(command -v git)"
printf '%s\n' '#!/usr/bin/env bash' \
  'if [[ "$1" == ls-remote ]]; then' \
  '  ref="${@: -1}"; name="${ref##*/}"; file="$FAKE_GITHUB_STATE/ref-${name//\//_}"' \
  '  name="${ref#refs/heads/}"; file="$FAKE_GITHUB_STATE/ref-${name//\//_}"' \
  '  [[ -s "$file" ]] && printf "%s\t%s\n" "$(cat "$file")" "$ref"' \
  '  exit 0' \
  'fi' \
  'for arg in "$@"; do' \
  '  if [[ "$arg" == *":refs/heads/main" ]]; then' \
  '    printf "%s" "${arg%%:*}" > "$FAKE_GITHUB_STATE/ref-main"' \
  '  elif [[ "$arg" == *":refs/heads/product/complete-candidate" ]]; then' \
  '    printf "%s" "${arg%%:*}" > "$FAKE_GITHUB_STATE/ref-product_complete-candidate"' \
  '  fi' \
  'done' \
  'if [[ " $* " == *" push "* ]]; then' \
  '  printf "WRITE git-push %s\n" "$*" >> "$FAKE_GITHUB_STATE/calls"' \
  '  printf "ok\n"; exit 0' \
  'fi' \
  'exec "$REAL_GIT" "$@"' > "$fake_bin/git"
chmod +x "$fake_bin/git"
printf '%s\n' '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  'printf "CALL gh %s\n" "$*" >> "$FAKE_GITHUB_STATE/calls"' \
  'args=" $* "' \
  'if [[ "$args" == *" api user --jq .login "* ]]; then echo nicholas-ruest; exit 0; fi' \
  'if [[ "$args" == *" api users/nicholas-ruest --jq .login "* ]]; then echo nicholas-ruest; exit 0; fi' \
  'if [[ "$args" == *" api --include --method GET repos/nicholas-ruest/capability-provenance-mesh-2026-11-26 "* ]]; then' \
  '  if [[ -e "$FAKE_GITHUB_STATE/repo" ]]; then printf "HTTP/2 200 OK\n\n{}\n"; exit 0; fi' \
  '  printf "HTTP/2 404 Not Found\n\ngh: Not Found (HTTP 404)\n"; exit 1' \
  'fi' \
  'if [[ "$args" == *" api --method POST user/repos "* ]]; then' \
  '  touch "$FAKE_GITHUB_STATE/repo"; printf "WRITE repo-create\n" >> "$FAKE_GITHUB_STATE/calls"; echo "{}"; exit 0' \
  'fi' \
  'if [[ "$args" == *"/pulls/1"* ]]; then' \
  '  printf "https://github.com/nicholas-ruest/capability-provenance-mesh-2026-11-26/pull/1\tfalse\topen\t%s\tproduct/complete-candidate\tnicholas-ruest\tmain\n" "$SOURCE_SHA"; exit 0' \
  'fi' \
  'if [[ "$args" == *"/pulls"* ]]; then' \
  '  [[ -e "$FAKE_GITHUB_STATE/pr" ]] && echo 1' \
  '  exit 0' \
  'fi' \
  'if [[ "$args" == *" --jq [.owner.login,.name,.visibility,.archived,.is_template]|@tsv "* ]]; then' \
  '  printf "nicholas-ruest\tcapability-provenance-mesh-2026-11-26\tpublic\tfalse\tfalse\n"; exit 0' \
  'fi' \
  'if [[ "$args" == *" --jq .default_branch "* ]]; then echo main; exit 0; fi' \
  'if [[ "$args" == *" api --method GET repos/nicholas-ruest/capability-provenance-mesh-2026-11-26 "* ]]; then echo "{}"; exit 0; fi' \
  'if [[ "$args" == *" pr create "* ]]; then' \
  '  touch "$FAKE_GITHUB_STATE/pr"; printf "WRITE pr-create\n" >> "$FAKE_GITHUB_STATE/calls"; exit 0' \
  'fi' \
  'printf "unexpected gh invocation: %s\n" "$*" >&2; exit 96' > "$fake_bin/gh"
chmod +x "$fake_bin/gh"

run_publisher() {
  (
    cd "$candidate"
    PATH="$fake_bin:$PATH" REAL_GIT="$real_git" FAKE_GITHUB_STATE="$state" SOURCE_SHA="$source_sha" \
    DREAM_MACHINE_SOURCE_SHA="$source_sha" \
    DREAM_MACHINE_PRODUCER_RUN_ID='e4167056-ce98-832f-b1c9-92740e1d9d99' \
    DREAM_MACHINE_EVIDENCE_DIR="$evidence" \
      bash scripts/publish-via-pr.sh >/dev/null
  )
}

run_publisher
first_writes="$(grep -c '^WRITE ' "$state/calls")"
[[ "$first_writes" -eq 3 ]]
grep -q '^WRITE repo-create$' "$state/calls"
grep -q '^WRITE git-push ' "$state/calls"
grep -q '^WRITE pr-create$' "$state/calls"

run_publisher
second_writes="$(grep -c '^WRITE ' "$state/calls")"
[[ "$second_writes" -eq "$first_writes" ]]
[[ "$(cat "$state/ref-main")" == "$base_sha" ]]
[[ "$(cat "$state/ref-product_complete-candidate")" == "$source_sha" ]]
node --input-type=module -e '
  import { readFileSync } from "node:fs";
  const receipt = JSON.parse(readFileSync(process.argv[1], "utf8"));
  if (receipt.owner_repo !== "nicholas-ruest/capability-provenance-mesh-2026-11-26") throw new Error("owner mismatch");
  if (receipt.base.ref !== "main" || receipt.base.sha !== process.argv[2]) throw new Error("base mismatch");
  if (receipt.candidate.sha !== process.argv[3]) throw new Error("candidate mismatch");
  if (receipt.pull_request.draft || receipt.pull_request.state !== "open") throw new Error("PR state mismatch");
' "$evidence/publication-readback.json" "$base_sha" "$source_sha"

printf 'PUBLISHER_ROUTE_NON_NETWORK_PASS writes=%s idempotent_retry_writes=0\n' "$first_writes"
