#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path


TARGETS = {
    "macos_arm": "aarch64-apple-darwin",
    "macos_intel": "x86_64-apple-darwin",
    "linux_arm": "aarch64-unknown-linux-gnu",
    "linux_intel": "x86_64-unknown-linux-gnu",
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def find_asset(assets_dir: Path, version: str, target: str) -> Path:
    name = f"skillmux-{version}-{target}.tar.gz"
    path = assets_dir / name
    if not path.is_file():
        raise SystemExit(f"Missing Homebrew release asset: {path}")
    return path


def render_formula(repo: str, tag: str, version: str, assets_dir: Path) -> str:
    assets = {key: find_asset(assets_dir, version, target) for key, target in TARGETS.items()}
    urls = {
        key: f"https://github.com/{repo}/releases/download/{tag}/{path.name}"
        for key, path in assets.items()
    }
    sums = {key: sha256(path) for key, path in assets.items()}

    return f'''class Skillmux < Formula
  desc "Fast multi-source, multi-target skill manager"
  homepage "https://skills.kingdee.com"
  version "{version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "{urls["macos_arm"]}"
      sha256 "{sums["macos_arm"]}"
    else
      url "{urls["macos_intel"]}"
      sha256 "{sums["macos_intel"]}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "{urls["linux_arm"]}"
      sha256 "{sums["linux_arm"]}"
    else
      url "{urls["linux_intel"]}"
      sha256 "{sums["linux_intel"]}"
    end
  end

  def install
    bin.install "skillmux"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/skillmux --version")
  end
end
'''


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Write the Homebrew formula for the current release.")
    parser.add_argument("--tap-root", required=True, type=Path, help="Checked-out Homebrew tap root")
    parser.add_argument("--assets-dir", required=True, type=Path, help="Directory containing release assets")
    parser.add_argument("--repo", required=True, help="GitHub repository, e.g. owner/repo")
    parser.add_argument("--tag", required=True, help="Release tag, e.g. v3.2.0")
    parser.add_argument("--version", required=True, help="Release version, e.g. 3.2.0")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    formula = render_formula(args.repo, args.tag, args.version, args.assets_dir)
    formula_dir = args.tap_root / "Formula"
    formula_dir.mkdir(parents=True, exist_ok=True)
    formula_path = formula_dir / "skillmux.rb"
    formula_path.write_text(formula, encoding="utf-8", newline="\n")
    print(formula_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
