"""本地存储管理"""
import json
import shutil
from pathlib import Path
from typing import Any, Dict, List, Optional

from skillhub.core.config import get_default_storage_path


class SkillStorage:
    """Skill 本地存储"""
    
    def __init__(self, base_path: Optional[str] = None):
        if base_path:
            self.base_path = Path(base_path)
        else:
            self.base_path = Path(get_default_storage_path())
        
        self.base_path.mkdir(parents=True, exist_ok=True)
    
    def get_skill_path(self, name: str) -> Path:
        """获取 Skill 目录"""
        return self.base_path / name
    
    def is_installed(self, name: str) -> bool:
        """检查是否已安装"""
        skill_dir = self.get_skill_path(name)
        return skill_dir.exists() and (skill_dir / "SKILL.md").exists()
    
    def list_installed_skills(self) -> List[Dict[str, Any]]:
        """列出已安装的 Skill"""
        skills = []
        
        for item in self.base_path.iterdir():
            if item.is_dir() and not item.name.startswith("."):
                info = self.get_skill_info(item.name)
                if info:
                    skills.append(info)
        
        return skills
    
    def get_skill_info(self, name: str) -> Optional[Dict[str, Any]]:
        """获取 Skill 信息"""
        info_file = self.get_skill_path(name) / ".skillhub" / "info.json"
        
        if info_file.exists():
            try:
                with open(info_file, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                pass
        
        # 尝试从 metadata.json 读取
        metadata_file = self.get_skill_path(name) / "metadata.json"
        if metadata_file.exists():
            try:
                with open(metadata_file, "r", encoding="utf-8") as f:
                    return json.load(f)
            except Exception:
                pass
        
        return None
    
    def save_skill_info(self, skill_id: int, info: Dict[str, Any]) -> None:
        """保存 Skill 信息"""
        name = info.get("name", str(skill_id))
        skill_dir = self.get_skill_path(name)
        skillhub_dir = skill_dir / ".skillhub"
        skillhub_dir.mkdir(exist_ok=True)
        
        info_file = skillhub_dir / "info.json"
        with open(info_file, "w", encoding="utf-8") as f:
            json.dump(info, f, indent=2, ensure_ascii=False)
    
    def validate_skill(self, name: str) -> bool:
        """验证 Skill"""
        skill_dir = self.get_skill_path(name)
        return (skill_dir / "SKILL.md").exists()
    
    def remove_skill(self, name: str) -> None:
        """删除 Skill"""
        skill_dir = self.get_skill_path(name)
        if skill_dir.exists():
            shutil.rmtree(skill_dir)
