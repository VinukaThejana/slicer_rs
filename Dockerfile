FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY proto ./proto
COPY build.rs ./

RUN mkdir src && \
  echo "fn main() {}" > src/main.rs && \
  cargo build --release && \
  rm -rf src

COPY src ./src

RUN touch src/main.rs && \
  cargo build --release && \
  strip target/release/slicer_rs

FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /app/target/release/slicer_rs /app/slicer_rs

USER nonroot

EXPOSE 8080

CMD ["/app/slicer_rs"]
