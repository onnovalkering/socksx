#!/usr/bin/env bash

set -euo pipefail

IPERF_TARGET="${1}"
PROXY_HOST="${2:-}"
PROXY_VERSION="${3-'5'}"

if [ ! -z "$PROXY_HOST" ]; then
    echo "Using proxy."

    iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000
    ./redirector --socks $PROXY_VERSION $PROXY_HOST &

    sleep 1s
else 
    echo "Not using proxy."
fi

# Wait until the next full minute.
sleep $((60 - $(date +%S) ))

iperf -c $TARGET_HOST