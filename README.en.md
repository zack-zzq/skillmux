# skillmux

**skillmux** is a CLI tool for managing Skills across multiple sources and multiple local targets.

> Philosophy: **A fast multi-source, multi-target skill manager.**

## Features
- Search skills from configured source (`kingdee` / `clawhub`).
- Install skills from official source or GitHub (`gh:owner/repo`, `github:owner/repo`, GitHub URL).
- List installed skills with source, version and description.
- Update one skill or all installed skills.
- Remove skills.
- Configure multiple install targets (`codex`, `qoder`, `qoderwork`, `kiro`, `workbuddy`).
- Built-in operation skills auto-bootstrapped to configured targets on first run.

## Install
```bash
pip install skillmux
```

## Quick Start
```bash
skillmux search pdf
skillmux install pdf-processing
skillmux list
skillmux update --all
skillmux remove pdf-processing
```

## Built-in bootstrap skills
On the first `skillmux` execution, bundled skills in `./skills` are copied to every configured install target.
These skills explain how agents should handle install/search/update/remove workflows.
