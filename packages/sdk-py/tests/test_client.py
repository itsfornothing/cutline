"""Unit test suite for Cutline Python SDK."""

import json
import unittest
from unittest.mock import MagicMock, patch

from cutline import Cutline, CutlineError


class TestCutlineSdk(unittest.TestCase):
    def setUp(self):
        self.client = Cutline(api_key="cl_test_py_key", base_url="https://api.cutline.dev")

    def test_client_init(self):
        self.assertEqual(self.client.api_key, "cl_test_py_key")
        self.assertEqual(self.client.base_url, "https://api.cutline.dev")

    @patch("urllib.request.urlopen")
    def test_renders_create_with_idempotency(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.read.return_value = json.dumps({
            "id": "01927c34-8c88-7e23-81a1-8d0739c9f28c",
            "status": "queued",
            "lane": "priority",
            "progress": 0.0,
            "billed_seconds": 10.0,
            "output_urls": {},
            "stages": [],
        }).encode("utf-8")
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        timeline = {
            "schema_version": "1.0",
            "outputs": [{"id": "out", "aspect": "16:9", "resolution": {"width": 1920, "height": 1080}, "fps": 30, "format": "mp4"}],
            "tracks": [],
        }

        job = self.client.renders.create(timeline, idempotency_key="py-idem-123")
        self.assertEqual(job.id, "01927c34-8c88-7e23-81a1-8d0739c9f28c")
        self.assertEqual(job.status, "queued")
        self.assertEqual(job.lane, "priority")

        # Verify request sent
        req = mock_urlopen.call_args[0][0]
        self.assertEqual(req.get_header("Authorization"), "Bearer cl_test_py_key")
        self.assertEqual(req.get_header("Idempotency-key"), "py-idem-123")

    @patch("urllib.request.urlopen")
    def test_renders_validate_dry_run(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.read.return_value = json.dumps({
            "valid": True,
            "estimated_duration_seconds": 15.0,
            "estimated_total_usd": 0.02,
            "quote": {"billed_seconds": 15.0},
            "warnings": [],
        }).encode("utf-8")
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        res = self.client.renders.validate({"schema_version": "1.0", "outputs": [], "tracks": []})
        self.assertTrue(res.valid)
        self.assertEqual(res.estimated_total_usd, 0.02)

    @patch("urllib.request.urlopen")
    def test_batches_create(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.read.return_value = json.dumps({
            "id": "batch-123",
            "tenant_id": "tenant_py",
            "total_jobs": 2,
            "render_ids": ["r1", "r2"],
            "created_at": "2026-10-14T15:00:00Z",
        }).encode("utf-8")
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        batch = self.client.batches.create(
            template={"schema_version": "1.0"},
            items=[{"title": "Item 1"}, {"title": "Item 2"}],
        )
        self.assertEqual(batch.id, "batch-123")
        self.assertEqual(batch.total_jobs, 2)
        self.assertEqual(len(batch.render_ids), 2)


if __name__ == "__main__":
    unittest.main()
