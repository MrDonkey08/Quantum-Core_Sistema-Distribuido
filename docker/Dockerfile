FROM rust:1.93-trixie

WORKDIR /app

COPY Cargo.lock ./
COPY Cargo.toml ./

# PERFORMANCE: increase the rebuild speed because it loads all rust dependencies
# See: https://www.youtube.com/watch?v=5Wfpzfniu4I
RUN mkdir -p src \
	&& echo 'fn main() {}' > src/main.rs \
	&& cargo build --release \
	# Remove dummy artifacts
	&& rm -rf src target/release/deps/mandelbrot*

COPY ./src/ ./src/
RUN cargo build --release

# Executes the pre-compiled rust release binary, just like `cargo run --release`
CMD ["./target/release/mandelbrot"]
