"""
Demo postgres connector
"""

from __future__ import print_function, unicode_literals

import socket
import struct
from contextlib import closing

# pylint: disable=invalid-name
unix_socket = '/run/postgresql/.s.PGSQL.5432'
unix_socket = '/private/tmp/.s.PGSQL.5432'
# pylint: enable=invalid-name

def receive(sock):
    """
    Get all currently available data from the socket, and return as a single string.
    """
    chunk_size = 16
    chunks = [sock.recv(chunk_size)]
    while len(chunks[-1]) == chunk_size:
        chunks.append(sock.recv(chunk_size))
    return b''.join(chunks)

def send_startup_message(sock):
    """
    Send a startup message
    """
    #                 proto_version params                                                terminator
    startup_message = b'\0\x03\0\0' b'database\0' b'cliffdyer\0' b'user\0' b'cliffdyer\0' b'\0'
    length = struct.pack('!I', len(startup_message) + 4)
    startup_message = b''.join([length, startup_message])
    print(repr(startup_message))
    sock.sendall(startup_message)
    return receive(sock)

def send_terminate(sock):
    """
    End the connection
    """
    sock.sendall(b'X\0\0\0\x04')
    return receive(sock)

def main():
    """
    Connect to postgres server.
    """
    with closing(socket.socket(socket.AF_UNIX)) as sock:
        sock.connect(unix_socket)
        response = send_startup_message(sock)
        print(response)
        response = send_terminate(sock)
        print(response)
    print("done")


if __name__ == '__main__':
    main()
