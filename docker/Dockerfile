FROM rust:1.93-slim-trixie AS builder

WORKDIR /app

COPY Cargo.lock ./
COPY Cargo.toml ./

# PERFORMANCE: increase the rebuild speed because it loads all rust dependencies
# See: https://www.youtube.com/watch?v=5Wfpzfniu4I
RUN mkdir -p src \
	&& echo 'fn main() {}' > src/main.rs \
	&& cargo build --release \
	&& rm -rf src target/release/deps/mandelbrot*

COPY ./src/ ./src/
RUN cargo build --release

# Final image — no Rust toolchain, no build artifacts, just the binary
FROM debian:trixie-slim
WORKDIR /app
COPY --from=builder /app/target/release/mandelbrot .
CMD ["./mandelbrot"]
