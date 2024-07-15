cargo build
cargo run $NORDVPN_TOKEN

docker build -t nordvpn-rs . --build-arg TARGETPLATFORM=amd64 --build-arg BUILD_SOURCE_IMAGE=rust:1.79

source .env
docker run --cap-add NET_ADMIN --cap-add NET_RAW  nordvpn-rs --token $NORDVPN_TOKEN