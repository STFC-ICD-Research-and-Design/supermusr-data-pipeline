# Build
FROM docker.io/library/rust:latest as builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      clang-19 \
      cmake \
      flatbuffers-compiler \
      libclang1-19 \
    && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ARG component
RUN cargo build \
      --release \
      --package $component && \
    cp /app/target/release/$component /app/app

# Runtime
FROM docker.io/library/debian:13-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends tini && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/app /app/app

ENV OBSERVABILITY_ADDRESS=0.0.0.0:9090
EXPOSE 9090/tcp

ENTRYPOINT ["/usr/bin/tini", "--", "/app/app"]
