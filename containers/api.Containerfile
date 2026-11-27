FROM docker.io/library/rust:1.100.0-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo build --locked --release -p cpm-service --bin cpm-service

FROM docker.io/library/debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && groupadd --gid 65532 cpm \
    && useradd --uid 65532 --gid 65532 --no-create-home --shell /usr/sbin/nologin cpm \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build --chown=65532:65532 /src/target/release/cpm-service /usr/local/bin/cpm-service
COPY --chmod=0755 scripts/container-readiness.sh /usr/local/bin/readiness
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cpm-service"]
