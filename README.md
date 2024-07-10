# Zero2Prod

This repo is a companion to the book [Zero To Production In Rust](https://www.zero2prod.com/). It contains the code samples and exercises discussed in the book.

## Getting Started

To get started, clone the repo and navigate to the project directory, and run `cargo-watch` to start developing.

```bash
cargo watch -q -c -w src/ -x check -x test -x run
```

To avoid recompiling when changing anything else than the source code, the cargo-watch command is configured to only watch the `src/` directory.

### Run tests

To run the tests, use the following command:

```bash
cargo test
```

To run tests with logging enabled, use the following command:

```bash
TEST_LOG=true cargo test health_check_works | bunyan
```

OBS: The `bunyan` command is used to format the logs in a more readable way. You can install it using the following command:

```bash
npm install -g bunyan
```

or

```bash
cargo install bunyan
```

### Docker

To build the docker image, use the following command:

```bash
docker build --tag zero2prod --file Dockerfile .
```

To run the docker image, use the following command:

```bash
docker run --rm -p 8000:8000 zero2prod
```

## Scripts

To run the database run the script in the folder `scripts/init_db.sh`:

```bash
./scripts/init_db.sh
```

If the DB has already been deployed before, you the add the variable `SKIP_DOCKER` to skip the docker deployment:

```bash
SKIP_DOCKER=true ./scripts/init_db.sh
```

## Optmizations

To learn about symbol stripping, check the following [link](https://github.com/johnthagen/min-sized-rust#strip-symbols-from-binary)

The Docker image rust:alpine is used to build the final image. This image is very small, but it does not have the necessary tools to debug the application. To debug the application, you can use the image rust:slim.

## Rust Development Tools

### Run Migrations

Install sqlx-cli to run migrations:

```bash
cargo install --no-default-features \
--features rustls,postgres
```

If you want to use sqlx offline, you can use the following command:

```bash
cargo sqlx prepare --workspace
```

To get the logs from sqlx, run:

```bash
# sqlx logs are a bit spammy, cutting them out to reduce noise
export RUST_LOG="sqlx=error,info"
export TEST_LOG=true
# Runing the logs for tests
cargo t <TEST_NAME> | bunyan
```

### Measure Code Coverage

First, install cargo-tarpaulin:

```bash
cargo install cargo-tarpaulin
```

To measure code coverage, run the following command:

```bash
cargo tarpaulin --ignore-tests
```

Check the tarpaullin documentation for more information [here](https://github.com/xd009642/tarpaulin)

### Clippy Linting - Your new best friend

Clippy is a collection of lints to catch common mistakes and improve your Rust code.

If Clippy is not installed, you can install it using the following command:

```bash
rustup component add clippy
```

To run clippy, use the following command:

```bash
cargo clippy
```

If in CI pipelines, you can fail in warning with the following command:

```bash
cargo clippy -- -D warnings
```

Clippy's documentation can be found [here](https://github.com/rust-lang/rust-clippy)

### Rust Formatting

Rustfmt is Rust's official formatting tool. It can be installed using the following command:

```bash
rustup component add rustfmt
```

To format your code, use the following command:

```bash
cargo fmt
```

If in CI pipelines, you can fail in warning with the following command:

```bash
cargo fmt -- --check
```

Rustfmt's documentation can be found [here](https://github.com/rust-lang/rustfmt)

### Rust Crates Security

To check for security vulnerabilities in your dependencies, you can use the cargo-audit tool. Cargo-deny is also another tool that can be used to check for security vulnerabilities.

First, install the tool using the following command:

```bash
cargo install cargo-audit
```

To check for vulnerabilities, use the following command:

```bash
cargo audit
```

### Remove unused dependencies

To remove unused dependencies, you can use the cargo-udeps tool. First, install the tool using the following command:

```bash
cargo install cargo-udeps
```

To remove unused dependencies, use the following command:

```bash
cargo udeps
```

### Cargo Chef

Allow to build the dependencies in a separate step, to speed up the build process.

First, install the tool using the following command:

```bash
cargo install cargo-chef
```

To prepare the dependencies, use the following command:

```bash
cargo chef prepare --recipe-path recipe.json
```

To build the dependencies, use the following command:

```bash
cargo chef cook --release --recipe-path recipe.json
```

## Tips

These are some usefull crates:

- tracing
- tracing-bunyan-formatter
- tracing-log
- tracing-subscriber
- secrecy (Protect your logs from leaking sensitive information)
- fake (Generate random fake data)
- wiremock
- linkify (extract links from text)
- tera (template engine for emails)
- thiserror (better error handling)
- anyhow (better error handling, working together with thiserror)
- sha3 (hashing with SHA3)
- argon2 (hashing with Argon2)

Search about PHC string format for password hashing.

For example, the PHC string format for Argon2 is:

- Argon2:
  `${algorithm}${algorithm version}${,-separated algorithm parameters}${hash}${salt}`

For working with HTML in Rust, you can use the following crates:

- htmlescape
- hmac
- urlencoding
