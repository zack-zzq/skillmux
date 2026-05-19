"""更新命令 - 跨 qoder / qoderwork / kiro 三目录同步更新"""
import io
import json as json_lib
import os
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import click
from rich.console import Console
from rich.table import Table
from rich import box

from skillhub.core.api_client import SkillAPIClient
from skillhub.core.storage import SkillStorage
from skillhub.core.config import ALL_IDES, get_ide_install_check_path, get_ide_skills_path
from skillhub.utils.console import success, error, info
from skillhub.utils.reference import refresh_command_reference


@click.command()
@click.argument("skill_name", required=False)
@click.option("--all", "update_all", is_flag=True, help="更新所有 Skill")
@click.option("--check", "check_only", is_flag=True, help="仅检查最新版本，不执行更新")
@click.option("--json", "output_json", is_flag=True, help="JSON 格式输出")
@click.pass_context
def update(
    ctx: click.Context,
    skill_name: Optional[str],
    update_all: bool,
    check_only: bool,
    output_json: bool
) -> None:
    """更新 Skill（同时作用于 qoder / qoderwork / kiro / workbuddy 四个目录）

    规则：
        - 若某 IDE/工具 检查路径不存在，则跳过该 IDE/工具
          （workbuddy 要求 ~/.workbuddy/skills 存在）
        - 某 IDE 下对应技能已被用户手动删除，则跳过该 IDE 的这条技能，
          但其他仍有该技能的 IDE 照常更新

    示例:
        kdskillhub update pdf-processing    # 更新指定 Skill
        kdskillhub update --all             # 更新所有 Skill
        kdskillhub update --check           # 查询所有已安装 Skill 的最新版本
    """
    api_endpoint = ctx.obj.get("api_endpoint") or ctx.obj["config"].get("api.endpoint")
    api = SkillAPIClient(api_endpoint)
    try:
        # 收集所有存在的 IDE/工具 的 storage
        ide_storages: List[Tuple[str, SkillStorage]] = []
        skipped_ides: List[str] = []
        for ide in ALL_IDES:
            check_path = get_ide_install_check_path(ide)
            if not check_path.exists():
                skipped_ides.append(ide)
                continue
            ide_storages.append((ide, SkillStorage(get_ide_skills_path(ide))))

        if not ide_storages:
            error("未检测到 qoder / qoderwork / kiro / workbuddy 任一可更新目标")
            raise click.Abort()

        if skipped_ides and not output_json and not check_only:
            info(f"已跳过未安装的 IDE/工具: {', '.join(skipped_ides)}")

        # skill_ide_map: name -> [(ide, storage, info)]
        skill_ide_map: Dict[str, List[Tuple[str, SkillStorage, Dict]]] = {}
        for ide, storage in ide_storages:
            for info_item in storage.list_installed_skills():
                name = info_item.get("name")
                if not name:
                    continue
                skill_ide_map.setdefault(name, []).append((ide, storage, info_item))

        # --check 默认检查所有已安装 Skill
        if check_only and not skill_name:
            update_all = True

        # 确定要处理的技能名列表
        if update_all or check_only:
            target_names = list(skill_ide_map.keys())
        elif skill_name:
            if skill_name in skill_ide_map:
                target_names = [skill_name]
            else:
                # 按 displayName 匹配
                matched = [
                    n for n, items in skill_ide_map.items()
                    if any(it[2].get("displayName") == skill_name for it in items)
                ]
                if matched:
                    target_names = matched
                else:
                    error(f"Skill 未安装: {skill_name}")
                    raise click.Abort()
        else:
            click.echo("请指定 Skill 名称或使用 --all 更新所有")
            raise click.Abort()

        if not target_names:
            info("没有已安装的 Skill")
            return

        # 处理每个技能
        updated_count = 0
        results = []
        for name in target_names:
            entries = skill_ide_map.get(name, [])
            if not entries:
                continue

            # 以第一个条目的元数据作为展示参考
            display_name = entries[0][2].get("displayName") or name
            # 各 IDE 的当前版本（可能不同）
            current_versions = {ide: info_item.get("version") for ide, _, info_item in entries}
            representative_current = current_versions[entries[0][0]]

            # 远程查询
            try:
                result = api.search_skills(keyword=name, page_size=20)
            except Exception as e:
                error(f"{'检查' if check_only else '更新'} {name} 失败: {e}")
                results.append(_result_row(name, display_name, representative_current, "-", "error", str(e), current_versions))
                continue

            if result.get("code") != 200:
                results.append(_result_row(name, display_name, representative_current, "-", "error",
                                           result.get("message", "查询失败"), current_versions))
                continue

            remote_skills = result.get("data", {}).get("list", [])
            matched = [s for s in remote_skills if s.get("name") == name]
            if not matched:
                if not check_only:
                    error(f"未找到与标识 {name} 匹配的远程 Skill")
                results.append(_result_row(name, display_name, representative_current, "-", "not_found", None, current_versions))
                continue

            remote_skill = matched[0]
            latest_version = remote_skill.get("currentVersion") or remote_skill.get("version")
            skill_id = remote_skill.get("id")
            display_name = remote_skill.get("displayName") or name
            description = remote_skill.get("description", "")

            # 哪些 IDE 需要更新
            pending: List[Tuple[str, SkillStorage, Optional[str]]] = []
            already: List[str] = []
            for ide, storage, info_item in entries:
                if info_item.get("version") == latest_version:
                    already.append(ide)
                else:
                    pending.append((ide, storage, info_item.get("version")))

            if not pending:
                if not check_only:
                    info(f"{name} 已是最新版本 ({latest_version})，覆盖 IDE: {', '.join(already)}")
                results.append(_result_row(name, display_name, representative_current, latest_version,
                                           "up_to_date", None, current_versions, already=already))
                continue

            # --check 模式：只登记可更新
            if check_only:
                results.append(_result_row(name, display_name, representative_current, latest_version,
                                           "updatable", None, current_versions,
                                           pending=[p[0] for p in pending], already=already))
                updated_count += 1
                continue

            # 下载一次，分发到多个 IDE
            info(f"更新 {name} -> {latest_version}（目标 IDE: {', '.join(p[0] for p in pending)}）")
            try:
                zip_data = api.download_skill(skill_id, latest_version)
            except Exception as e:
                error(f"下载 {name}@{latest_version} 失败: {e}")
                results.append(_result_row(name, display_name, representative_current, latest_version,
                                           "error", str(e), current_versions))
                continue

            if not zip_data:
                error(f"下载 {name}@{latest_version} 失败（空响应）")
                results.append(_result_row(name, display_name, representative_current, latest_version,
                                           "error", "empty response", current_versions))
                continue

            done_ides: List[str] = []
            failed_ides: List[Dict] = []
            for ide, storage, cur_ver in pending:
                try:
                    # 再次校验该 IDE 下技能是否仍存在（防止并发期间被删）
                    if not storage.is_installed(name):
                        # 用户删除了 → 跳过该 IDE
                        continue
                    skill_dir = str(storage.get_skill_path(name))
                    storage.remove_skill(name)
                    _extract_zip(zip_data, skill_dir)
                    storage.save_skill_info(skill_id, {
                        "id": skill_id,
                        "name": name,
                        "displayName": display_name,
                        "description": description,
                        "version": latest_version,
                        "installed_at": datetime.now().isoformat()
                    })
                    success(f"  ✓ {ide}: {cur_ver or '-'} -> {latest_version}")
                    done_ides.append(ide)
                except Exception as e:
                    error(f"  ✗ {ide} 更新失败: {e}")
                    failed_ides.append({"ide": ide, "error": str(e)})

            if done_ides:
                updated_count += 1
            results.append(_result_row(
                name, display_name, representative_current, latest_version,
                "updated" if done_ides else ("error" if failed_ides else "up_to_date"),
                None, current_versions,
                updated=done_ides, already=already, failed=failed_ides,
            ))

        # --check 模式输出
        if check_only:
            if output_json:
                click.echo(json_lib.dumps({
                    "results": results,
                    "updatable_count": updated_count,
                    "skipped_ides": skipped_ides,
                }, ensure_ascii=False))
                return

            _render_check_table(results, updated_count, skipped_ides)
            return

        # 普通更新模式输出
        if updated_count == 0:
            info("所有 Skill 都是最新版本")
        else:
            success(f"共更新 {updated_count} 个 Skill")

        if output_json:
            click.echo(json_lib.dumps({
                "results": results,
                "updated_count": updated_count,
                "skipped_ides": skipped_ides,
            }, ensure_ascii=False))

    except click.Abort:
        raise
    except Exception as e:
        if ctx.obj.get("verbose"):
            raise
        error(f"更新失败: {e}")
        raise click.Abort()
    finally:
        # 同步服务器上的最新 COMMAND_REFERENCE.md，失败静默不提示
        refresh_command_reference(api)


def _result_row(
    name: str,
    display_name: str,
    current_version: Optional[str],
    latest_version: str,
    status: str,
    message: Optional[str],
    current_versions: Dict[str, Optional[str]],
    updated: Optional[List[str]] = None,
    already: Optional[List[str]] = None,
    pending: Optional[List[str]] = None,
    failed: Optional[List[Dict]] = None,
) -> Dict:
    row = {
        "name": name,
        "displayName": display_name,
        "currentVersion": current_version or "-",
        "latestVersion": latest_version or "-",
        "status": status,
        "perIdeCurrentVersion": current_versions,
    }
    if updated is not None:
        row["updated"] = updated
    if already is not None:
        row["already"] = already
    if pending is not None:
        row["pending"] = pending
    if failed is not None:
        row["failed"] = failed
    if message:
        row["message"] = message
    return row


def _render_check_table(results: List[Dict], updatable_count: int, skipped_ides: List[str]) -> None:
    console = Console()
    if not results:
        console.print("没有已安装的 Skill")
        return

    table = Table(
        show_header=True,
        header_style="bold cyan",
        box=box.HORIZONTALS,
        show_edge=False,
        pad_edge=False,
        show_lines=True,
        expand=True,
    )
    table.add_column("名称", style="green", no_wrap=False, overflow="fold", min_width=6, ratio=2)
    table.add_column("标识", style="cyan", no_wrap=False, overflow="fold", min_width=6, ratio=2)
    table.add_column("已安装 IDE (当前版本)", style="white", no_wrap=False, overflow="fold", min_width=14, ratio=4)
    table.add_column("最新版本", justify="right", no_wrap=False, overflow="fold", min_width=8, ratio=1)

    for r in results:
        status = r.get("status")
        latest_ver = r.get("latestVersion", "-")

        if status == "updatable":
            latest_ver_text = f"[bold magenta]{latest_ver}[/]"
        elif status == "up_to_date":
            latest_ver_text = f"[green]{latest_ver}[/]"
        elif status == "not_found":
            latest_ver_text = "[dim]-[/]"
        else:
            latest_ver_text = f"[red]{latest_ver}[/]"

        per_ide = r.get("perIdeCurrentVersion") or {}
        per_ide_text = ", ".join(f"{ide}={ver or '-'}" for ide, ver in per_ide.items()) or "-"

        table.add_row(
            r.get("displayName", "-"),
            r.get("name", "-"),
            per_ide_text,
            latest_ver_text,
        )

    console.print("\n[bold]已安装 Skill 版本检查结果（跨 IDE）：[/]\n")
    console.print(table)

    if skipped_ides:
        console.print(f"\n[dim]未安装而跳过的 IDE: {', '.join(skipped_ides)}[/]")

    if updatable_count > 0:
        console.print(f"\n共 {len(results)} 个 Skill，其中 [bold magenta]{updatable_count}[/] 个可更新。")
        console.print("使用 [bold]kdskillhub update --all[/] 更新所有，或 [bold]kdskillhub update <标识>[/] 更新指定 Skill。")
    else:
        console.print(f"\n共 {len(results)} 个 Skill，全部已是最新版本。")


def _extract_zip(zip_data: bytes, dest_dir: str) -> None:
    """解压 ZIP，自动处理多余的外层包装文件夹（复用 install 的逻辑）"""
    import tempfile
    import shutil

    with tempfile.TemporaryDirectory() as tmp_dir:
        with zipfile.ZipFile(io.BytesIO(zip_data), 'r') as zf:
            zf.extractall(tmp_dir)

        content_dir = _find_skill_content_dir(tmp_dir)

        os.makedirs(dest_dir, exist_ok=True)
        for item in os.listdir(content_dir):
            src = os.path.join(content_dir, item)
            dst = os.path.join(dest_dir, item)
            if os.path.isdir(src):
                shutil.copytree(src, dst, dirs_exist_ok=True)
            else:
                shutil.copy2(src, dst)


def _find_skill_content_dir(extracted_dir: str) -> str:
    if os.path.isfile(os.path.join(extracted_dir, "SKILL.md")):
        return extracted_dir

    subdirs = [
        d for d in os.listdir(extracted_dir)
        if os.path.isdir(os.path.join(extracted_dir, d))
        and not d.startswith("__") and not d.startswith(".")
    ]

    if len(subdirs) == 1:
        return _find_skill_content_dir(os.path.join(extracted_dir, subdirs[0]))

    return extracted_dir
