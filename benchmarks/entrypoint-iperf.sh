#!/usr/bin/env bash

set -euo pipefail

PROXY_HOST="${1}"
TARGET_HOST="${2}"

iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000

./redirector $PROXY_HOST &

sleep 1s

iperf -c $TARGET_HOST