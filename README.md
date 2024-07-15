cargo build
cargo run $NORDVPN_TOKEN

docker build -t nordvpn-rs . --build-arg TARGETPLATFORM=amd64 --build-arg BUILD_SOURCE_IMAGE=rust:1.79

source .env
docker run nordvpn-rs $NORDVPN_TOKEN