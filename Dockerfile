FROM rust:1.67-alpine as build

ARG BUILD_VERSION

RUN apk add --update --no-cache openssl-dev musl-dev protoc
RUN rustup component add rustfmt

WORKDIR /app

COPY . .

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN sed -i -e "s/^version = .*/version = \"${BUILD_VERSION}\"/" Cargo.toml
RUN cargo install --path .

FROM alpine:3.17

ARG BUILD_VERSION
ARG COMMIT_SHA

LABEL org.opencontainers.image.source="https://github.com/WirePact/k8s-basic-auth-translator" \
    org.opencontainers.image.authors="cbuehler@rootd.ch" \
    org.opencontainers.image.url="https://github.com/WirePact/k8s-basic-auth-translator" \
    org.opencontainers.image.documentation="https://github.com/WirePact/k8s-basic-auth-translator/blob/main/README.md" \
    org.opencontainers.image.source="https://github.com/WirePact/k8s-basic-auth-translator/blob/main/Dockerfile" \
    org.opencontainers.image.version="${BUILD_VERSION}" \
    org.opencontainers.image.revision="${COMMIT_SHA}" \
    org.opencontainers.image.licenses="Apache-2.0" \
    org.opencontainers.image.title="WirePact Kubernetes Basic Auth Translator" \
    org.opencontainers.image.description="Translator for WirePact that handles HTTP Basic Auth (RFC7617) for any software behind."

WORKDIR /app

ENV USER=appuser \
    UID=1000 \
    BUILD_VERSION=${BUILD_VERSION}

COPY --from=build /usr/local/cargo/bin/k8s-basic-auth-translator /usr/local/bin/k8s-basic-auth-translator

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}" && \
    chown -R appuser:appuser /app && \
    chmod +x /usr/local/bin/k8s-basic-auth-translator && \
    apk add --update --no-cache libgcc ca-certificates

USER appuser:appuser

ENTRYPOINT ["/usr/local/bin/k8s-basic-auth-translator"]
