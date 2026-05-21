# KDSkillHub CLI

Rust 重写版核心 CLI，通过 PyPI 分发多平台 wheel，安装后命令保持为 `kdskillhub`。

## 安装

```bash
pip install kdskillhub-zack
kdskillhub --help
```

## 配置

默认配置：`~/.config/skillhub/config.yaml`

```yaml
api:
  endpoint: https://skills.kingdee.com/api
  timeout: 30
  token: optional
install:
  targets: [codex, qoder, qoderwork, kiro, workbuddy]
```

Token 优先级：CLI 参数 `--token` > `KDSKILLHUB_TOKEN` > `api.token` > 构建注入默认 token。

## 开发构建

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo run -- search pdf
maturin build --release
pip install dist/*.whl
kdskillhub --help
```

## 发布

发布由 `.github/workflows/publish.yml` 执行：tag `v*` 或手动触发。
构建时需要仓库 Secret：`KDSKILLHUB_DEFAULT_TOKEN`。
