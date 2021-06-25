#!/usr/bin/env python3
from asyncio import get_event_loop
from socksx import copy_bidirectional
from socksx.socks6 import Client
from socksx.socket import SocketServer

# iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000

async def main(loop):
    proxy = Client("localhost:1080")
    server = await SocketServer.bind('localhost', 42000)

    while True:
        source = await server.accept()
        loop.create_task(redirect(source, proxy))


async def redirect(source, proxy):
    destination = await source.get_original_dst()
    destination = await proxy.connect(destination)

    copy_bidirectional(source, destination)


if __name__ == "__main__":
    loop = get_event_loop()
    loop.run_until_complete(main(loop))
