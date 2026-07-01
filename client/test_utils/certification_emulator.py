#!/usr/bin/env python3
import sys
import http.server
import time

response_code = 200
response_contents = '{"status": "certified", "certified_url": "https://certification.ubuntu.com/certified/"}'


class mysimpleHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        global response_code, response_contents

        time.sleep(1)
        self.send_response(response_code)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        # Return a sample JSON response
        self.wfile.write(response_contents.encode("utf-8"))

    def do_POST(self):
        self.do_GET()


def run_server():
    global response_code, response_contents
    port = 8000
    handler = mysimpleHTTPRequestHandler
    status = sys.argv[1]
    if len(sys.argv) > 2:
        port = int(sys.argv[2])
    if status == "certified":
        response_contents = """{
  "status": "Certified",
  "certified_url": "https://ubuntu.com/certified/202405-34051",
  "architecture": "amd64",
  "bios": {
    "vendor": "Dell",
    "version": "1.7.0",
    "revision": null,
    "firmware_revision": "1.9",
    "release_date": null
  },
  "board": {
    "manufacturer": "Dell",
    "product_name": "02395C",
    "version": "X04"
  },
  "chassis": null,
  "available_releases": [
    {
      "distributor": "Ubuntu",
      "version": "24.04",
      "codename": "noble",
      "kernel": {
        "name": null,
        "version": "6.8.0-1009-oem",
        "signature": null,
        "loaded_modules": []
      }
    }
  ]
}
"""
    elif status == "not-certified":
        response_contents = """{
  "status": "Not Seen",
  "certified_url": "https://ubuntu.com/certified/202405-34051",
  "architecture": "amd64",
  "bios": {
    "vendor": "Dell",
    "version": "1.7.0",
    "revision": null,
    "firmware_revision": "1.9",
    "release_date": null
  },
  "board": {
    "manufacturer": "Dell",
    "product_name": "02395C",
    "version": "X04"
  },
  "chassis": null,
  "available_releases": [
    {
      "distributor": "Ubuntu",
      "version": "24.04",
      "codename": "noble",
      "kernel": {
        "name": null,
        "version": "6.8.0-1009-oem",
        "signature": null,
        "loaded_modules": []
      }
    }
  ]
}
"""
    elif status == "image-exists":
        response_contents = """{
  "status": "Certified Image Exists",
  "certified_url": "https://ubuntu.com/certified/202405-34051",
  "architecture": "amd64",
  "bios": {
    "vendor": "Dell",
    "version": "1.7.0",
    "revision": null,
    "firmware_revision": "1.9",
    "release_date": null
  },
  "board": {
    "manufacturer": "Dell",
    "product_name": "02395C",
    "version": "X04"
  },
  "chassis": null,
  "available_releases": [
    {
      "distributor": "Ubuntu",
      "version": "24.04",
      "codename": "noble",
      "kernel": {
        "name": null,
        "version": "6.8.0-1009-oem",
        "signature": null,
        "loaded_modules": []
      }
    }
  ]
}
"""
    elif status == "system-exists":
        response_contents = """{
  "status": "Related Certified System Exists",
  "certified_url": "https://ubuntu.com/certified/202405-34051",
  "architecture": "amd64",
  "bios": {
    "vendor": "Dell",
    "version": "1.7.0",
    "revision": null,
    "firmware_revision": "1.9",
    "release_date": null
  },
  "board": {
    "manufacturer": "Dell",
    "product_name": "02395C",
    "version": "X04"
  },
  "chassis": null,
  "available_releases": [
    {
      "distributor": "Ubuntu",
      "version": "24.04",
      "codename": "noble",
      "kernel": {
        "name": null,
        "version": "6.8.0-1009-oem",
        "signature": null,
        "loaded_modules": []
      }
    }
  ],
  "pci_peripherals": [],
  "usb_peripherals": []
}
"""
    elif status == "error-data":
        response_contents = """{
  "status": "Related Certified System Exists",
  "certified_url": "https://ubuntu.com/certified/202405-34051",
  "architecture": "amd64",
  "bios": {
    "vendor": "Dell",
    "version": "1.7.0",
    "revision": null,
    "firmware_revision": "1.9",
    "release_date": null
  },
  "board": {
    "manufacturer": "Dell",
    "product_name": "02395C",
    "version": "X04"
  },
  "chassis": null,
  "available_releases": [
    {
      "distributor": "Ubuntu",
      "version": "24.04",
      "codename": "noble",
      "kernel": {
        "name": null,
        "version": "6.8.0-1009-oem",
        "signature": null,
        "loaded_modules": []
      }
    }
  ],
  "pci_peripherals": [],
  "usb_peripherals": [],
}
"""
    elif status == "error":
        response_code = 500
        response_contents = """{
  "status": "Error"
}"""
    with http.server.HTTPServer(("", port), handler) as httpd:
        print(f"Serving on port {port} with status '{status}'...")
        httpd.serve_forever()


if __name__ == "__main__":
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print("Usage: python3 certification_emulator.py <status> [port]")
        print(
            " with <status> being one of: certified, not-certified, image-exists, system-exists, error, error-data"
        )
        print(" and [port] being an optional port number (default is 8000)")
        sys.exit(1)
    run_server()
