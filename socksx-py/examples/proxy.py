#!/usr/bin/env python3
from asyncio import get_event_loop
from click import command, option, Choice
from socksx import copy_bidirectional, socks6
from socksx.socket import Socket, SocketServer

@command()
@option('-c', '--chain', multiple=True, help="Entry in the proxy chain, the order is preserved")
@option('-d', '--debug', default=False, help="Prints debug information")
@option('-h', '--host', default="127.0.0.1", help="Host (IP) for the SOCKS server")
@option('-p', '--port', default=1080, help="Port for the SOCKS server")
@option('-l', '--limit', default=256, help="Concurrent connections limit (0=unlimted)")
@option('-s', '--socks', default=6, type=Choice([6]), help="SOCKS version")
def cli(**kwargs):
    loop = get_event_loop()
    loop.run_until_complete(main(loop, **kwargs))


async def main(loop, chain, debug, host, port, limit, socks):
    """
    
    """
    # --debug, --limit, and --socks are currently ignored.
    server = await SocketServer.bind(host, port)

    while True:
        source = await server.accept()
        loop.create_task(accept_request(source, chain))


async def accept_request(source, local_chain):
    """

    """
    destination = await setup(source, local_chain)

    copy_bidirectional(source, destination)


async def setup(source, local_chain):
    """
    
    """
    request = await socks6.read_request(source)
    await socks6.write_no_authentication(source)

    # Adhere to the chain.
    chain = request.chain(local_chain)
    if chain is not None and chain.has_next():
        next = chain.next_link()
        client = socks6.Client(f"{next.host}:{next.port}")

        # Connect to destination through another proxy.
        destination = await client.connect(request.destination, chain)
    else:
        # Connect directly to the destination.
        destination = await Socket.connect(request.destination)

    # Notify source that the connection has been set up.
    await socks6.write_success_reply(source)

    return destination


if __name__ == "__main__":
    cli()
