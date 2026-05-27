# skillmux

**skillmux** 是一个支持多源、多目标安装的 Skill 管理 CLI 工具。

> 理念：**A fast multi-source, multi-target skill manager.**

## 功能
- 按配置源搜索技能（`kingdee` / `clawhub`）。
- 从官方源或 GitHub 安装技能（`gh:owner/repo`、`github:owner/repo`、GitHub URL）。
- 列出本地已安装技能，并显示来源、版本、描述。
- 更新单个技能或全部技能。
- 卸载技能。
- 配置多安装目标（`codex`、`qoder`、`qoderwork`、`kiro`、`workbuddy`）。
- 首次执行时自动把内置操作 skill 安装到配置目标目录。

## 安装
```bash
pip install skillmux
```

## 快速开始
```bash
skillmux search pdf
skillmux install pdf-processing
skillmux list
skillmux update --all
skillmux remove pdf-processing
```

## 内置 bootstrap skills
首次执行 `skillmux` 时，会将仓库 `./skills` 下内置 skill 拷贝到每个已配置 target。
这些 skill 用于指导 agent 在安装、检索、升级、卸载 skill 时应如何操作。
