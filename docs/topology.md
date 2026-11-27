# Deployment topology

[compose.yml](../compose.yml) starts the Rust API and web frontend as rootless containers with a shared durable data boundary. [containers/api.Containerfile](../containers/api.Containerfile) builds the service; [containers/frontend.Containerfile](../containers/frontend.Containerfile) and [containers/nginx.conf](../containers/nginx.conf) serve the React build.

The authorized host cannot rely on privileged containers or default slirp access to a tunnel device, so [scripts/container-readiness.sh](../scripts/container-readiness.sh) verifies the compatible host-network topology and localhost readiness. Health is process liveness; readiness includes storage and application usability.

[deploy/cloud-run-service.yaml](../deploy/cloud-run-service.yaml) and [deploy/rollback.env.example](../deploy/rollback.env.example) are configuration only. No deployment or spending is authorized. Restart evidence retains the mounted data boundary, restarts the API, and retrieves the same decision in the browser.
