#!/usr/bin/env python3
from asyncio import get_event_loop
from socksx.socks6 import Client
from socket import fromfd, AF_INET, SOCK_STREAM

# socksx -p 1080
# nc -l -k 12345

async def main():
    client = Client("localhost:1080")
    socket = await client.connect("localhost:12345")

    raw_fd = socket.get_raw_fd()
    print(f"RAW SOCKET FD: {raw_fd}")

    py_socket = fromfd(raw_fd, AF_INET, SOCK_STREAM)
    py_socket.send(b"Hello, world!\n")

get_event_loop().run_until_complete(main())
