# Skillhub CLI

Skillhub CLI 是一个用于管理 Skillhub 商店技能的命令行工具，支持搜索、安装、更新和卸载技能，并可按 IDE/工具类型分发安装目录。

## 特性

- ✅ 无需登录即可使用
- 🔍 搜索远端 Skill
- 📦 安装 Skill（支持多目标安装）
- 📋 列出本地已安装 Skill
- 🔄 更新 Skill（支持 `--all` 一键更新全部已安装 Skill）
- 🗑️ 卸载 Skill
- ⚙️ 可配置安装目标（如 `codex`、`qoder`、`qoderwork`、`kiro`、`workbuddy`）
- 🚀 提供 GitHub Actions 工作流用于构建并发布到 PyPI

## 安装

```bash
pip install skillhub-cli
```

## 快速开始

```bash
# （可选）配置后端地址
kdskillhub config set api.endpoint https://skills.kingdee.com/api

# 搜索 Skill
kdskillhub search pdf

# 安装 Skill
kdskillhub install pdf-processing

# 查看已安装
kdskillhub list

# 更新指定 Skill
kdskillhub update pdf-processing

# 一键更新所有已安装 Skill（现有能力）
kdskillhub update --all

# 卸载 Skill
kdskillhub remove pdf-processing
```

## 配置安装目标（含 Codex）

可通过配置项选择技能分发到哪些安装目录：

```bash
# 方式 1：通用 set
kdskillhub config set install.targets codex,qoder

# 方式 2：快捷命令
kdskillhub config targets codex,qoder,qoderwork

# 查看当前配置
kdskillhub config list
```

默认支持的目标包括：
- `codex` → `~/.codex/skills`（Windows 对应 `C:\Users\<用户名>\.codex\skills`）
- `qoder` → `~/.qoder/skills`
- `qoderwork` → `~/.qoderwork/skills`
- `kiro` → `~/.kiro/skills`
- `workbuddy` → `~/.workbuddy/skills`

## 常用命令

```bash
kdskillhub search [keyword]
kdskillhub install <skill-name|skill-name@version>
kdskillhub list
kdskillhub update <skill-name>
kdskillhub update --all
kdskillhub remove <skill-name>
kdskillhub config list
kdskillhub config get <key>
kdskillhub config set <key> <value>
kdskillhub config targets <ide1,ide2,...>
```

## 配置文件

默认路径：`~/.config/skillhub/config.yaml`

示例：

```yaml
api:
  endpoint: https://skills.kingdee.com/api
  timeout: 30

storage:
  path: ~/.qoderwork/skills

install:
  targets:
    - codex
    - qoder
    - qoderwork
```

## CI/CD（GitHub Actions）

仓库包含 `publish.yml` 工作流：
- 在推送 `v*` 标签时自动构建并发布到 PyPI
- 支持手动触发（workflow_dispatch）

## Development with uv

```bash
uv sync --group dev
uv run kdskillhub --help
uv run pytest
uv build
```
