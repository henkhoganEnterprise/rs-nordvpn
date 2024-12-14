## Preparation: IMAGE ARGS need to be set at the beginning
ARG APP_SOURCE_IMAGE
ARG BUILD_SOURCE_IMAGE=rust:1.83

## Layer 1
FROM $BUILD_SOURCE_IMAGE AS build-image

COPY src src
COPY Cargo.toml Cargo.toml

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build --release


## Layer 2
FROM ubuntu:24.10 AS app-image
WORKDIR /app

ARG NORDVPN_CLIENT_VERSION=3.19.2
RUN apt-get update && \
    apt-get install -y wget iputils-ping curl && \
    wget -O /tmp/nordrepo.deb https://repo.nordvpn.com/deb/nordvpn/debian/pool/main/n/nordvpn-release/nordvpn-release_1.0.0_all.deb && \
    apt-get install -y /tmp/nordrepo.deb && \
    apt-get update && \
    apt-get install -y nordvpn=${NORDVPN_CLIENT_VERSION} && \
    apt-get remove -y wget nordvpn-release && \
    rm /tmp/nordrepo.deb

COPY --from=build-image /target/release/nordvpn .

ARG VERSION
ENV RSNORDVPN_VERSION=$VERSION
LABEL org.opencontainers.image.version=$VERSION

ARG COMMIT
ENV PYNORDVPN_COMMIT=$COMMIT
LABEL org.opencontainers.image.revision=$COMMIT

LABEL org.opencontainers.image.vendor="Henkhogan"
LABEL org.opencontainers.image.source="https://github.com/HenkhoganEnterprise/rs-nordvpn"

EXPOSE 80 3128
ENTRYPOINT [ "./nordvpn" ]