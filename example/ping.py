import socket
import sys
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
addr = ('localhost', int(sys.argv[1]))
buf = "this is a ping!".encode('utf-8')

print("pinging...", file=sys.stderr)
while True:
	sock.sendto(buf, ("127.0.0.1", int(sys.argv[1])))
	time.sleep(1)
	# buf, raddr = sock.recvfrom(4096)
	# print(buf.decode("utf-8"), file=sys.stderr)
	# if buf and buf.decode("utf-8") == "reply":
	# 	# print(buf.decode("utf-8"), file=sys.stderr)
	# 	print("receive the reply from qemu.")
	# 	print("test pass!")
	# 	break
