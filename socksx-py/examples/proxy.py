#!/usr/bin/env python3
from asyncio import get_event_loop
from socksx import copy_bidirectional, socks6
from socksx.socket import Socket, SocketServer


async def main(loop):
    server = await SocketServer.bind('localhost', 1080)

    while True:
        source = await server.accept()
        loop.create_task(accept_request(source))


async def accept_request(source):
    """

    """
    destination = await setup(source)
    copy_bidirectional(source, destination)


async def setup(source):
    """
    
    """
    request = await socks6.read_request(source)
    await socks6.write_no_authentication(source)

    # Connecto to the destination
    destination = await Socket.connect(request.destination)

    # Notify source that the connection has been set up.
    await socks6.write_success_reply(source)

    return destination


if __name__ == "__main__":
    loop = get_event_loop()
    loop.run_until_complete(main(loop))
