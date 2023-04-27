# Build stage of the container
FROM rust-build as build
ARG PROFILE=release
WORKDIR /app

# Create dependency caching layer for fast recompilation with dummy main
RUN echo "fn main() {}" > /app/dummy.rs
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
RUN sed -i 's#src/main.rs#dummy.rs#' /app/Cargo.toml
RUN cargo build --profile=$PROFILE

# Build the application
COPY Cargo.toml /app/Cargo.toml
COPY src /app/src
RUN cargo build --profile=$PROFILE

# Final stage of the container
FROM rust-final as final

# Extract compiled binary and environment variables
COPY --from=build /app/target/*/reverse_proxy /reverse_proxy

# Exponse the running port
EXPOSE 80

ENTRYPOINT ["/reverse_proxy"]
