"""Skillhub CLI 主入口"""
import os
import sys

# 强制 UTF-8 编码，解决 Windows 终端中文乱码问题
os.environ.setdefault("PYTHONIOENCODING", "utf-8")
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

import click
from skillhub.commands import search, install, list_cmd, remove, config, update
from skillhub.core.config import Config


@click.group()
@click.version_option(version="0.1.0", prog_name="kdskillhub")
@click.option("--config", "-c", type=click.Path(), help="配置文件路径")
@click.option("--api", "-a", help="API 端点地址")
@click.option("--verbose", "-v", is_flag=True, help="详细输出")
@click.pass_context
def cli(ctx: click.Context, config: str, api: str, verbose: bool) -> None:
    """Skillhub - QoderWork Skill 管理工具
    
    示例:
        kdskillhub search pdf              # 搜索 PDF 相关 Skill
        kdskillhub install pdf-processing  # 安装 Skill
        kdskillhub list                    # 列出已安装 Skill
    
    更多信息: https://skills.kingdee.com/docs
    """
    ctx.ensure_object(dict)
    
    # 加载配置
    cfg = Config(config_path=config)
    
    ctx.obj["config"] = cfg
    ctx.obj["api_endpoint"] = api or cfg.get("api.endpoint")
    ctx.obj["verbose"] = verbose


# 注册子命令（无需登录）
cli.add_command(search.search)
cli.add_command(install.install)
cli.add_command(list_cmd.list)
cli.add_command(update.update)
cli.add_command(remove.remove)
cli.add_command(config.config)


if __name__ == "__main__":
    cli()
