FROM docker.io/library/node:24.11.1-bookworm AS node
FROM docker.io/library/rust:1.100.0-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo build --locked --release -p cpm-service --bin cpm-service

FROM docker.io/library/rust:1.100.0-bookworm
COPY --from=node /usr/local/ /usr/local/
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl git python3 python3-venv podman && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/cpm-service /usr/local/bin/cpm-service
COPY --chmod=0755 crates/cpm-service/upstream-harness/ /opt/cpm/harness/
COPY --chmod=0755 scripts/container-readiness.sh /usr/local/bin/readiness
ENV CPM_UPSTREAM_HARNESS_ROOT=/opt/cpm/harness
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cpm-service"]
