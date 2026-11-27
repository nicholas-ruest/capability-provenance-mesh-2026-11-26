# Deployment topology

One-command launch starts rootless Rust API and frontend containers on one shared data/evidence volume. Upstreams run as bounded child processes; the service owns no private database. Health covers liveness; readiness verifies store, contract, and adapter configuration.

The host forbids privileged networking and cannot assume slirp access to /dev/net/tun. Verification uses compatible host or network-none operation with localhost connectivity and no host-policy change. Cloud Run service and rollback manifests are configuration only.

Start, stop, reset, and seed are idempotent. Reset targets only explicit product data. Restart verification retains the shared volume, restarts the backend, and retrieves identical decision/audit hashes through the browser.
