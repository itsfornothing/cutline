"""Cutline SDK Exceptions."""

from typing import Any, Optional


class CutlineError(Exception):
    """Base exception for all Cutline API failures."""

    def __init__(
        self,
        message: str,
        status_code: Optional[int] = None,
        code: Optional[str] = None,
        details: Optional[Any] = None,
    ):
        super().__init__(message)
        self.status_code = status_code
        self.code = code
        self.details = details
