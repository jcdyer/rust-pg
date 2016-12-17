"""
Demo postgres connector
"""

from __future__ import print_function, unicode_literals

import socket
import struct
from contextlib import closing

# pylint: disable=invalid-name
unix_socket = '/run/postgresql/.s.PGSQL.5432'
# unix_socket = '/private/tmp/.s.PGSQL.5432'
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

def create_message(identifier, body):
    """
    Build a message from an identifier and a body
    """
    if not identifier:
        identifier = b''
    length = struct.pack('!I', len(body) + 4)
    return b''.join([identifier, length, body])


def send_startup_message(sock):
    """
    Send a startup message
    """
    #                 proto_version params                                                terminator
    startup_message = b'\0\x03\0\0' b'database\0' b'cliff\0' b'user\0' b'cliff\0' b'\0'
    startup_message = create_message(None, startup_message)
    print(repr(startup_message))
    sock.sendall(startup_message)
    return receive(sock)


def send_terminate(sock):
    """
    End the connection
    """
    sock.sendall(b'X\0\0\0\x04')
    return receive(sock)

def send_query(sock):
    """
    Send a query
    """
    query = "SELECT version()\0".encode('utf-8')
    message = create_message(b'Q', query)
    sock.sendall(message)
    return receive(sock)

def iter_msgs(inputs):
    while inputs:
        length = (inputs[1] << 24) + (inputs[2] << 16) + (inputs[3] << 8) + inputs[4] + 1
        msg, inputs = inputs[:length], inputs[length:]
        yield msg

def parse_msg(msg):
    if not msg:
        print("No message")
        return
    identifier = msg[:1].decode('utf-8')
    length = (msg[1] << 24) + (msg[2] << 16) + (msg[3] << 8) + msg[4] + 1
    remainder = msg[5:]
    print("Message: {}".format(msg))
    print("  Identifier: {}".format(identifier))
    print("  Length: {}".format(length))
    print("  Remainder: {!r}".format(remainder.decode('latin1')))

def main():
    """
    Connect to postgres server.
    """
    with closing(socket.socket(socket.AF_UNIX)) as sock:
        sock.connect(unix_socket)
        response = send_startup_message(sock)
        print("Startup response")
        print(response)
        for msg in iter_msgs(response):
            parse_msg(msg)
        print()

        response = send_query(sock)
        print("Query response")
        print(response)
        for msg in iter_msgs(response):
            parse_msg(msg)
        print()
        response = send_terminate(sock)
        print("Terminate response")
        print(response)
        for msg in iter_msgs(response):
            parse_msg(msg)
        print()
    print("done")


if __name__ == '__main__':
    main()
