# * Chefs stage
FROM lukemathwalker/cargo-chef AS chef

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app

# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .

# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

# * Builder stage
# We use the latest Rust stable release as base image
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build project dependencies, and not the app
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.

# Copy all files from our working environment to our Docker image
COPY . .

ENV SQLX_OFFLINE=true

# We'll use the release profile to make it blazingly fast
RUN cargo build --release

# * Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime!
COPY configuration configuration

ENV APP_ENVIRONMENT=production

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]

# Build a docker image tagged as "zero2prod" according to the recipe
# specified in `Dockerfile`
# docker build --tag zero2prod --file Dockerfile .