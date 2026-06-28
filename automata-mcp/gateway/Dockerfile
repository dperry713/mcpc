FROM rust:1.77 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Hardened Runtime Container (Zero-Trust Distroless)
FROM gcr.io/distroless/cc-debian12:latest
COPY --from=builder "/usr/src/app/target/release/gateway" "/usr/local/bin/gateway"
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/gateway"]
