#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/candidate-context.sh"
cpm_bind_candidate cosign
r=$1; p="$r/sigstore_cosign"; h=$(cd "$(dirname "$0")" && pwd); img=ghcr.io/sigstore/cosign/cosign@sha256:7d08ec40d6f6fd9ced81a498c367ee6d4dd90d73218f58a791f78661bdbef769
test "$(git -C "$p" rev-parse HEAD)" = 252f10a80ba871152a12e5ee324ad1e02c85d079
if podman run --rm --network=none --user 0:0 -v "$h/fixtures:/fixture:ro,z" "$img" verify-blob --key /fixture/probe.pub --bundle /fixture/probe.bundle --insecure-ignore-tlog /fixture/tampered.md; then echo unexpected-success >&2; exit 1; else echo tamper-rejected; fi
