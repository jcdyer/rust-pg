import socket
import struct

unix_socket = '/run/postgresql/.s.PGSQL.5432'

sock = socket.socket(socket.AF_UNIX)
sock.connect(unix_socket)

#                 proto_version params                                        terminator
startup_message = b'\0\x03\0\0' b'database\0' b'cliff\0' b'user\0' b'cliff\0' b'\0' 
print(repr(startup_message))
print(hex(4+ len(startup_message)))
length = struct.pack('!I', len(startup_message) + 4)
print([hex(x) for x in length])
sock.sendall(length)
sock.sendall(startup_message)
import time
print( "Receiving")
chunks = [sock.recv(16)]
print(chunks)
while len(chunks[-1]) == 16:
    chunks.append(sock.recv(16))
    print(chunks[-1])
response = b''.join(chunks)

sock.sendall(b'X\0\0\0\x04')
res = sock.recv(16)
print(res)
sock.close()
print("done")
print(response)
