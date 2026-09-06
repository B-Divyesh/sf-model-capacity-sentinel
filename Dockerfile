# syntax=docker/dockerfile:1.7
ARG BUILD_SHA=dev
FROM node:22-alpine AS web
ARG BUILD_SHA=dev
WORKDIR /src
COPY package.json package-lock.json tsconfig.json vite.config.ts ./
COPY frontend ./frontend
ENV VITE_BUILD_SHA=$BUILD_SHA
RUN npm ci && npm run build

FROM rust:1-slim AS server
ARG BUILD_SHA
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
ENV BUILD_SHA=$BUILD_SHA
RUN cargo build --locked --release
RUN mkdir /empty-data && chown 65532:65532 /empty-data

FROM gcr.io/distroless/cc-debian12:nonroot
ARG BUILD_SHA
WORKDIR /app
COPY --from=server /src/target/release/model-capacity-sentinel /app/sentinel
COPY --from=web /src/dist /app/dist
COPY --from=server --chown=65532:65532 /empty-data /data
LABEL org.opencontainers.image.revision=$BUILD_SHA
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["/app/sentinel"]
