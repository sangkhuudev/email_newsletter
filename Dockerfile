# Use a Rust base image for the build stage
FROM lukemathwalker/cargo-chef:latest-rust-1.72.0 as chef
WORKDIR /app

# Install required build tools
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
# Build our project
RUN cargo build --release --bin email_newsletter 

# Create the final runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install OpenSSL and CA certificates (Updated to include OpenSSL)
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/email_newsletter email_newsletter
COPY configuration configuration

# Set the environment variable for the application
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./email_newsletter"]
