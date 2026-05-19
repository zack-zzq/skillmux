# Skillhub CLI

QoderWork Skill 管理工具 - 从 Skillhub 商店搜索、安装和管理 Skill。

## 特性

- ✅ **无需登录** - 内置默认 token，直接使用
- 🔍 **搜索 Skill** - 按名称搜索 Skillhub 商店
- 📦 **一键安装** - 自动下载并解压到本地
- 📋 **列出已安装** - 查看本地 Skill 列表
- 🔄 **自动更新** - 检查并更新到最新版本
- 🗑️ **轻松卸载** - 移除不需要的 Skill

## 安装

```bash
pip install skillhub-cli
```

## 快速开始

```bash
# 配置后端地址（可选，使用默认）
kdskillhub config set api.endpoint https://skills.kingdee.com/api

# 搜索 Skill
kdskillhub search pdf

# 安装 Skill
kdskillhub install pdf-processing

# 列出已安装
kdskillhub list

# 查看详情
kdskillhub info pdf-processing

# 更新 Skill
kdskillhub update pdf-processing
# 或更新所有
kdskillhub update --all

# 卸载 Skill
kdskillhub remove pdf-processing
```

## 命令参考

### 全局选项

```bash
kdskillhub [OPTIONS] COMMAND [ARGS]...

Options:
  --version, -v       显示版本
  --config, -c PATH   配置文件路径
  --api, -a TEXT      API 端点地址
  --verbose           详细输出
  --help              显示帮助
```

### search - 搜索 Skill

```bash
kdskillhub search [KEYWORD] [OPTIONS]

Options:
  -l, --limit INTEGER   返回数量限制 [默认: 20]
  -p, --page INTEGER    页码 [默认: 1]

示例:
  kdskillhub search          # 列出所有 Skill
  kdskillhub search pdf      # 搜索 PDF 相关
```

### install - 安装 Skill

```bash
kdskillhub install SKILL_IDENTIFIER [OPTIONS]

SKILL_IDENTIFIER 可以是:
  skill-name           # Skill 名称
  skill-name@1.2.0    # 名称+版本
  12345               # Skill ID

Options:
  -v, --version TEXT    指定版本
  -f, --force          强制重新安装

示例:
  kdskillhub install pdf-processing
  kdskillhub install pdf-processing@1.2.0
  kdskillhub install 12345
```

### list - 列出已安装

```bash
kdskillhub list [OPTIONS]

显示已安装的 Skill 列表及版本信息
```

### info - 查看详情

```bash
kdskillhub info SKILL_IDENTIFIER

示例:
  kdskillhub info pdf-processing
  kdskillhub info 12345
```

### update - 更新 Skill

```bash
kdskillhub update [SKILL_NAME] [OPTIONS]

Options:
  --all    更新所有 Skill

示例:
  kdskillhub update pdf-processing    # 更新指定 Skill
  kdskillhub update --all             # 更新所有 Skill
```

### remove - 卸载 Skill

```bash
kdskillhub remove SKILL_NAME [OPTIONS]

Options:
  -y, --yes    跳过确认

示例:
  kdskillhub remove pdf-processing
  kdskillhub remove pdf-processing --yes
```

### config - 配置管理

```bash
# 列出配置
kdskillhub config list

# 获取配置项
kdskillhub config get KEY

# 设置配置项
kdskillhub config set KEY VALUE
```

## 配置

配置文件位置: `~/.config/skillhub/config.yaml`

```yaml
api:
  endpoint: https://skills.kingdee.com/api
  timeout: 30

storage:
  path: ~/.qoderwork/skills
```

## Skill 存储位置

安装的 Skill 默认存储在:
```
~/.qoderwork/skills/
├── pdf-processing/
│   ├── SKILL.md
│   ├── metadata.json
│   └── .skillhub/info.json
├── excel-analysis/
│   └── ...
```

## 开发

```bash
# 克隆仓库
git clone https://github.com/kingdee/skillhub-cli.git
cd skillhub-cli

# 创建虚拟环境
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# 安装依赖
pip install -e ".[dev]"

# 运行测试
pytest
```

## License

MIT License

## Development with uv

This recovered repository is configured for [uv](https://docs.astral.sh/uv/).

```bash
uv sync --group dev
uv run kdskillhub --help
```

Build a wheel locally:

```bash
uv build
```

