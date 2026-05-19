"""搜索命令"""
import json as json_lib
from typing import Optional
import click
from rich.console import Console

from skillhub.core.api_client import SkillAPIClient
from skillhub.utils.console import create_skill_table, truncate_desc
from skillhub.utils.reference import refresh_command_reference


@click.command()
@click.argument("keyword", required=False)
@click.option("--limit", "-l", type=int, default=20, help="返回数量")
@click.option("--page", "-p", type=int, default=1, help="页码")
@click.option("--json", "output_json", is_flag=True, help="JSON 格式输出")
@click.pass_context
def search(
    ctx: click.Context,
    keyword: Optional[str],
    limit: int,
    page: int,
    output_json: bool
) -> None:
    """搜索 Skill（无需登录）
    
    示例:
        kdskillhub search          # 列出所有 Skill
        kdskillhub search pdf      # 搜索 PDF 相关
    """
    api_endpoint = ctx.obj.get("api_endpoint")
    api = SkillAPIClient(api_endpoint)
    try:
        result = api.search_skills(keyword=keyword, page=page, page_size=limit)
        
        if result.get("code") != 200:
            if output_json:
                click.echo(json_lib.dumps({"error": result.get("message")}, ensure_ascii=False))
            else:
                console = Console()
                console.print(f"[bold red]✗ 搜索失败:[/] {result.get('message')}")
            return
        
        data = result.get("data", {})
        skills = data.get("list", [])
        total = data.get("total", 0)
        
        if not skills:
            if output_json:
                click.echo(json_lib.dumps({"skills": [], "total": 0}, ensure_ascii=False))
            else:
                click.echo("未找到匹配的 Skill")
            return
        
        if output_json:
            out = [{
                "displayName": s.get("displayName") or s.get("name", "-"),
                "name": s.get("name", "-"),
                "description": s.get("description", "") or "-",
                "version": s.get("currentVersion") or s.get("version") or "-"
            } for s in skills]
            click.echo(json_lib.dumps({"skills": out, "total": total}, ensure_ascii=False))
            return
        
        # 使用真实终端宽度；Rich 会按列的 ratio 同步缩放
        console = Console()
        search_hint = f'"{keyword}"' if keyword else "所有"
        console.print(f"[bold green]搜索成功！以下是 {search_hint} 相关的主要技能：[/]\n")
        
        table = create_skill_table()
        
        for skill in skills:
            table.add_row(
                skill.get("displayName") or skill.get("name", "-"),
                skill.get("name", "-"),
                truncate_desc(skill.get("description", "")),
                skill.get("currentVersion") or skill.get("version") or "-",
            )
        
        console.print(table)
        console.print(f"\n如需安装某个技能，可以使用 [bold]kdskillhub install <名称/标识>[/]。")
        
    except Exception as e:
        if output_json:
            click.echo(json_lib.dumps({"error": str(e)}, ensure_ascii=False))
        else:
            console = Console()
            console.print(f"[bold red]✗ 搜索失败:[/] {e}")
    finally:
        # 同步服务器上的最新 COMMAND_REFERENCE.md，失败静默不提示
        refresh_command_reference(api)
