"""配置管理"""
import os
import yaml
from pathlib import Path
from typing import Any, Dict, Optional


def detect_ide() -> str:
    """检测当前运行的 IDE 环境
    
    返回:
        "qoder"     - Qoder IDE
        "qoderwork" - QoderWork IDE
        "kiro"      - Kiro IDE
        "qoderwork" - 默认回退
    """
    # 1. 检查专用环境变量
    if os.environ.get("KIRO_IDE"):
        return "kiro"
    if os.environ.get("QODER_IDE"):
        return "qoder"
    if os.environ.get("QODERWORK_IDE"):
        return "qoderwork"
    
    # 2. 通过 VSCODE_GIT_ASKPASS_NODE 路径中的 IDE 可执行文件名判断
    askpass_node = os.environ.get("VSCODE_GIT_ASKPASS_NODE", "")
    askpass_lower = askpass_node.lower()
    if "kiro" in askpass_lower:
        return "kiro"
    if "qoder" in askpass_lower and "qoderwork" not in askpass_lower:
        return "qoder"
    if "qoderwork" in askpass_lower:
        return "qoderwork"
    
    # 3. 通过 GIT_ASKPASS 路径判断
    git_askpass = os.environ.get("GIT_ASKPASS", "")
    git_askpass_lower = git_askpass.lower()
    if "kiro" in git_askpass_lower:
        return "kiro"
    if "qoder" in git_askpass_lower and "qoderwork" not in git_askpass_lower:
        return "qoder"
    if "qoderwork" in git_askpass_lower:
        return "qoderwork"
    
    # 4. 默认 qoderwork
    return "qoderwork"


def get_default_storage_path() -> str:
    """根据 IDE 环境返回默认 skills 存储路径"""
    ide = detect_ide()
    return str(Path.home() / f".{ide}" / "skills")


# 安装时需要同时写入的所有 IDE / 工具
ALL_IDES = ("qoder", "qoderwork", "kiro", "workbuddy")


def get_ide_skills_path(ide: str) -> str:
    """获取指定 IDE / 工具的 skills 目录路径"""
    return str(Path.home() / f".{ide}" / "skills")


def get_ide_install_check_path(ide: str) -> Path:
    """返回用于判断该 IDE / 工具是否存在的路径

    - qoder / qoderwork / kiro：检查 ~/.{ide} 根目录是否存在
    - workbuddy：检查 ~/.workbuddy/skills 目录是否存在，
      若不存在则跳过安装（不会自动创建）
    """
    if ide == "workbuddy":
        return Path.home() / ".workbuddy" / "skills"
    return Path.home() / f".{ide}"


def get_all_storage_paths() -> list:
    """返回需要同时安装的所有 IDE / 工具的 skills 目录

    用于 install 命令：一次性把技能写入 qoder / qoderwork / kiro / workbuddy 四个位置。
    """
    return [get_ide_skills_path(ide) for ide in ALL_IDES]


class Config:
    """Skillhub 配置管理"""
    
    DEFAULT_CONFIG = {
        "api": {
            "endpoint": "https://skills.kingdee.com/api",
            "timeout": 30
        },
        "storage": {
            "path": None  # 动态计算
        }
    }
    
    def __init__(self, config_path: Optional[str] = None):
        self._config: Dict[str, Any] = {}
        
        if config_path:
            self.config_path = Path(config_path)
        else:
            self.config_path = Path.home() / ".config" / "skillhub" / "config.yaml"
        
        self.config_path.parent.mkdir(parents=True, exist_ok=True)
        self._load()
    
    def _load(self) -> None:
        """加载配置"""
        self._config = self._deep_copy(self.DEFAULT_CONFIG)
        self._config["storage"]["path"] = get_default_storage_path()
        
        if self.config_path.exists():
            try:
                with open(self.config_path, "r", encoding="utf-8") as f:
                    user_config = yaml.safe_load(f) or {}
                    self._merge_config(self._config, user_config)
            except Exception:
                pass
    
    def save(self) -> None:
        """保存配置"""
        with open(self.config_path, "w", encoding="utf-8") as f:
            yaml.dump(self._config, f, default_flow_style=False)
    
    def get(self, key: str, default: Any = None) -> Any:
        """获取配置项"""
        keys = key.split(".")
        value = self._config
        
        for k in keys:
            if isinstance(value, dict) and k in value:
                value = value[k]
            else:
                return default
        
        return value
    
    def set(self, key: str, value: Any) -> None:
        """设置配置项"""
        keys = key.split(".")
        config = self._config
        
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]
        
        config[keys[-1]] = value
    
    @staticmethod
    def _deep_copy(config: Dict) -> Dict:
        import copy
        return copy.deepcopy(config)
    
    @staticmethod
    def _merge_config(base: Dict, override: Dict) -> None:
        for key, value in override.items():
            if key in base and isinstance(base[key], dict) and isinstance(value, dict):
                Config._merge_config(base[key], value)
            else:
                base[key] = value
