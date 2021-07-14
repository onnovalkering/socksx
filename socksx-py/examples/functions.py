#!/usr/bin/env python3
from asyncio import get_event_loop
from click import command, option, Choice
from socksx import copy_bidirectional, socks6
from socksx.socket import Socket, SocketServer, SocketFunction

class Counter(SocketFunction):
    """
    Doesn't perform any transformation, just counts the observed bytes.
    """
    def __init__(self):
        self.observed = 0

    def partial(self, bytes):
        self.observed += len(bytes)
        print(f"Observed {self.observed} bytes up until now.")

        return bytes

    def end(self):
        print("EOF")

@command()
@option('-d', '--debug', default=False, help="Prints debug information")
@option('-h', '--host', default="127.0.0.1", help="Host (IP) for the SOCKS server")
@option('-p', '--port', default=1080, help="Port for the SOCKS server")
@option('-l', '--limit', default=256, help="Concurrent connections limit (0=unlimted)")
@option('-s', '--socks', default=6, type=Choice([6]), help="SOCKS version")
def cli(**kwargs):
    loop = get_event_loop()
    loop.run_until_complete(main(loop, **kwargs))


async def main(loop, debug, host, port, limit, socks):
    """
    
    """
    # --debug, --limit, and --socks are currently ignored.
    server = await SocketServer.bind(host, port)

    while True:
        source = await server.accept()
        loop.create_task(accept_request(source))


async def accept_request(source):
    """

    """
    destination = await setup(source)
    if destination is None:
        return

    # Initialize `counter` function and apply to source.
    counter = Counter()
    source.apply(counter)

    copy_bidirectional(source, destination)


async def setup(source):
    """
    
    """
    request = await socks6.read_request(source)
    await socks6.write_no_authentication(source)

    # Only allow destinations with port `12345`.
    if request.destination.port != 12345:
        print("Connection refused.")
        socks6.write_connection_refused_reply(source)
        return

    # Adhere to the chain.
    chain = request.chain(None)
    if chain is not None and chain.has_next():
        next = chain.next_link()
        client = socks6.Client(f"{next.host}:{next.port}")

        # Connect to destination through another proxy.
        destination = await client.connect(request.destination)
    else:
        # Connect directly to the destination.
        destination = await Socket.connect(request.destination)

    # Notify source that the connection has been set up.
    await socks6.write_success_reply(source)

    return destination


if __name__ == "__main__":
    cli()
