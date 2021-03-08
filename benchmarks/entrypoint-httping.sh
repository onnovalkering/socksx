#!/usr/bin/env bash

set -euo pipefail

HTTPING_COUNT="${1}"
HTTPING_INTERVAL="${2}"
HTTPING_TARGET="${3}"
PROXY_HOST="${4:-}"
PROXY_PORT="${5-1080}"
PROXY_VERSION="${6-5}"

if [ ! -z "$PROXY_HOST" ]; then
    echo "Using proxy."

    iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000
    ./redirector $PROXY_HOST $PROXY_PORT --socks $PROXY_VERSION &

    sleep 1s
else 
    echo "Not using proxy."
fi

httping -S --count $HTTPING_COUNT --interval $HTTPING_INTERVAL $HTTPING_TARGET