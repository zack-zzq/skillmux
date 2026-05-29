#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import shutil
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def parse_package_metadata() -> tuple[str, str]:
    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    package_match = re.search(r"(?ms)^\[package\](.*?)(?:^\[|\Z)", cargo_toml)
    if not package_match:
        raise SystemExit("Cargo.toml is missing a [package] section")

    section = package_match.group(1)
    name_match = re.search(r'(?m)^\s*name\s*=\s*"([^"]+)"', section)
    version_match = re.search(r'(?m)^\s*version\s*=\s*"([^"]+)"', section)
    if not name_match or not version_match:
        raise SystemExit("Cargo.toml [package] must contain name and version")

    return name_match.group(1), version_match.group(1)


def binary_name(package_name: str, target: str) -> str:
    if "windows" in target:
        return f"{package_name}.exe"
    return package_name


def find_binary(package_name: str, target: str) -> Path:
    name = binary_name(package_name, target)
    candidates = [
        ROOT / "target" / target / "release" / name,
        ROOT / "target" / "release" / name,
    ]

    for candidate in candidates:
        if candidate.is_file():
            return candidate

    wheel_binary = extract_binary_from_wheel(package_name, target)
    if wheel_binary is not None:
        return wheel_binary

    searched = "\n".join(f"  - {path}" for path in candidates)
    raise SystemExit(f"Could not find release binary for {target}. Searched:\n{searched}\n  - dist/*.whl .data/scripts")


def extract_binary_from_wheel(package_name: str, target: str) -> Path | None:
    name = binary_name(package_name, target)
    scripts_suffixes = (f".data/scripts/{name}", f".data/scripts/{package_name}", f".data/scripts/{package_name}.exe")

    for wheel in sorted((ROOT / "dist").glob("*.whl")):
        with zipfile.ZipFile(wheel) as archive:
            members = [
                member
                for member in archive.namelist()
                if any(member.endswith(suffix) for suffix in scripts_suffixes)
            ]
            if not members:
                continue

            member = members[0]
            output_dir = ROOT / "target" / "release-assets-extracted" / target
            output_dir.mkdir(parents=True, exist_ok=True)
            output = output_dir / name
            output.write_bytes(archive.read(member))
            if "windows" not in target:
                os.chmod(output, 0o755)
            return output

    return None


def release_asset_name(package_name: str, version: str, target: str) -> str:
    suffix = ".exe" if "windows" in target else ""
    return f"{package_name}-{version}-{target}{suffix}"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Copy a compiled CLI binary as a release asset.")
    parser.add_argument("--target", required=True, help="Rust target triple, e.g. x86_64-apple-darwin")
    parser.add_argument("--out", default="release-assets", help="Output directory")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    package_name, version = parse_package_metadata()
    binary = find_binary(package_name, args.target)

    output_dir = Path(args.out)
    if not output_dir.is_absolute():
        output_dir = ROOT / output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    output = output_dir / release_asset_name(package_name, version, args.target)
    shutil.copy2(binary, output)
    if "windows" not in args.target:
        os.chmod(output, output.stat().st_mode | 0o755)

    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
