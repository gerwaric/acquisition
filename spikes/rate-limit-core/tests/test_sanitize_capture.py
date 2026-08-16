import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "tools" / "sanitize_capture.py"
SPEC = importlib.util.spec_from_file_location("sanitize_capture", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
sanitize_capture = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sanitize_capture)


class SanitizerTest(unittest.TestCase):
    def raw_record(self):
        return {
            "v": 1,
            "kind": "reply",
            "policy": "stash-request-limit",
            "endpoint": "Get Stash",
            "request_id": 42,
            "url": "https://api.pathofexile.com/stash/private-id",
            "scheduled": "2026-07-18T12:00:01.000+00:00",
            "sent": "2026-07-18T12:00:01.250+00:00",
            "received": "2026-07-18T12:00:02.000+00:00",
            "status": 200,
            "headers": {
                "x-rate-limit-policy": "stash-request-limit",
                "x-rate-limit-rules": "Account",
                "x-rate-limit-account": "15:10:60, 30:300:300",
                "x-rate-limit-account-state": "1:10:0, 1:300:0",
                "retry-after": "61",
                "date": "Sat, 18 Jul 2026 12:00:02 GMT",
                "authorization": "Bearer secret",
                "set-cookie": "POESESSID=secret",
                "content-type": "application/json",
            },
            "payload": {"accountName": "private"},
        }

    def sanitize(self, records):
        import json

        lines = [json.dumps(record).encode() + b"\n" for record in records]
        return sanitize_capture.sanitize(
            lines,
            capture_date="2026-07-18",
            capture_schema_version=1,
            session_shape="full refresh, ~121 stashes, OAuth, PC realm",
            claim_lanes=["N23-N26"],
        )

    def test_allowlist_removes_identifiers_urls_tokens_and_payloads(self):
        fixture = self.sanitize([self.raw_record()])
        record = fixture["records"][0]
        self.assertEqual(
            set(record),
            {
                "kind",
                "policy",
                "endpoint",
                "status",
                "scheduled_ms",
                "sent_ms",
                "received_ms",
                "headers",
                "date_ms",
            },
        )
        self.assertEqual(record["endpoint"], "get-stash")
        self.assertEqual(record["scheduled_ms"], 0)
        self.assertEqual(record["sent_ms"], 250)
        self.assertEqual(record["received_ms"], 1_000)
        self.assertEqual(record["date_ms"], 1_000)
        self.assertNotIn("authorization", record["headers"])
        self.assertNotIn("set-cookie", record["headers"])
        serialized = str(fixture)
        self.assertNotIn("private-id", serialized)
        self.assertNotIn("accountName", serialized)
        self.assertNotIn("secret", serialized)

    def test_first_head_received_time_is_t0_and_legacy_url_becomes_label(self):
        head = self.raw_record()
        head.update(
            {
                "kind": "head",
                "endpoint": (
                    "https://www.pathofexile.com/character-window/get-stash-items"
                ),
                "received": "2026-07-18T12:00:00.000+00:00",
                "status": 200,
            }
        )
        head.pop("scheduled")
        head.pop("sent")
        fixture = self.sanitize([head])
        self.assertEqual(fixture["records"][0]["received_ms"], 0)
        self.assertEqual(
            fixture["records"][0]["endpoint"], "get-legacy-stash-index"
        )

    def test_unknown_endpoint_and_ambiguous_time_refuse_instead_of_leaking(self):
        raw = self.raw_record()
        raw["endpoint"] = "https://example.invalid/private"
        with self.assertRaises(sanitize_capture.SanitizationError):
            self.sanitize([raw])
        raw = self.raw_record()
        raw["scheduled"] = "2026-07-18T12:00:01.000"
        with self.assertRaises(sanitize_capture.SanitizationError):
            self.sanitize([raw])

    def test_record_and_line_caps_pin_n_plus_one(self):
        import json

        raw = self.raw_record()
        encoded = json.dumps(raw).encode()
        fixture = sanitize_capture.sanitize(
            [encoded] * sanitize_capture.MAX_RECORDS,
            capture_date="2026-07-18",
            capture_schema_version=1,
            session_shape="shape",
            claim_lanes=["lane"],
        )
        self.assertEqual(len(fixture["records"]), sanitize_capture.MAX_RECORDS)
        with self.assertRaises(sanitize_capture.SanitizationError):
            sanitize_capture.sanitize(
                [b"{}\n"] * (sanitize_capture.MAX_RECORDS + 1),
                capture_date="2026-07-18",
                capture_schema_version=1,
                session_shape="shape",
                claim_lanes=["lane"],
            )
        exact_line = encoded + b" " * (sanitize_capture.MAX_LINE_BYTES - len(encoded))
        self.assertEqual(len(exact_line), sanitize_capture.MAX_LINE_BYTES)
        sanitize_capture.sanitize(
            [exact_line],
            capture_date="2026-07-18",
            capture_schema_version=1,
            session_shape="shape",
            claim_lanes=["lane"],
        )
        with self.assertRaises(sanitize_capture.SanitizationError):
            sanitize_capture.sanitize(
                [exact_line + b" "],
                capture_date="2026-07-18",
                capture_schema_version=1,
                session_shape="shape",
                claim_lanes=["lane"],
            )
        # The exact header bound remains accepted.
        raw["headers"]["x-rate-limit-policy"] = "p" * 256
        self.sanitize([raw])


if __name__ == "__main__":
    unittest.main()
