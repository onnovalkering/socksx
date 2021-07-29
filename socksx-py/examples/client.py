#!/usr/bin/env python3
from asyncio import get_event_loop
from click import command, option, Choice
from socksx.socks6 import Client
from socksx.socket import SocketAddress
from socket import fromfd, AF_INET, SOCK_STREAM

@command()
@option('-d', '--debug', default=False, help="Prints debug information")
@option('-d', '--destination', default="127.0.0.1:12345", help="Address of the destination")
@option('-p', '--proxy', default='127.0.0.1:1080', help="Address of the proxy")
@option('-s', '--socks', default=6, type=Choice([6]), help="SOCKS version")
def cli(**kwargs):
    loop = get_event_loop()
    loop.run_until_complete(main(loop, **kwargs))


async def main(loop, debug, destination, proxy, socks):
    """

    """
    # --debug  and --socks are currently ignored.
    destination = SocketAddress(destination)

    client = Client(proxy)
    socket = await client.connect(destination)

    # Convert to Python socket.
    raw_fd = await socket.get_raw_fd()
    print(f"RAW SOCKET FD: {raw_fd}")

    py_socket = fromfd(raw_fd, AF_INET, SOCK_STREAM)
    py_socket.send(b"Hello, world!\n")


if __name__ == "__main__":
    cli()