"""API 客户端 - 对接现有后端"""
import requests
from typing import Any, Dict, List, Optional
from urllib.parse import urljoin, urlparse


# 默认 token，无需登录
DEFAULT_TOKEN = "f495c3c7d17b38bc4f1bb77e1e78cde2f1af96b1dbd9e7c5fb806c227bd00a38"


class SkillAPIClient:
    """Skillhub API 客户端"""
    
    def __init__(self, base_url: Optional[str] = None, timeout: int = 30):
        self.base_url = (base_url or "https://skills.kingdee.com/api").rstrip("/")
        self.timeout = timeout
        self.session = requests.Session()
        
        # 设置默认请求头，包含默认 token
        self.session.headers.update({
            "Accept": "application/json",
            "Content-Type": "application/json",
            "User-Agent": "skillhub-cli/0.1.0",
            "token": DEFAULT_TOKEN
        })
    
    def _request(
        self,
        method: str,
        endpoint: str,
        **kwargs
    ) -> Dict[str, Any]:
        """发送 HTTP 请求"""
        url = self.base_url + "/" + endpoint.lstrip("/")
        
        try:
            response = self.session.request(
                method=method,
                url=url,
                timeout=self.timeout,
                **kwargs
            )
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            raise APIError(f"API request failed: {e}")
    
    def get(self, endpoint: str, params: Optional[Dict] = None) -> Dict[str, Any]:
        """GET 请求"""
        return self._request("GET", endpoint, params=params)
    
    def post(self, endpoint: str, data: Optional[Dict] = None) -> Dict[str, Any]:
        """POST 请求"""
        return self._request("POST", endpoint, json=data)
    
    def download(
        self,
        endpoint: str,
        params: Optional[Dict] = None
    ) -> bytes:
        """下载二进制数据"""
        url = self.base_url + "/" + endpoint.lstrip("/")
        
        try:
            response = self.session.get(
                url,
                params=params,
                timeout=self.timeout
            )
            response.raise_for_status()
            return response.content
        except requests.exceptions.RequestException as e:
            raise APIError(f"Download failed: {e}")
    
    # ============ Skill 相关接口 ============
    
    def search_skills(
        self,
        keyword: Optional[str] = None,
        page: int = 1,
        page_size: int = 20
    ) -> Dict[str, Any]:
        """搜索 Skill - GET /skills/list"""
        params = {
            "page": page,
            "pageSize": page_size
        }
        
        if keyword:
            params["keyword"] = keyword
        
        return self.get("/skills/list", params=params)
    
    def download_skill(
        self,
        skill_id: int,
        version: str
    ) -> bytes:
        """下载 Skill ZIP - GET /skills/download"""
        params = {
            "id": skill_id,
            "version": version,
            "token": DEFAULT_TOKEN
        }
        
        return self.download("/skills/download", params=params)
    
    def get_qoder_download_url(
        self,
        skill_id: int,
        user_id: int,
        token: str
    ) -> Dict[str, Any]:
        """获取 Qoder 下载链接 - POST /skills/qoderDownloadUrl"""
        data = {
            "skillId": skill_id,
            "userId": user_id,
            "token": token
        }
        
        return self.post("/skills/qoderDownloadUrl", data=data)

    def get_command_reference(self) -> str:
        """获取服务器上的 COMMAND_REFERENCE.md - GET /install/COMMAND_REFERENCE.md

        注意：该文档与 install.sh / install.ps1 / *.whl 一同存放于服务器的 /install/ 目录下，不带 /api 前缀。
        """
        parsed = urlparse(self.base_url)
        root = f"{parsed.scheme}://{parsed.netloc}"
        url = f"{root}/install/COMMAND_REFERENCE.md"

        try:
            response = self.session.get(url, timeout=self.timeout)
            response.raise_for_status()
            # 明确使用 utf-8 避免中文乱码
            if not response.encoding or response.encoding.lower() == "iso-8859-1":
                response.encoding = "utf-8"
            return response.text
        except requests.exceptions.RequestException as e:
            raise APIError(f"Fetch command reference failed: {e}")


class APIError(Exception):
    """API 错误"""
    pass
