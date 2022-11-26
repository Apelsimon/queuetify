### Build client ###
FROM node:latest as client-builder
WORKDIR /app
COPY client/. .
RUN npm install
RUN npm run build

### Build server ###
FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY server/. .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as server-builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY server/. .
ENV SQLX_OFFLINE true
RUN cargo build --release

### Start runtime ###
FROM debian:bullseye-slim AS runtime

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

COPY --from=client-builder /app/dist/. .
COPY --from=server-builder /app/target/release/queuetify queuetify
COPY --from=server-builder /app/.env.secret .
COPY --from=server-builder /app/configuration configuration
COPY --from=server-builder /app/templates templates

ENV APP_ENVIRONMENT production

ENTRYPOINT ["./queuetify"]