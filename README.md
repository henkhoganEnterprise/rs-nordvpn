cargo build
cargo run $NORDVPN_TOKEN

docker build . --build-arg TARGETPLATFORM=amd64 --build-arg BUILD_SOURCE_IMAGE=ubuntu:24.04

source .env
docker run nordvpn-rs $NORDVPN_TOKEN