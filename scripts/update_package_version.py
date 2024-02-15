#!/usr/bin/python3
"""A script for updating cargo package version and debian files as well."""

import re
import os.path
from pathlib import Path
import datetime
import argparse
import toml


def update_cargo_toml_version(file_path, new_version):
    """Update the version in the Cargo.toml file"""
    with open(file_path, "r", encoding="utf-8") as toml_file:
        cargo_data = toml.load(toml_file)

    cargo_data["package"]["version"] = new_version

    with open(file_path, "w", encoding="utf-8") as toml_file:
        toml.dump(cargo_data, toml_file)


def update_debian_control_version(file_path, new_version):
    """Update debian/control file to replace old version numbers with new_version."""
    with open(file_path, "r", encoding="utf-8") as control_file:
        content = control_file.read()

    major, minor, patch = new_version.split(".")

    patterns = {
        r"hwlib-\d+-dev": f"hwlib-{major}-dev",
        r"hwlib-\d+\+\w+-dev": f"hwlib-{major}+default-dev",
        r"hwlib-\d\.\d+-dev": f"hwlib-{major}.{minor}-dev",
        r"hwlib-\d\.\d+\+\w+-dev": f"hwlib-{major}.{minor}+default-dev",
        r"hwlib-\d\.\d\.\d+-dev": f"hwlib-{major}.{minor}.{patch}-dev",
        r"hwlib-\d\.\d\.\d+\+\w+-dev": f"hwlib-{major}.{minor}.{patch}+default-dev",
    }

    for old_pattern, new_replacement in patterns.items():
        content = re.sub(old_pattern, new_replacement, content)

    with open(file_path, "w", encoding="utf-8") as control_file:
        control_file.write(content)


def update_debian_tests_control_version(file_path, new_version, package_name):
    """Update debian/tests/control file"""
    with open(file_path, "r", encoding="utf-8") as control_file:
        content = control_file.read()

    updated_content = re.sub(
        rf"{package_name} \d+\.\d+\.\d+", f"{package_name} {new_version}", content
    )

    with open(file_path, "w", encoding="utf-8") as control_file:
        control_file.write(updated_content)


def update_debian_changelog(file_path, package_name, new_version, email, distribution):
    """Update the debian/changelog file"""
    date = datetime.datetime.now().strftime("%a, %d %b %Y %H:%M:%S %z")
    entry = (
        f"{package_name} ({new_version}) {distribution}; urgency=medium\n\n  "
        f"* Team upload.\n  * Package {package_name} {new_version}\n\n -- "
        f"{email}  {date}\n\n"
    )

    with open(file_path, "r", encoding="utf-8") as control_file:
        content = control_file.read()

    with open(file_path, "w", encoding="utf-8") as control_file:
        control_file.write(entry + content)


def valid_file_path(path: str) -> Path:
    if os.path.exists(path):
        return Path(path)
    raise argparse.ArgumentTypeError(f"{path} is not a valid file path")


def valid_version(version):
    """A new version must be like X.Y.Z or X.Y.Z~devN"""
    if re.match(r"^\d+\.\d+\.\d+(~dev\d+)?$", version):
        return version
    raise argparse.ArgumentTypeError("A new version must be like X.Y.Z or X.Y.Z~devN")


def main():
    parser = argparse.ArgumentParser(
        description="Update version information in project files."
    )
    parser.add_argument(
        "new_version", help="The new version number to apply.", type=valid_version
    )
    parser.add_argument("email", help="The email address for the changelog entry.")
    parser.add_argument(
        "--distribution",
        default="mantic",
        help='The distribution name for the changelog. Default is "mantic".',
    )
    parser.add_argument(
        "--cargo-file",
        default="Cargo.toml",
        help='Path to the Cargo.toml file. Default is "Cargo.toml".',
        type=valid_file_path,
    )
    parser.add_argument(
        "--control-file",
        default="debian/control",
        help='Path to the debian control file. Default is "debian/control".',
        type=valid_file_path,
    )
    parser.add_argument(
        "--tests-control-file",
        default="debian/tests/control",
        help='Path to the debian tests control file. Default is "debian/tests/control".',
        type=valid_file_path,
    )
    parser.add_argument(
        "--changelog-file",
        default="debian/changelog",
        help='Path to the debian changelog file. Default is "debian/changelog".',
        type=valid_file_path,
    )
    parser.add_argument(
        "--package-name", default="hwlib", help='The package name. Default is "hwlib".'
    )

    args = parser.parse_args()

    # Update versions in files
    update_cargo_toml_version(args.cargo_file, args.new_version)
    update_debian_control_version(args.control_file, args.new_version)
    update_debian_tests_control_version(
        args.tests_control_file, args.new_version, args.package_name
    )
    update_debian_changelog(
        args.changelog_file,
        args.package_name,
        args.new_version,
        args.email,
        args.distribution,
    )


if __name__ == "__main__":
    main()
