#!/usr/bin/env bash
set -euo pipefail
h=$(cd "$(dirname "$0")" && pwd); source "$h/candidate-context.sh"; cpm_bind_candidate cosign
r=$1; p="$r/sigstore_cosign"; img=ghcr.io/sigstore/cosign/cosign@sha256:7d08ec40d6f6fd9ced81a498c367ee6d4dd90d73218f58a791f78661bdbef769
test "$(git -C "$p" rev-parse HEAD)" = 252f10a80ba871152a12e5ee324ad1e02c85d079
podman run --rm --network=none --user 0:0 -e COSIGN_PASSWORD=candidate-test -v "$CPM_CANDIDATE_DIR:/work:z" -w /work "$img" generate-key-pair
podman run --rm --network=none --user 0:0 -e COSIGN_PASSWORD=candidate-test -v "$CPM_CANDIDATE_DIR:/work:z" -w /work "$img" sign-blob --use-signing-config=false --new-bundle-format=false --tlog-upload=false --yes --key cosign.key artifact.bin > "$CPM_CANDIDATE_DIR/candidate.sig"
podman run --rm --network=none --user 0:0 -v "$CPM_CANDIDATE_DIR:/work:ro,z" -w /work "$img" verify-blob --new-bundle-format=false --key cosign.pub --signature candidate.sig --insecure-ignore-tlog artifact.bin
printf 'candidate=%s cosign-verified\n' "$CPM_CANDIDATE_ARTIFACT_SHA256"
