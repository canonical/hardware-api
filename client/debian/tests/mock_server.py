#!/usr/bin/env python3

import socketserver
import sys
from http.server import BaseHTTPRequestHandler
from json import dumps
import socket
import os

RESPONSE = dumps({"status": "Not Seen"}).encode()

class CustomHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == "/v1/certification/status":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(RESPONSE)))
            self.end_headers()
            self.wfile.write(RESPONSE)
        else:
            self.send_response(404)
            self.end_headers()

def notify_systemd_ready():
    """Notify systemd that the service is ready"""
    notify_socket = os.getenv("NOTIFY_SOCKET")
    if notify_socket:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
        try:
            sock.connect(notify_socket)
            sock.sendall(b"READY=1\n")
        finally:
            sock.close()
        print("Systemd notified: READY=1", file=sys.stderr)

PORT = 8089
with socketserver.TCPServer(("", PORT), CustomHandler) as httpd:
    print(f"Starting mock server on port {PORT}", file=sys.stderr)
    # Notify systemd of readiness
    notify_systemd_ready()
    httpd.serve_forever()
