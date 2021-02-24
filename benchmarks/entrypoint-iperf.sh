#!/usr/bin/env bash

set -euo pipefail

PROXY_HOST="${1}"
PROXY_VERSION="${2}"
TARGET_HOST="${3}"

iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000

./redirector --socks $PROXY_VERSION $PROXY_HOST &

sleep 1s

iperf -c $TARGET_HOST