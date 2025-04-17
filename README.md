cargo build
cargo run $NORDVPN_TOKEN

docker build -t nordvpn-rs . 
docker build -t nordvpn-rs . --build-arg TARGETPLATFORM=amd64 --build-arg BUILD_SOURCE_IMAGE=rust:1.86

source .env
docker container prune -f
docker run --cap-add NET_ADMIN --cap-add NET_RAW -p 3180:80 -p 3128:3128 --name nordvpn --env RUST_BACKTRACE=1 nordvpn-rs --token $NORDVPN_TOKEN

# Interval rotation
docker run --cap-add NET_ADMIN --cap-add NET_RAW -p 3180:80 -p 3128:3128 --env RUST_BACKTRACE=1 nordvpn-rs --token $NORDVPN_TOKEN --monitored-hosts query1.finance.yahoo.com --proxy-rotation-argument 100 --proxy-rotation-mode timed

# Request count rotation
docker run --cap-add NET_ADMIN --cap-add NET_RAW -p 3180:80 -p 3128:3128 --env RUST_BACKTRACE=1 nordvpn-rs --token $NORDVPN_TOKEN --monitored-hosts query1.finance.yahoo.com --proxy-rotation-argument 100 --proxy-rotation-mode requestcount


tail -f /var/log/nordvpn/daemon.log

curl http://127.0.0.1:3180/nordvpn/status

curl --proxy http://127.0.0.1:3128 https://google.com


# ToDo
proxy needs to become an object with a state that counts requests to hosts and calls reconnected after x connections


main()

Admin           +
                +
Proxy       +++++
            +
Nordvpn +++++