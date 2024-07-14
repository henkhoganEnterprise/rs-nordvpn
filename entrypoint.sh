# Starting the nordvpn daemon
/etc/init.d/nordvpn start

# Waiting for the daemon to start
sleep 5

# Starting the nordvpn proxy application
./nordvpn "$@"