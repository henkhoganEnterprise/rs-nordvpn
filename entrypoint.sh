# Exit script if any command fails (non-zero value)
set -e 

# Starting the nordvpn daemon
/etc/init.d/nordvpn start

# Waiting for the daemon to start
sleep 5

# Starting the nordvpn proxy application
exec ./nordvpn "$@"