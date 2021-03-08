#!/usr/bin/env bash

set -euo pipefail

IPERF_TARGET="${1}"
IPERF_PARELLEL="${2-0}"
PROXY_HOST="${3:-}"
PROXY_PORT="${4-1080}"
PROXY_VERSION="${5-5}"

if [ ! -z "$PROXY_HOST" ]; then
    echo "Using proxy."

    iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000
    ./redirector $PROXY_HOST $PROXY_PORT --socks $PROXY_VERSION &

    sleep 1s
else 
    echo "Not using proxy."
fi

iperf --client $IPERF_TARGET --parallel $IPERF_PARELLEL