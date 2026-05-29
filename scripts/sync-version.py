#!/usr/bin/env python3
"""Synchronize Shuttle bundle versions with Cargo.toml."""

from __future__ import annotations

import argparse
import plistlib
import re
import sys
from pathlib import Path


VERSION_RE = re.compile(r'^version\s*=\s*"([^"]+)"\s*(?:#.*)?$')


def read_cargo_version(cargo_toml: Path) -> str:
    in_package = False
    for line in cargo_toml.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            in_package = False
        if in_package:
            match = VERSION_RE.match(stripped)
            if match:
                return match.group(1)
    raise ValueError(f"could not find package.version in {cargo_toml}")


def read_plist_versions(plist_path: Path) -> tuple[str | None, str | None]:
    with plist_path.open("rb") as plist_file:
        plist = plistlib.load(plist_file)
    return plist.get("CFBundleShortVersionString"), plist.get("CFBundleVersion")


def sync_plist_versions(plist_path: Path, version: str) -> bool:
    with plist_path.open("rb") as plist_file:
        plist = plistlib.load(plist_file)

    changed = False
    for key in ("CFBundleShortVersionString", "CFBundleVersion"):
        if plist.get(key) != version:
            plist[key] = version
            changed = True

    if changed:
        with plist_path.open("wb") as plist_file:
            plistlib.dump(plist, plist_file, sort_keys=False)
    return changed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if versions differ instead of updating the plist")
    parser.add_argument("--cargo-toml", type=Path, default=Path("Cargo.toml"), help="path to Cargo.toml")
    parser.add_argument("--plist", type=Path, default=Path("resources/Shuttle-Info.plist"), help="path to Shuttle Info.plist")
    args = parser.parse_args(argv)

    cargo_version = read_cargo_version(args.cargo_toml)
    short_version, bundle_version = read_plist_versions(args.plist)

    if args.check:
        if short_version == cargo_version and bundle_version == cargo_version:
            print(f"versions match: {cargo_version}")
            return 0
        print(
            "version mismatch: "
            f"Cargo.toml package.version={cargo_version}, "
            f"CFBundleShortVersionString={short_version}, "
            f"CFBundleVersion={bundle_version}",
            file=sys.stderr,
        )
        return 1

    changed = sync_plist_versions(args.plist, cargo_version)
    if changed:
        print(f"updated {args.plist} to {cargo_version}")
    else:
        print(f"versions already match: {cargo_version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
