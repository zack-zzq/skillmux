"""Skillhub CLI 入口点"""
import os
import sys

# 强制 UTF-8 编码，解决 Windows 终端中文乱码问题
os.environ.setdefault("PYTHONIOENCODING", "utf-8")
if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

from skillhub.cli import cli

if __name__ == "__main__":
    cli()
