#!/usr/bin/env bash

set -euo pipefail

WRK_CONNECTIONS="${1}"
WRK_DURATION="${2}"
WRK_THREADS="${3}"
WRK_TARGET="${4}"
PROXY_HOST="${5:-}"
PROXY_PORT="${6-1080}"
PROXY_VERSION="${7-5}"

if [ ! -z "$PROXY_HOST" ]; then
    echo "Using proxy."

    iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000
    ./redirector $PROXY_HOST $PROXY_PORT --socks $PROXY_VERSION &

    sleep 1s
else 
    echo "Not using proxy."
fi

wrk --header "Connection: Close" --latency --connections $WRK_CONNECTIONS --duration $WRK_DURATION --threads $WRK_THREADS $WRK_TARGET