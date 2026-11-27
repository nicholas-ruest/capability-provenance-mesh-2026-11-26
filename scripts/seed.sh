#!/usr/bin/env bash
set -euo pipefail
api="${CPM_PUBLIC_API_URL:-http://127.0.0.1:${CPM_API_PORT:-8080}}"
payload='{"name":"deterministic-safe-capability","version":"1.0.0","authority":["read:evidence"],"effects":["append:audit"],"seed":2601126}'
curl --fail --silent --show-error -H "Authorization: Bearer ${CPM_DEMO_TOKEN:-local-operator}" -H 'Content-Type: application/json' -H 'Idempotency-Key: seed-2601126' --data "$payload" "$api/api/v1/intakes"
printf '\nSEED_SUBMITTED seed=2601126\n'
