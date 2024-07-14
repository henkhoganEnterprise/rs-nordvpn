## Preparation: IMAGE ARGS need to be set at the beginning
ARG APP_SOURCE_IMAGE
ARG BUILD_SOURCE_IMAGE

## Layer 1
FROM --platform=$TARGETPLATFORM $BUILD_SOURCE_IMAGE AS build-image

COPY src src
#COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN cargo build --release


## Layer 2
FROM --platform=$TARGETPLATFORM ubuntu:24.04 AS app-image
WORKDIR /app

ARG NORDVPN_CLIENT_VERSION=3.18.2
RUN apt-get update && \
    apt-get install -y wget iputils-ping curl && \
    wget -O /tmp/nordrepo.deb https://repo.nordvpn.com/deb/nordvpn/debian/pool/main/nordvpn-release_1.0.0_all.deb && \
    apt-get install -y /tmp/nordrepo.deb && \
    apt-get update && \
    apt-get install -y nordvpn=3.17.1 && \
    apt-get remove -y wget nordvpn-release && \
    rm /tmp/nordrepo.deb

COPY --from=build-image /target/release/nordvpn .
COPY entrypoint.sh entrypoint.sh

ARG VERSION
ENV RSNORDVPN_VERSION=$VERSION
LABEL org.opencontainers.image.version=$VERSION

ARG COMMIT
ENV PYNORDVPN_COMMIT=$COMMIT
LABEL org.opencontainers.image.revision=$COMMIT

LABEL org.opencontainers.image.vendor="Henkhogan"
LABEL org.opencontainers.image.source="https://github.com/HenkhoganEnterprise/rs-nordvpn"

EXPOSE 3128 3129
ENTRYPOINT [ "sh", "entrypoint.sh"]