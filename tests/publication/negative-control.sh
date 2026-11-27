#!/usr/bin/env bash
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
controller="$root/scripts/dream-machine-publication-gate.mjs"
expected_controller='f64eddb2d284c82e59c066603c4229e735604220212f3d3b30d3552b058ee229'
[[ "$(sha256sum "$controller" | awk '{ print $1 }')" == "$expected_controller" ]]

fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
candidate="$fixture/one-package"
evidence="$fixture/evidence"
fake_bin="$fixture/fake-bin"
write_log="$fixture/github-writes.log"
publisher_log="$fixture/publisher-invoked.log"
mkdir -p "$candidate/src" "$evidence" "$fake_bin"

printf '%s\n' '[package]' 'name = "denied-one-package"' 'version = "0.1.0"' \
  'edition = "2024"' > "$candidate/Cargo.toml"
printf '%s\n' 'pub fn tiny() -> bool { true }' > "$candidate/src/lib.rs"
git -C "$candidate" init -q -b candidate
git -C "$candidate" config user.name 'Publication negative control'
git -C "$candidate" config user.email 'negative-control@example.invalid'
git -C "$candidate" add Cargo.toml src/lib.rs
git -C "$candidate" commit -q -m 'one-package negative control'

real_git="$(command -v git)"
printf '%s\n' '#!/usr/bin/env bash' \
  'if [[ " $* " == *" push "* ]]; then printf "git-push:%s\n" "$*" >> "$GITHUB_WRITE_LOG"; exit 97; fi' \
  'exec "$REAL_GIT" "$@"' > "$fake_bin/git"
printf '%s\n' '#!/usr/bin/env bash' \
  'printf "gh:%s\n" "$*" >> "$GITHUB_WRITE_LOG"' \
  'exit 98' > "$fake_bin/gh"
chmod +x "$fake_bin/git" "$fake_bin/gh"

set +e
PATH="$fake_bin:$PATH" REAL_GIT="$real_git" GITHUB_WRITE_LOG="$write_log" \
DREAM_MACHINE_PRODUCER_RUN_ID='e4167056-ce98-832f-b1c9-92740e1d9d99' \
DREAM_MACHINE_PUBLISHER_INVOCATION_LOG="$publisher_log" \
node "$controller" --publish "$candidate" "$evidence" -- \
  bash "$root/scripts/publish-via-pr.sh" >"$fixture/stdout.log" 2>"$fixture/stderr.log"
status=$?
set -e

[[ "$status" -ne 0 ]]
grep -q 'PUBLISH_DENIED_OR_FAILED' "$fixture/stderr.log"
[[ ! -e "$publisher_log" ]]
[[ ! -s "$write_log" ]]
node --input-type=module -e '
  import { readFileSync } from "node:fs";
  const receipt = JSON.parse(readFileSync(process.argv[1], "utf8"));
  if (receipt.status !== "PUBLISH_DENIED") throw new Error("negative control did not deny");
  if (receipt.publication_attempted !== false) throw new Error("publisher was attempted");
  if (receipt.source_sha !== process.argv[2]) throw new Error("source binding mismatch");
' "$evidence/publication-gate.json" "$(git -C "$candidate" rev-parse HEAD)"

printf 'PUBLISH_DENIED_NEGATIVE_CONTROL_PASS publisher_calls=0 github_writes=0\n'
