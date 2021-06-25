#!/usr/bin/env python3
from asyncio import get_event_loop
from socksx import TcpServer
from socksx.socks6 import Client


async def main(loop):
    proxy = Client("localhost:1080")
    server = await TcpServer.bind('localhost', 42000)

    while True:
        incoming = await server.accept()
        loop.create_task(redirect(incoming, proxy))


async def redirect(incoming, proxy):
    destination = await incoming.get_original_dst()

    outgoing = await proxy.connect("localhost:12345")
    incoming.copy_bidirectional(outgoing)


if __name__ == "__main__":
    loop = get_event_loop()
    loop.run_until_complete(main(loop))
