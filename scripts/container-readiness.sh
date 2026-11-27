#!/bin/sh
set -eu
url="${1:?readiness URL required}"
if command -v curl >/dev/null 2>&1; then exec curl --fail --silent --show-error "$url"; fi
exec wget -q -O - "$url"
