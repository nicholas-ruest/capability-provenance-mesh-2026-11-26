FROM docker.io/library/rust:1.91-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
RUN cargo build --locked --release -p cpm-service --bin cpm-service

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/cpm-service /usr/local/bin/cpm-service
COPY scripts/container-readiness.sh /usr/local/bin/readiness
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cpm-service"]
