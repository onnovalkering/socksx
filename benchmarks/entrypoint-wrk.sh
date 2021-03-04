#!/usr/bin/env bash

set -euo pipefail

WRK_CONNECTIONS="${1}"
WRK_DURATION="${2}"
WRK_THREADS="${3}"
WRK_TARGET="${4}"
PROXY_HOST="${5:-}"
PROXY_VERSION="${6-5}"

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

wrk --connections $WRK_CONNECTIONS --duration $WRK_DURATION --threads $WRK_THREADS $WRK_TARGET