#!/usr/bin/python3

import os
import hashlib
import json

def calculate_sha256sum(file_path):
    """Calculate the SHA-256 checksum of a file."""
    sha256 = hashlib.sha256()
    with open(file_path, 'rb') as f:
        while chunk := f.read(8192):
            sha256.update(chunk)
    return sha256.hexdigest()

def generate_checksums(root_dir):
    """Generate a dictionary of file paths and their SHA-256 checksums."""
    checksums = {}
    for dirpath, dirnames, filenames in os.walk(root_dir):
        # Exclude debian/ and vendor/ directories
        if 'debian' in dirnames:
            dirnames.remove('debian')
        if 'vendor' in dirnames:
            dirnames.remove('vendor')

        for filename in filenames:
            file_path = os.path.join(dirpath, filename)
            relative_path = os.path.relpath(file_path, root_dir)
            checksums[relative_path] = calculate_sha256sum(file_path)
    return checksums

def main():
    root_dir = '.'
    checksums = generate_checksums(root_dir)

    cargo_checksum = {"files": checksums}

    with open('debian/cargo-checksum.json', 'w') as f:
        json.dump(cargo_checksum, f, separators=(',', ':'))

    print("debian/cargo-checksum.json has been generated.")

if __name__ == "__main__":
    main()
