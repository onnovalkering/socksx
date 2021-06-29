#!/usr/bin/env python3
from asyncio import get_event_loop
from socksx.socks6 import Client
from socksx.socket import SocketAddress
from socket import fromfd, AF_INET, SOCK_STREAM

# ./examples/proxy.py
# nc -l -k 12345

async def main():
    destination = SocketAddress("127.0.0.1:12345")

    client = Client("localhost:1080")
    socket = await client.connect(destination)

    raw_fd = await socket.get_raw_fd()
    print(f"RAW SOCKET FD: {raw_fd}")

    py_socket = fromfd(raw_fd, AF_INET, SOCK_STREAM)
    py_socket.send(b"Hello, world!\n")

get_event_loop().run_until_complete(main())
