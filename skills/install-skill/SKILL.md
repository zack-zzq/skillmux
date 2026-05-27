---
name: lifecycle-skill
version: 1.0.0
description: Guidance for skill lifecycle workflow in skillmux.
---

Use `skillmux config list` to inspect targets first.

- Install: `skillmux install <skill|gh:owner/repo> [--version <v>]`
- Search: `skillmux search <keyword>`
- Upgrade: `skillmux update <skill>` or `skillmux update --all`
- Uninstall: `skillmux remove <skill> [--purge]`

After any operation, verify result with `skillmux list`.
