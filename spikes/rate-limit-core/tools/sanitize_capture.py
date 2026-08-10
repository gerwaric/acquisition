#!/usr/bin/env python3
"""Sanitize NetworkCapture JSONL into the scenarios.md §4 fixture shape."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
from pathlib import Path
from typing import Any

SANITIZER_VERSION = 1
MAX_INPUT_BYTES = 40 * 1024 * 1024
MAX_RECORDS = 10_000
MAX_LINE_BYTES = 64 * 1024
MAX_HEADER_VALUE_BYTES = 1_024
MAX_METADATA_BYTES = 512

ENDPOINTS = {
    "List Stashes": "list-stashes",
    "Get Stash": "get-stash",
    "List Characters": "list-characters",
    "Get Character": "get-character",
    "https://www.pathofexile.com/character-window/get-stash-items": (
        "get-legacy-stash-index"
    ),
}


class SanitizationError(ValueError):
    """The raw capture cannot safely satisfy the allowlist contract."""


def _bounded_text(value: Any, field: str, limit: int = MAX_METADATA_BYTES) -> str:
    if not isinstance(value, str) or not value or len(value.encode("utf-8")) > limit:
        raise SanitizationError(f"{field} must be non-empty UTF-8 text <= {limit} bytes")
    return value


def _parse_iso(value: Any, field: str) -> datetime:
    raw = _bounded_text(value, field)
    try:
        parsed = datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError as error:
        raise SanitizationError(f"{field} is not ISO-8601") from error
    if parsed.tzinfo is None:
        raise SanitizationError(f"{field} has no UTC offset; rebasing would be ambiguous")
    return parsed.astimezone(timezone.utc)


def _parse_http_date(value: str) -> datetime:
    try:
        parsed = parsedate_to_datetime(value)
    except (TypeError, ValueError) as error:
        raise SanitizationError("Date header is not an RFC HTTP date") from error
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def _relative_ms(value: datetime, origin: datetime) -> int:
    return round((value - origin).total_seconds() * 1_000)


def _first_record_origin(record: dict[str, Any]) -> datetime:
    # "t0 = first record" is interpreted as the first client-side instant in
    # that record: scheduled, then sent, then received. A HEAD has received
    # only. Server Date is deliberately not the origin because its wire
    # precision is one second and could move t0 backward by rounding.
    for field in ("scheduled", "sent", "received"):
        if field in record:
            return _parse_iso(record[field], field)
    raise SanitizationError("first record has no scheduled/sent/received timestamp")


def _sanitize_headers(
    raw: Any, origin: datetime
) -> tuple[dict[str, str], int | None]:
    if not isinstance(raw, dict):
        raise SanitizationError("headers must be an object")
    retained: dict[str, str] = {}
    date_ms: int | None = None
    for name, value in raw.items():
        if not isinstance(name, str):
            raise SanitizationError("header name is not text")
        normalized = name.lower()
        if normalized == "date":
            text = _bounded_text(value, "Date header", MAX_HEADER_VALUE_BYTES)
            date_ms = _relative_ms(_parse_http_date(text), origin)
        elif normalized.startswith("x-rate-limit-") or normalized == "retry-after":
            retained[normalized] = _bounded_text(
                value, normalized, MAX_HEADER_VALUE_BYTES
            )
        # Everything else is dropped by allowlist, including auth/cookies.
    return retained, date_ms


def _sanitize_record(
    raw: Any, origin: datetime, expected_capture_version: int
) -> dict[str, Any]:
    if not isinstance(raw, dict):
        raise SanitizationError("capture line is not an object")
    if raw.get("v") != expected_capture_version:
        raise SanitizationError("capture record schema version mismatch")
    kind = raw.get("kind")
    if kind not in ("head", "reply"):
        raise SanitizationError("record kind is not head/reply")
    endpoint = ENDPOINTS.get(raw.get("endpoint"))
    if endpoint is None:
        raise SanitizationError("endpoint is not in the D5 five-endpoint vocabulary")

    record: dict[str, Any] = {"kind": kind, "endpoint": endpoint}
    if "policy" in raw:
        record["policy"] = _bounded_text(raw["policy"], "policy", 256)
    status = raw.get("status")
    if not isinstance(status, int) or isinstance(status, bool) or not 0 <= status <= 999:
        raise SanitizationError("status is not an integer in [0, 999]")
    record["status"] = status
    if "error" in raw:
        if not isinstance(raw["error"], int) or isinstance(raw["error"], bool):
            raise SanitizationError("error is not an integer")
        record["error"] = raw["error"]
    if "error_string" in raw:
        record["error_string"] = _bounded_text(raw["error_string"], "error_string")

    for field in ("scheduled", "sent", "received"):
        if field in raw:
            record[f"{field}_ms"] = _relative_ms(_parse_iso(raw[field], field), origin)
    if "received_ms" not in record:
        raise SanitizationError("record has no received timestamp")

    headers, date_ms = _sanitize_headers(raw.get("headers", {}), origin)
    record["headers"] = headers
    if date_ms is not None:
        record["date_ms"] = date_ms
    return record


def sanitize(
    lines: list[bytes],
    *,
    capture_date: str,
    capture_schema_version: int,
    session_shape: str,
    claim_lanes: list[str],
) -> dict[str, Any]:
    if not lines:
        raise SanitizationError("capture is empty")
    if len(lines) > MAX_RECORDS:
        raise SanitizationError(f"capture exceeds {MAX_RECORDS} records")
    records: list[dict[str, Any]] = []
    decoded: list[dict[str, Any]] = []
    for line in lines:
        if len(line) > MAX_LINE_BYTES:
            raise SanitizationError(f"capture line exceeds {MAX_LINE_BYTES} bytes")
        try:
            raw = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise SanitizationError("capture contains invalid JSONL") from error
        if not isinstance(raw, dict):
            raise SanitizationError("capture line is not an object")
        decoded.append(raw)

    origin = _first_record_origin(decoded[0])
    for raw in decoded:
        records.append(_sanitize_record(raw, origin, capture_schema_version))

    return {
        "provenance": {
            "capture_date": _bounded_text(capture_date, "capture_date", 32),
            "capture_schema_version": capture_schema_version,
            "sanitizer_version": SANITIZER_VERSION,
            "session_shape": _bounded_text(session_shape, "session_shape"),
            "claim_lanes": [
                _bounded_text(lane, "claim lane", 64) for lane in claim_lanes
            ],
        },
        "records": records,
    }


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--capture-date", required=True)
    parser.add_argument("--capture-schema-version", required=True, type=int)
    parser.add_argument("--session-shape", required=True)
    parser.add_argument("--claim-lane", action="append", required=True)
    return parser.parse_args()


def main() -> None:
    arguments = _arguments()
    size = arguments.input.stat().st_size
    if size > MAX_INPUT_BYTES:
        raise SanitizationError(f"capture exceeds {MAX_INPUT_BYTES} bytes")
    with arguments.input.open("rb") as source:
        lines = source.readlines(MAX_INPUT_BYTES + 1)
    fixture = sanitize(
        lines,
        capture_date=arguments.capture_date,
        capture_schema_version=arguments.capture_schema_version,
        session_shape=arguments.session_shape,
        claim_lanes=arguments.claim_lane,
    )
    # The output is a new sanitized artifact. Refuse to overwrite any file so
    # a mistaken target cannot destroy the retained raw capture.
    with arguments.output.open("x", encoding="utf-8") as destination:
        json.dump(fixture, destination, indent=2, sort_keys=True)
        destination.write("\n")


if __name__ == "__main__":
    main()
