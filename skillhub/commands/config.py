"""配置命令"""
import click
from skillhub.core.config import Config
from skillhub.utils.console import success, info


@click.group()
def config() -> None:
    """管理配置"""
    pass


@config.command(name="list")
@click.pass_context
def config_list(ctx: click.Context) -> None:
    """列出配置"""
    cfg = ctx.obj["config"]
    
    info("当前配置:")
    click.echo(f"  api.endpoint = {cfg.get('api.endpoint')}")
    click.echo(f"  storage.path = {cfg.get('storage.path')}")


@config.command(name="get")
@click.argument("key")
@click.pass_context
def config_get(ctx: click.Context, key: str) -> None:
    """获取配置项"""
    cfg = ctx.obj["config"]
    value = cfg.get(key)
    if value is not None:
        click.echo(value)
    else:
        click.echo(f"配置项 '{key}' 未设置")


@config.command(name="set")
@click.argument("key")
@click.argument("value")
@click.pass_context
def config_set(ctx: click.Context, key: str, value: str) -> None:
    """设置配置项"""
    cfg = ctx.obj["config"]
    cfg.set(key, value)
    cfg.save()
    success(f"已设置 {key} = {value}")
