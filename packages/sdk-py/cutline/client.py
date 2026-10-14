"""Official Python Client for Cutline Video Rendering API."""

import json
import os
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional

from .errors import CutlineError
from .types import (
    BatchRecord,
    BrandKit,
    RenderJob,
    ValidationResponse,
)


class Cutline:
    """Client for communicating with the Cutline API."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: str = "https://api.cutline.dev",
        timeout: float = 30.0,
    ):
        self.api_key = api_key or os.environ.get("CUTLINE_API_KEY", "")
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

        self.renders = RendersResource(self)
        self.batches = BatchesResource(self)
        self.brand_kits = BrandKitsResource(self)
        self.templates = TemplatesResource(self)
        self.usage = UsageResource(self)

    def request(
        self,
        method: str,
        path: str,
        body: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
    ) -> Any:
        url = f"{self.base_url}{path}"
        req_headers = {
            "Content-Type": "application/json",
            "Accept": "application/json, application/problem+json",
        }
        if self.api_key:
            req_headers["Authorization"] = f"Bearer {self.api_key}"
        if headers:
            req_headers.update(headers)

        data = json.dumps(body).encode("utf-8") if body is not None else None
        req = urllib.request.Request(url, data=data, headers=req_headers, method=method)

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as response:
                content = response.read().decode("utf-8")
                return json.loads(content) if content else {}
        except urllib.error.HTTPError as err:
            err_content = err.read().decode("utf-8")
            try:
                err_data = json.loads(err_content)
                message = (
                    err_data.get("detail")
                    or err_data.get("title")
                    or err_data.get("message")
                    or str(err)
                )
                code = err_data.get("code")
            except Exception:
                err_data = err_content
                message = str(err)
                code = None
            raise CutlineError(
                message=message,
                status_code=err.code,
                code=code,
                details=err_data,
            ) from None
        except Exception as err:
            raise CutlineError(message=f"Request failed: {err}") from err


class RendersResource:
    def __init__(self, client: Cutline):
        self.client = client

    def create(
        self,
        timeline: Dict[str, Any],
        idempotency_key: Optional[str] = None,
    ) -> RenderJob:
        headers = {}
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        data = self.client.request("POST", "/v1/renders", body=timeline, headers=headers)
        return RenderJob(
            id=data["id"],
            tenant_id=data.get("tenant_id", ""),
            status=data["status"],
            lane=data.get("lane", "standard"),
            progress=data.get("progress", 0.0),
            billed_seconds=data.get("billed_seconds", 0.0),
            output_urls=data.get("output_urls", {}),
            stages=data.get("stages", []),
            created_at=data.get("created_at"),
        )

    def get(self, render_id: str) -> RenderJob:
        data = self.client.request("GET", f"/v1/renders/{render_id}")
        return RenderJob(
            id=data["id"],
            tenant_id=data.get("tenant_id", ""),
            status=data["status"],
            lane=data.get("lane", "standard"),
            progress=data.get("progress", 0.0),
            billed_seconds=data.get("billed_seconds", 0.0),
            output_urls=data.get("output_urls", {}),
            stages=data.get("stages", []),
        )

    def cancel(self, render_id: str) -> RenderJob:
        data = self.client.request("DELETE", f"/v1/renders/{render_id}")
        return RenderJob(
            id=data["id"],
            tenant_id=data.get("tenant_id", ""),
            status=data["status"],
            lane=data.get("lane", "standard"),
            progress=data.get("progress", 0.0),
            billed_seconds=data.get("billed_seconds", 0.0),
        )

    def validate(self, timeline: Dict[str, Any]) -> ValidationResponse:
        data = self.client.request("POST", "/v1/renders:validate", body=timeline)
        return ValidationResponse(
            valid=data.get("valid", True),
            estimated_duration_seconds=data.get("estimated_duration_seconds", 0.0),
            estimated_total_usd=data.get("estimated_total_usd", 0.0),
            quote=data.get("quote", {}),
            warnings=data.get("warnings", []),
        )


class BatchesResource:
    def __init__(self, client: Cutline):
        self.client = client

    def create(
        self,
        template: Dict[str, Any],
        items: List[Dict[str, Any]],
        lane: str = "standard",
        priority: int = 0,
    ) -> BatchRecord:
        body = {
            "template": template,
            "items": items,
            "lane": lane,
            "priority": priority,
        }
        data = self.client.request("POST", "/v1/batches", body=body)
        return BatchRecord(
            id=data["id"],
            tenant_id=data["tenant_id"],
            total_jobs=data["total_jobs"],
            render_ids=data["render_ids"],
            created_at=data["created_at"],
        )

    def get(self, batch_id: str) -> Dict[str, Any]:
        return self.client.request("GET", f"/v1/batches/{batch_id}")


class BrandKitsResource:
    def __init__(self, client: Cutline):
        self.client = client

    def create(
        self,
        brand_id: str,
        name: str,
        fonts: Optional[Dict[str, str]] = None,
        colors: Optional[Dict[str, str]] = None,
        logo_url: Optional[str] = None,
        intro_bumper_url: Optional[str] = None,
        outro_bumper_url: Optional[str] = None,
    ) -> BrandKit:
        body = {
            "brand_id": brand_id,
            "name": name,
            "fonts": fonts or {},
            "colors": colors or {},
            "logo_url": logo_url,
            "intro_bumper_url": intro_bumper_url,
            "outro_bumper_url": outro_bumper_url,
        }
        data = self.client.request("POST", "/v1/brand-kits", body=body)
        return BrandKit(
            brand_id=data["brand_id"],
            tenant_id=data["tenant_id"],
            name=data["name"],
            fonts=data.get("fonts", {}),
            colors=data.get("colors", {}),
            logo_url=data.get("logo_url"),
            intro_bumper_url=data.get("intro_bumper_url"),
            outro_bumper_url=data.get("outro_bumper_url"),
        )

    def get(self, brand_id: str) -> BrandKit:
        data = self.client.request("GET", f"/v1/brand-kits/{brand_id}")
        return BrandKit(
            brand_id=data["brand_id"],
            tenant_id=data["tenant_id"],
            name=data["name"],
            fonts=data.get("fonts", {}),
            colors=data.get("colors", {}),
        )


class TemplatesResource:
    def __init__(self, client: Cutline):
        self.client = client

    def create(self, template_id: str, name: str, timeline: Dict[str, Any]) -> Dict[str, Any]:
        body = {"id": template_id, "name": name, "timeline": timeline}
        return self.client.request("POST", "/v1/templates", body=body)

    def get(self, template_id: str) -> Dict[str, Any]:
        return self.client.request("GET", f"/v1/templates/{template_id}")

    def pin_golden(self, template_id: str, timestamps: List[float]) -> Dict[str, Any]:
        body = {"timestamps": timestamps}
        return self.client.request("POST", f"/v1/templates/{template_id}/golden", body=body)

    def run_ci(self, template_id: str) -> Dict[str, Any]:
        return self.client.request("POST", f"/v1/templates/{template_id}/ci-run")


class UsageResource:
    def __init__(self, client: Cutline):
        self.client = client

    def get(self) -> List[Dict[str, Any]]:
        return self.client.request("GET", "/v1/usage")
