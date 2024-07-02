# We use the latest Rust stable release as base image
FROM rust

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y
# Copy all files from our working environment to our Docker image
COPY . .

ENV SQLX_OFFLINE=true

# We'll use the release profile to make it blazingly fast
RUN cargo build --release
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]


# Build a docker image tagged as "zero2prod" according to the recipe
# specified in `Dockerfile`
# docker build --tag zero2prod --file Dockerfile .