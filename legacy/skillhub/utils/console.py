"""控制台输出工具"""
from rich.console import Console
from rich.table import Table
from rich import box

console = Console()

# 描述最大字符数
DESC_MAX_LEN = 100


def truncate_desc(text: str, max_len: int = DESC_MAX_LEN) -> str:
    """截断描述文本，超过 max_len 字符后用 ... 省略"""
    if not text:
        return "-"
    text = text.replace("\n", " ").replace("\r", " ").strip()
    if len(text) > max_len:
        return text[:max_len] + "..."
    return text


def create_skill_table(*extra_columns: tuple) -> Table:
    """创建统一风格的 Skill 表格

    默认包含：名称、标识、描述、版本
    extra_columns: 额外列，格式为 (name, style, kwargs)

    列宽策略：
        - 所有列按 ratio 随终端/对话框宽度同步缩放
        - 空间不足时所有列自动折行，而不是用 ... 截断
        - min_width 仅作兜底，避免极窄终端下 0 宽
    """
    table = Table(
        show_header=True,
        header_style="bold cyan",
        box=box.HORIZONTALS,
        show_edge=False,
        pad_edge=False,
        show_lines=True,
        expand=True,
    )
    table.add_column(
        "名称",
        style="green",
        no_wrap=False,
        overflow="fold",
        min_width=6,
        ratio=2,
    )
    table.add_column(
        "标识",
        style="cyan",
        no_wrap=False,
        overflow="fold",
        min_width=6,
        ratio=2,
    )
    table.add_column(
        "描述",
        style="white",
        no_wrap=False,
        overflow="fold",
        min_width=10,
        ratio=5,
    )
    table.add_column(
        "版本",
        style="yellow",
        justify="right",
        no_wrap=False,
        overflow="fold",
        min_width=5,
        ratio=1,
    )

    for col in extra_columns:
        name, style, kwargs = col
        table.add_column(name, style=style, **kwargs)

    return table


def success(message: str) -> None:
    """成功消息"""
    console.print(f"[green]✓[/] {message}")


def error(message: str) -> None:
    """错误消息"""
    console.print(f"[red]✗[/] {message}")


def info(message: str) -> None:
    """信息消息"""
    console.print(f"[blue]ℹ[/] {message}")


def warning(message: str) -> None:
    """警告消息"""
    console.print(f"[yellow]⚠[/] {message}")
