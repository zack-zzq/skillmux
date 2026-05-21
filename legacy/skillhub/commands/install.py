"""安装命令 - 从数据库下载 ZIP"""
import os
import io
import json as json_lib
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Optional
import click
from rich.console import Console

from skillhub.core.api_client import SkillAPIClient
from skillhub.core.storage import SkillStorage
from skillhub.core.config import get_all_storage_paths, get_ide_install_check_path, get_enabled_ides
from skillhub.utils.console import success, error, info, create_skill_table, truncate_desc


@click.command()
@click.argument("skill_identifier")
@click.option("--version", "-v", help="指定版本")
@click.option("--force", "-f", is_flag=True, help="强制重新安装")
@click.option("--json", "output_json", is_flag=True, help="JSON 格式输出")
@click.pass_context
def install(
    ctx: click.Context,
    skill_identifier: str,
    version: Optional[str],
    force: bool,
    output_json: bool
) -> None:
    """安装 Skill（无需登录）
    
    SKILL_IDENTIFIER 可以是:
        skill-name           # Skill 名称/标识
        skill-name@1.2.0     # 名称+版本
    
    示例:
        kdskillhub install pdf-processing
        kdskillhub install pdf-processing@1.2.0
    """
    try:
        # 解析 identifier
        skill_name, specified_version = _parse_identifier(skill_identifier)
        target_version = version or specified_version
        
        info(f"正在查找 {skill_name}...")
        
        # 初始化
        cfg = ctx.obj["config"]
        api_endpoint = ctx.obj.get("api_endpoint") or cfg.get("api.endpoint")
        
        api = SkillAPIClient(api_endpoint)

        # 同时安装到 qoder / qoderwork / kiro / workbuddy 四个目录，
        # 若对应检查路径不存在则跳过：
        #   - qoder / qoderwork / kiro：检查 ~/.{ide} 根目录
        #   - workbuddy：检查 ~/.workbuddy/skills 目录
        storages = []
        skipped_ides = []
        enabled_ides = get_enabled_ides(cfg)
        for ide, path in zip(enabled_ides, get_all_storage_paths(cfg)):
            check_path = get_ide_install_check_path(ide)
            if not check_path.exists():
                skipped_ides.append(ide)
                continue
            storages.append((ide, SkillStorage(path)))

        if skipped_ides and not output_json:
            info(f"检测到以下 IDE/工具 未安装，已跳过: {', '.join(skipped_ides)}")

        if not storages:
            error(f"未检测到可安装目标（已启用: {', '.join(enabled_ides)}）")
            if output_json:
                click.echo(json_lib.dumps({
                    "action": "no_target",
                    "skipped": skipped_ides
                }, ensure_ascii=False))
            raise click.Abort()

        # 始终通过 search 接口查询
        result = api.search_skills(keyword=skill_name, page_size=20)
        
        if result.get("code") != 200:
            error(f"查询失败: {result.get('message')}")
            raise click.Abort()
        
        skills = result.get("data", {}).get("list", [])
        
        if not skills:
            error(f"未找到匹配的 Skill: {skill_name}")
            raise click.Abort()
        
        # 尝试按标识(name)精确匹配
        exact_match = [s for s in skills if s.get("name") == skill_name]
        
        if len(exact_match) == 1:
            # 精确匹配到唯一一条，直接安装
            skill = exact_match[0]
        elif len(skills) == 1:
            # 只有一条结果，直接安装
            skill = skills[0]
        else:
            # 多条结果，展示表格并提示
            if output_json:
                out = [{
                    "displayName": s.get("displayName") or s.get("name", "-"),
                    "name": s.get("name", "-"),
                    "description": s.get("description", "") or "-",
                    "version": s.get("currentVersion") or s.get("version") or "-"
                } for s in skills]
                click.echo(json_lib.dumps({"action": "choose", "skills": out}, ensure_ascii=False))
            else:
                console = Console()
                console.print(f"\n[bold green]搜索到以下技能：[/]\n")
                _print_skills_table(console, skills)
                console.print(f"\n如需安装某个技能，可以使用 [bold]kdskillhub install <技能名/标识>[/]。")
            return
        
        # 执行安装（同时写入所有存在的 IDE 目录）
        _do_install_all(api, storages, skill, target_version, force, output_json, skipped_ides)
        
    except click.Abort:
        raise
    except Exception as e:
        if ctx.obj.get("verbose"):
            raise
        error(f"安装失败: {e}")
        raise click.Abort()


def _print_skills_table(console: Console, skills: list) -> None:
    """以表格形式展示技能列表"""
    table = create_skill_table()
    
    for skill in skills:
        table.add_row(
            skill.get("displayName") or skill.get("name", "-"),
            skill.get("name", "-"),
            truncate_desc(skill.get("description", "")),
            skill.get("currentVersion") or skill.get("version") or "-",
        )
    
    console.print(table)


def _do_install_all(
    api: SkillAPIClient,
    storages: list,
    skill: dict,
    target_version: Optional[str],
    force: bool,
    output_json: bool = False,
    skipped_ides: Optional[list] = None
) -> None:
    """同时安装到多个 IDE 的 skills 目录。

    仅下载一次 ZIP 包，然后分别解压到每个目录。
    任一目录已安装同版本且未指定 --force 时，则跳过该目录。
    """
    skipped_ides = skipped_ides or []
    skill_id = skill.get("id")
    skill_name = skill.get("name")
    display_name = skill.get("displayName") or skill_name
    description = skill.get("description", "")
    install_version = target_version or skill.get("currentVersion") or skill.get("version")

    # 先判断哪些目录需要安装
    pending = []      # 需要安装的 (ide, storage)
    already = []      # 已安装同版本的 ide 名称
    for ide, storage in storages:
        if not force and storage.is_installed(skill_name):
            installed = storage.get_skill_info(skill_name)
            if installed and installed.get("version") == install_version:
                # 补全 displayName / description
                if not installed.get("displayName") or not installed.get("description"):
                    installed["displayName"] = display_name
                    installed["description"] = description
                    storage.save_skill_info(skill_id, installed)
                already.append(ide)
                continue
        pending.append((ide, storage))

    # 全部已安装，无需下载
    if not pending:
        success(f"{skill_name}@{install_version} 已安装到: {', '.join(already)}")
        if output_json:
            click.echo(json_lib.dumps({
                "action": "already_installed",
                "name": skill_name,
                "displayName": display_name,
                "version": install_version,
                "targets": already,
                "skipped": skipped_ides
            }, ensure_ascii=False))
        return

    # 下载一次
    info(f"正在下载 {skill_name}@{install_version}...")
    zip_data = api.download_skill(skill_id, install_version)
    if not zip_data:
        error("下载失败")
        raise click.Abort()

    installed_targets = []
    failed_targets = []
    for ide, storage in pending:
        try:
            skill_dir = str(storage.get_skill_path(skill_name))
            if storage.is_installed(skill_name):
                storage.remove_skill(skill_name)
            _extract_zip(zip_data, skill_dir)
            storage.save_skill_info(skill_id, {
                "id": skill_id,
                "name": skill_name,
                "displayName": display_name,
                "description": description,
                "version": install_version,
                "installed_at": datetime.now().isoformat()
            })
            info(f"  ✓ {ide}: {skill_dir}")
            installed_targets.append(ide)
        except Exception as e:
            failed_targets.append({"ide": ide, "error": str(e)})
            error(f"  ✗ {ide} 安装失败: {e}")

    if installed_targets:
        all_ok = already + installed_targets
        success(f"{skill_name}@{install_version} 安装成功，已写入: {', '.join(all_ok)}")
    if output_json:
        click.echo(json_lib.dumps({
            "action": "installed",
            "name": skill_name,
            "displayName": display_name,
            "version": install_version,
            "installed": installed_targets,
            "already": already,
            "failed": failed_targets,
            "skipped": skipped_ides
        }, ensure_ascii=False))


def _do_install(
    api: SkillAPIClient,
    storage: SkillStorage,
    skill: dict,
    target_version: Optional[str],
    force: bool,
    output_json: bool = False
) -> None:
    """执行具体安装逻辑"""
    skill_id = skill.get("id")
    skill_name = skill.get("name")
    display_name = skill.get("displayName") or skill_name
    description = skill.get("description", "")
    install_version = target_version or skill.get("currentVersion") or skill.get("version")
    
    # 检查是否已安装
    if not force and storage.is_installed(skill_name):
        installed = storage.get_skill_info(skill_name)
        if installed and installed.get("version") == install_version:
            # 补充更新缺失的 displayName / description
            if not installed.get("displayName") or not installed.get("description"):
                installed["displayName"] = display_name
                installed["description"] = description
                storage.save_skill_info(skill_id, installed)
            success(f"{skill_name}@{install_version} 已安装")
            if output_json:
                click.echo(json_lib.dumps({"action": "already_installed", "name": skill_name, "displayName": display_name, "version": install_version}, ensure_ascii=False))
            return
    
    # 下载并安装
    info(f"正在下载 {skill_name}@{install_version}...")
    
    zip_data = api.download_skill(skill_id, install_version)
    
    if not zip_data:
        error("下载失败")
        raise click.Abort()
    
    # 安装
    skill_dir = storage.get_skill_path(skill_name)
    
    if storage.is_installed(skill_name):
        storage.remove_skill(skill_name)
    
    _extract_zip(zip_data, skill_dir)
    
    # 保存元数据（包含 displayName 和 description）
    storage.save_skill_info(skill_id, {
        "id": skill_id,
        "name": skill_name,
        "displayName": display_name,
        "description": description,
        "version": install_version,
        "installed_at": datetime.now().isoformat()
    })
    
    success(f"{skill_name}@{install_version} 安装成功！")
    if output_json:
        click.echo(json_lib.dumps({"action": "installed", "name": skill_name, "displayName": display_name, "version": install_version}, ensure_ascii=False))


def _parse_identifier(identifier: str) -> tuple:
    """解析 identifier"""
    if "@" in identifier:
        name, ver = identifier.rsplit("@", 1)
        return name.strip(), ver.strip()
    return identifier.strip(), None


def _extract_zip(zip_data: bytes, dest_dir: str) -> None:
    """解压 ZIP，自动处理多余的外层包装文件夹
    
    部分 ZIP 包结构为:
        skill-name-v1.0.0/
            __MACOSX/
            skill-name/
                SKILL.md
                ...
    需要跳过外层包装文件夹和 __MACOSX，取里层的实际 skill 文件夹内容。
    """
    import tempfile
    import shutil
    
    with tempfile.TemporaryDirectory() as tmp_dir:
        with zipfile.ZipFile(io.BytesIO(zip_data), 'r') as zf:
            zf.extractall(tmp_dir)
        
        # 找到实际的 skill 内容目录
        content_dir = _find_skill_content_dir(tmp_dir)
        
        # 复制到目标目录
        os.makedirs(dest_dir, exist_ok=True)
        for item in os.listdir(content_dir):
            src = os.path.join(content_dir, item)
            dst = os.path.join(dest_dir, item)
            if os.path.isdir(src):
                shutil.copytree(src, dst, dirs_exist_ok=True)
            else:
                shutil.copy2(src, dst)


def _find_skill_content_dir(extracted_dir: str) -> str:
    """在解压目录中找到实际的 skill 内容目录
    
    策略：
    1. 如果当前目录直接包含 SKILL.md，返回当前目录
    2. 如果只有一个有效子文件夹（忽略 __MACOSX），递归检查
    3. 否则返回当前目录
    """
    # 当前目录有 SKILL.md，就是 skill 内容目录
    if os.path.isfile(os.path.join(extracted_dir, "SKILL.md")):
        return extracted_dir
    
    # 过滤掉 __MACOSX 等无关目录
    subdirs = [
        d for d in os.listdir(extracted_dir)
        if os.path.isdir(os.path.join(extracted_dir, d))
        and not d.startswith("__") and not d.startswith(".")
    ]
    
    if len(subdirs) == 1:
        # 只有一个有效子文件夹，递归向下查找
        return _find_skill_content_dir(os.path.join(extracted_dir, subdirs[0]))
    
    # 多个子文件夹或没有子文件夹，返回当前目录
    return extracted_dir
