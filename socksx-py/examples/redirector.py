#!/usr/bin/env python3
from asyncio import get_event_loop
from click import command, option, Choice
from socksx import copy_bidirectional
from socksx.socks6 import Client
from socksx.socket import SocketServer

# iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000

@command()
@option('-d', '--debug', default=False, help="Prints debug information")
@option('-p', '--proxy', default='127.0.0.1:1080', help="Address of the proxy")
@option('-s', '--socks', default=6, type=Choice([6]), help="SOCKS version")
def cli(**kwargs):
    loop = get_event_loop()
    loop.run_until_complete(main(loop, **kwargs))


async def main(loop, debug, proxy, socks):
    """
    
    """
    # --debug and --socks are currently ignored.
    proxy = Client(proxy)
    server = await SocketServer.bind('localhost', 42000)

    while True:
        source = await server.accept()
        loop.create_task(redirect(source, proxy))


async def redirect(source, proxy):
    """
    
    """
    destination = await source.get_original_dst()
    destination = await proxy.connect(destination)

    copy_bidirectional(source, destination)


if __name__ == "__main__":
    cli()
