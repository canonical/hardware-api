#!/usr/bin/env python

import socketserver
from http.server import BaseHTTPRequestHandler
from json import dumps

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


PORT = 8089
with socketserver.TCPServer(("", PORT), CustomHandler) as httpd:
    httpd.serve_forever()
