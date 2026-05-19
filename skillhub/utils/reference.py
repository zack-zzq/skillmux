"""命令参考文档 (COMMAND_REFERENCE.md) 同步工具

每次执行 search / list / update 命令完成后，从服务器拉取最新的
COMMAND_REFERENCE.md 并保存到用户目录，供 AI / 用户复用。
"""
from pathlib import Path
from typing import Optional

from skillhub.core.api_client import SkillAPIClient


# 用户目录下的固定保存路径
REFERENCE_PATH = Path.home() / ".config" / "skillhub" / "COMMAND_REFERENCE.md"


def refresh_command_reference(api: SkillAPIClient) -> Optional[Path]:
    """从服务器拉取 COMMAND_REFERENCE.md 并保存到本地。

    成功返回保存路径；任何异常（网络/解析/写入失败）均静默返回 None，
    不影响主命令的执行结果。
    """
    try:
        content = api.get_command_reference()
        if not content:
            return None
        REFERENCE_PATH.parent.mkdir(parents=True, exist_ok=True)
        REFERENCE_PATH.write_text(content, encoding="utf-8")
        return REFERENCE_PATH
    except Exception:
        return None
