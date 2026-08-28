# syntax=docker/dockerfile:1.7
FROM node:22-alpine AS web
WORKDIR /src
COPY package.json package-lock.json tsconfig.json vite.config.ts ./
COPY frontend ./frontend
RUN npm ci && npm run build

FROM rust:1.90-bookworm AS server
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
ARG BUILD_SHA=container
ENV BUILD_SHA=$BUILD_SHA
RUN cargo build --locked --release
RUN mkdir /empty-data && chown 65532:65532 /empty-data

FROM gcr.io/distroless/cc-debian12:nonroot
ARG BUILD_SHA=unknown
WORKDIR /app
COPY --from=server /src/target/release/model-capacity-sentinel /app/sentinel
COPY --from=web /src/dist /app/dist
COPY --from=server --chown=65532:65532 /empty-data /data
ENV PORT=8080 DATA_DIR=/data STATIC_DIR=/app/dist RUST_LOG=info BUILD_SHA=$BUILD_SHA
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["/app/sentinel"]
