"""列出命令"""
import json as json_lib
import click
from rich.console import Console

from skillhub.core.storage import SkillStorage
from skillhub.core.api_client import SkillAPIClient
from skillhub.core.config import get_enabled_ides, get_ide_install_check_path, get_ide_skills_path
from skillhub.utils.console import create_skill_table, truncate_desc
from skillhub.utils.reference import refresh_command_reference


@click.command(name="list")
@click.option("--json", "output_json", is_flag=True, help="JSON 格式输出")
@click.pass_context
def list(ctx: click.Context, output_json: bool) -> None:
    """列出已安装的 Skill"""
    cfg = ctx.obj["config"]
    api_endpoint = ctx.obj.get("api_endpoint") or cfg.get("api.endpoint")
    api = SkillAPIClient(api_endpoint)

    try:
        skills = []

        enabled_ides = get_enabled_ides(cfg)
        for ide in enabled_ides:
            check_path = get_ide_install_check_path(ide)
            if not check_path.exists():
                continue
            ide_storage = SkillStorage(get_ide_skills_path(ide))
            for skill in ide_storage.list_installed_skills():
                skill_item = dict(skill)
                skill_item["_installed_in"] = ide
                skills.append(skill_item)

        # 按 skill name 去重，优先保留先扫描到的条目
        deduped_skills = []
        seen_names = set()
        for skill in skills:
            name = skill.get("name")
            if not name or name in seen_names:
                continue
            deduped_skills.append(skill)
            seen_names.add(name)

        skills = deduped_skills

        if not skills:
            if output_json:
                click.echo(json_lib.dumps({"skills": [], "total": 0}, ensure_ascii=False))
            else:
                click.echo("尚未安装任何 Skill")
            return

        if output_json:
            out = [{
                "displayName": s.get("displayName") or s.get("name", "-"),
                "name": s.get("name", "-"),
                "description": s.get("description", "") or "-",
                "version": s.get("version", "-"),
                "installed_at": (s.get("installed_at", "-")[:19] if s.get("installed_at") else "-")
            } for s in skills]
            click.echo(json_lib.dumps({"skills": out, "total": len(out)}, ensure_ascii=False))
            return

        console = Console()
        table = create_skill_table(
            ("安装时间", "blue", {"no_wrap": False, "overflow": "fold", "min_width": 10, "ratio": 2}),
        )

        for skill in skills:
            table.add_row(
                skill.get("displayName") or skill.get("name", "-"),
                skill.get("name", "-"),
                truncate_desc(skill.get("description", "")),
                skill.get("version", "-"),
                skill.get("installed_at", "-")[:19] if skill.get("installed_at") else "-"
            )

        console.print(table)
        click.echo(f"\n共 {len(skills)} 个 Skill")
    finally:
        # 同步服务器上的最新 COMMAND_REFERENCE.md，失败静默不提示
        refresh_command_reference(api)
