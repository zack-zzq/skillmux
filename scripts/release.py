#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
BUMP_ALIASES = {
    "1": "patch",
    "patch": "patch",
    "p": "patch",
    "fix": "patch",
    "bugfix": "patch",
    "small": "patch",
    "小": "patch",
    "小版本": "patch",
    "修订": "patch",
    "修订号": "patch",
    "2": "minor",
    "minor": "minor",
    "min": "minor",
    "次": "minor",
    "次版本": "minor",
    "次版本号": "minor",
    "3": "major",
    "major": "major",
    "maj": "major",
    "big": "major",
    "大": "major",
    "大版本": "major",
    "主版本": "major",
    "主版本号": "major",
}


@dataclass(frozen=True)
class VersionTarget:
    path: Path
    section: str


VERSION_TARGETS = (
    VersionTarget(ROOT / "Cargo.toml", "package"),
    VersionTarget(ROOT / "pyproject.toml", "project"),
)


class ReleaseError(RuntimeError):
    pass


def split_line_ending(line: str) -> tuple[str, str]:
    if line.endswith("\r\n"):
        return line[:-2], "\r\n"
    if line.endswith("\n"):
        return line[:-1], "\n"
    return line, ""


def read_lines(path: Path) -> list[str]:
    return path.read_bytes().decode("utf-8").splitlines(keepends=True)


def find_version_line(target: VersionTarget, lines: list[str]) -> tuple[int, str]:
    header = f"[{target.section}]"
    start = None
    for index, line in enumerate(lines):
        body, _ = split_line_ending(line)
        if body.strip() == header:
            start = index + 1
            break

    if start is None:
        raise ReleaseError(f"Missing {header} section in {target.path.relative_to(ROOT)}")

    end = len(lines)
    for index in range(start, len(lines)):
        body, _ = split_line_ending(lines[index])
        stripped = body.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            end = index
            break

    version_line_re = re.compile(r'^(\s*version\s*=\s*")([^"]+)(".*)$')
    for index in range(start, end):
        body, _ = split_line_ending(lines[index])
        match = version_line_re.match(body)
        if match:
            return index, match.group(2)

    raise ReleaseError(f"Missing version in {target.path.relative_to(ROOT)} [{target.section}]")


def read_version(target: VersionTarget) -> str:
    lines = read_lines(target.path)
    _, version = find_version_line(target, lines)
    return version


def write_version(target: VersionTarget, expected: str, new_version: str) -> None:
    lines = read_lines(target.path)
    index, current = find_version_line(target, lines)
    if current != expected:
        rel = target.path.relative_to(ROOT)
        raise ReleaseError(f"{rel} changed while releasing: expected {expected}, found {current}")

    body, ending = split_line_ending(lines[index])
    match = re.match(r'^(\s*version\s*=\s*")([^"]+)(".*)$', body)
    if not match:
        raise ReleaseError(f"Could not rewrite version line in {target.path.relative_to(ROOT)}")

    lines[index] = f"{match.group(1)}{new_version}{match.group(3)}{ending}"
    target.path.write_bytes("".join(lines).encode("utf-8"))


def parse_semver(version: str) -> tuple[int, int, int]:
    match = SEMVER_RE.match(version)
    if not match:
        raise ReleaseError(f"Only plain x.y.z versions are supported, found {version}")
    return tuple(int(part) for part in match.groups())


def bump_version(version: str, bump: str) -> str:
    major, minor, patch = parse_semver(version)
    if bump == "major":
        return f"{major + 1}.0.0"
    if bump == "minor":
        return f"{major}.{minor + 1}.0"
    if bump == "patch":
        return f"{major}.{minor}.{patch + 1}"
    raise ReleaseError(f"Unsupported bump type: {bump}")


def normalize_bump(value: str) -> str:
    key = value.strip().lower()
    bump = BUMP_ALIASES.get(key)
    if bump is None:
        raise ReleaseError("Bump must be one of: patch, minor, major")
    return bump


def prompt_bump(current: str) -> str:
    print(f"Current version: {current}")
    print("Select bump type:")
    print(f"  1) patch / small version  -> {bump_version(current, 'patch')}")
    print(f"  2) minor                  -> {bump_version(current, 'minor')}")
    print(f"  3) major / big version    -> {bump_version(current, 'major')}")
    value = input("Bump [1]: ").strip() or "1"
    return normalize_bump(value)


def cmd_display(args: list[str]) -> str:
    return subprocess.list2cmdline(args)


def run_capture(args: list[str]) -> str:
    completed = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if completed.returncode != 0:
        raise ReleaseError(f"Command failed: {cmd_display(args)}\n{completed.stdout}")
    return completed.stdout.strip()


def run_step(args: list[str], dry_run: bool) -> None:
    print(f"$ {cmd_display(args)}")
    if dry_run:
        return
    completed = subprocess.run(args, cwd=ROOT)
    if completed.returncode != 0:
        raise ReleaseError(f"Command failed: {cmd_display(args)}")


def ensure_command(name: str) -> None:
    if shutil.which(name) is None:
        raise ReleaseError(f"Required command not found on PATH: {name}")


def get_worktree_status() -> str:
    return run_capture(["git", "status", "--porcelain"])


def ensure_clean_worktree() -> None:
    status = get_worktree_status()
    if status:
        raise ReleaseError(
            "Working tree is not clean. Commit or stash changes before running release.\n"
            + status
        )


def ensure_remote(remote: str) -> None:
    run_capture(["git", "remote", "get-url", remote])


def ensure_branch() -> str:
    branch = run_capture(["git", "branch", "--show-current"])
    if not branch:
        raise ReleaseError("Release must run from a branch, not detached HEAD")
    return branch


def ensure_tag_available(remote: str, tag: str) -> None:
    local_tag = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", f"refs/tags/{tag}"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if local_tag.returncode == 0:
        raise ReleaseError(f"Local tag already exists: {tag}")

    remote_tag = subprocess.run(
        ["git", "ls-remote", "--exit-code", "--tags", remote, f"refs/tags/{tag}"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if remote_tag.returncode == 0:
        raise ReleaseError(f"Remote tag already exists: {tag}")
    if remote_tag.returncode != 2:
        raise ReleaseError(f"Could not check remote tag {tag}:\n{remote_tag.stdout}")


def ensure_gh_auth() -> None:
    completed = subprocess.run(
        ["gh", "auth", "status"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if completed.returncode != 0:
        raise ReleaseError("GitHub CLI is not authenticated.\n" + completed.stdout)


def confirm_release(current: str, new_version: str, tag: str, remote: str, branch: str) -> None:
    print()
    print(f"About to release {current} -> {new_version}")
    print(f"Branch: {branch}")
    print(f"Remote: {remote}")
    print(f"Tag:    {tag}")
    answer = input("Continue? [y/N]: ").strip().lower()
    if answer not in {"y", "yes"}:
        raise ReleaseError("Release cancelled")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Bump package versions, commit, tag, push, and create a GitHub release."
        )
    )
    parser.add_argument(
        "bump",
        nargs="?",
        help="Bump type: patch, minor, or major. Chinese aliases like 小版本 and 大版本 are accepted.",
    )
    parser.add_argument("--bump", dest="bump_flag", help="Same as positional bump.")
    parser.add_argument("--yes", "-y", action="store_true", help="Skip confirmation prompt.")
    parser.add_argument("--dry-run", action="store_true", help="Print the planned release steps only.")
    parser.add_argument("--remote", default="origin", help="Git remote to push to. Default: origin.")
    parser.add_argument(
        "--skip-checks",
        action="store_true",
        help="Skip cargo check before committing.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    try:
        ensure_command("git")
        if not args.dry_run:
            ensure_command("gh")

        versions = {target.path.relative_to(ROOT).as_posix(): read_version(target) for target in VERSION_TARGETS}
        unique_versions = set(versions.values())
        if len(unique_versions) != 1:
            details = ", ".join(f"{path}={version}" for path, version in versions.items())
            raise ReleaseError(f"Version files are out of sync: {details}")

        current = unique_versions.pop()
        parse_semver(current)

        bump_values = [value for value in (args.bump, args.bump_flag) if value]
        if len(bump_values) == 2 and normalize_bump(bump_values[0]) != normalize_bump(bump_values[1]):
            raise ReleaseError("Positional bump and --bump disagree")

        bump = normalize_bump(bump_values[0]) if bump_values else prompt_bump(current)
        new_version = bump_version(current, bump)
        tag = f"v{new_version}"
        branch = ensure_branch()

        if args.dry_run:
            status = get_worktree_status()
            if status:
                print("[dry-run] Working tree is dirty; an actual release would stop.")
        else:
            ensure_clean_worktree()
        if args.dry_run:
            print("[dry-run] Skipping GitHub auth and remote tag checks.")
        else:
            ensure_remote(args.remote)
            ensure_tag_available(args.remote, tag)
            ensure_gh_auth()

        if not args.yes and not args.dry_run:
            confirm_release(current, new_version, tag, args.remote, branch)

        print()
        print(f"Release plan: {current} -> {new_version} ({bump})")
        for target in VERSION_TARGETS:
            rel = target.path.relative_to(ROOT).as_posix()
            print(f"Update {rel}")

        if args.dry_run:
            run_step(["cargo", "check"], dry_run=True)
            run_step(["git", "add", "Cargo.toml", "pyproject.toml"], dry_run=True)
            run_step(["git", "commit", "-m", f"chore: release {tag}"], dry_run=True)
            run_step(["git", "tag", "-a", tag, "-m", f"Release {tag}"], dry_run=True)
            run_step(["git", "push", args.remote, f"HEAD:{branch}"], dry_run=True)
            run_step(["git", "push", args.remote, tag], dry_run=True)
            run_step(
                ["gh", "release", "create", tag, "--title", tag, "--generate-notes", "--verify-tag"],
                dry_run=True,
            )
            return 0

        for target in VERSION_TARGETS:
            write_version(target, current, new_version)

        if not args.skip_checks:
            run_step(["cargo", "check"], dry_run=False)

        run_step(["git", "add", "Cargo.toml", "pyproject.toml"], dry_run=False)
        run_step(["git", "commit", "-m", f"chore: release {tag}"], dry_run=False)
        run_step(["git", "tag", "-a", tag, "-m", f"Release {tag}"], dry_run=False)
        run_step(["git", "push", args.remote, f"HEAD:{branch}"], dry_run=False)
        run_step(["git", "push", args.remote, tag], dry_run=False)
        run_step(
            ["gh", "release", "create", tag, "--title", tag, "--generate-notes", "--verify-tag"],
            dry_run=False,
        )

        print(f"Release completed: {tag}")
        return 0
    except ReleaseError as error:
        print(f"release.py: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nrelease.py: cancelled", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
