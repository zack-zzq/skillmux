#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import stat
import tarfile
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


def add_tar_file(archive: tarfile.TarFile, source: Path, arcname: str, mode: int | None = None) -> None:
    info = archive.gettarinfo(str(source), arcname)
    if mode is not None:
        info.mode = mode
    with source.open("rb") as handle:
        archive.addfile(info, handle)


def write_tar_gz(output: Path, binary: Path, binary_arcname: str) -> None:
    with tarfile.open(output, "w:gz") as archive:
        add_tar_file(archive, binary, binary_arcname, stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR | stat.S_IRGRP | stat.S_IXGRP | stat.S_IROTH | stat.S_IXOTH)
        for extra in ("README.md", "LICENSE"):
            path = ROOT / extra
            if path.is_file():
                add_tar_file(archive, path, extra)


def write_zip(output: Path, binary: Path, binary_arcname: str) -> None:
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.write(binary, binary_arcname)
        for extra in ("README.md", "LICENSE"):
            path = ROOT / extra
            if path.is_file():
                archive.write(path, extra)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Package a compiled CLI binary as a release asset.")
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

    asset_stem = f"{package_name}-{version}-{args.target}"
    binary_arcname = binary_name(package_name, args.target)
    if "windows" in args.target:
        output = output_dir / f"{asset_stem}.zip"
        write_zip(output, binary, binary_arcname)
    else:
        output = output_dir / f"{asset_stem}.tar.gz"
        write_tar_gz(output, binary, binary_arcname)

    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
