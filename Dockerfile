FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY src src
COPY migrations migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/fleet-telemetry-api /usr/local/bin/fleet-telemetry-api
COPY migrations migrations
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
USER 10001:10001
ENTRYPOINT ["fleet-telemetry-api"]
