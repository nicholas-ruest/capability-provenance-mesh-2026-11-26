#!/usr/bin/env bash
set -euo pipefail
root=${1:?usage: install-upstreams.sh SOURCE_ROOT}
mkdir -p "$root"
pin() {
  repo=$1; dir=$2; rev=$3
  if test -d "$root/$dir/.git"; then
    test "$(git -C "$root/$dir" rev-parse HEAD)" = "$rev"
    return
  fi
  git init -q "$root/$dir"
  git -C "$root/$dir" remote add origin "https://github.com/$repo.git"
  git -C "$root/$dir" fetch -q --depth 1 origin "$rev"
  git -C "$root/$dir" checkout -q --detach FETCH_HEAD
  test "$(git -C "$root/$dir" rev-parse HEAD)" = "$rev"
}
pin ruvnet/autogenous ruvnet_autogenous 3155111ae78039c3993a544044a9cac041090ea6
pin ruvnet/ruGraph ruvnet_ruGraph d66d976c0ba8fa792fcf024c2a07b51480e4fbf1
pin ruvnet/agenticow ruvnet_agenticow a6467863e250371562c2e0c83f5f993459607746
pin ruvnet/ai-text-watermark ruvnet_ai-text-watermark 5871cf5fef6547c97626ba72d976f167562dab2a
pin ruvnet/LatentMesh ruvnet_LatentMesh b04a70523196181aadc6d694e02de1f5722fae6b
pin NVIDIA/SkillSpector NVIDIA_SkillSpector 48e0f942ea1f5fdb7aa096a6f65a0adeda2ecc4e
pin sigstore/cosign sigstore_cosign 252f10a80ba871152a12e5ee324ad1e02c85d079
cargo +1.90.0 build --manifest-path "$root/ruvnet_ruGraph/crates/ruvector-language/Cargo.toml" --locked --bin rvg-core
python3 -m venv "$root/.venv-skillspector"
"$root/.venv-skillspector/bin/pip" install -q -e "$root/NVIDIA_SkillSpector"
