"""卸载命令"""
import click

from skillhub.core.storage import SkillStorage
from skillhub.utils.console import success, error, info


@click.command()
@click.argument("skill_name")
@click.option("--yes", "-y", is_flag=True, help="跳过确认")
@click.pass_context
def remove(
    ctx: click.Context,
    skill_name: str,
    yes: bool
) -> None:
    """卸载 Skill
    
    示例:
        kdskillhub remove pdf-processing
        kdskillhub remove pdf-processing --yes
    """
    try:
        cfg = ctx.obj["config"]
        storage = SkillStorage(cfg.get("storage.path"))
        
        # 检查是否已安装
        if not storage.is_installed(skill_name):
            error(f"Skill 未安装: {skill_name}")
            raise click.Abort()
        
        # 确认
        if not yes:
            click.confirm(f"确定要卸载 {skill_name} 吗？", abort=True)
        
        # 执行卸载
        info(f"正在卸载 {skill_name}...")
        storage.remove_skill(skill_name)
        success(f"✓ {skill_name} 已卸载")
            
    except Exception as e:
        if ctx.obj.get("verbose"):
            raise
        error(f"卸载失败: {e}")
        raise click.Abort()
