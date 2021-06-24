#!/usr/bin/env python3
from asyncio import get_event_loop
from socksx.socks6 import Client
from socket import fromfd, AF_INET, SOCK_STREAM

# nc -l -k 12345

async def main():
    client = Client("localhost:1080")
    rs_socket = await client.connect("localhost:12345")

    raw_fd = rs_socket.get_raw_fd()
    print(f"RAW SOCKET FD: {raw_fd}")

    py_socket = fromfd(raw_fd, AF_INET, SOCK_STREAM)
    py_socket.send(b"Hello, world!\n")

get_event_loop().run_until_complete(main())
