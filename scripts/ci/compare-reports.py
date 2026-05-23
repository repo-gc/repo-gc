#!/usr/bin/env python3
"""
Cross-language report comparison script for repo-gc.

Validates that all 3 language implementations (Rust, TypeScript, Python)
produce consistent output when analyzing the same fixture repository.

Usage:
    python compare-reports.py rust.json ts.json py.json [--verbose]

The script:
  1. Loads 3 JSON report files
  2. Strips IDs (non-deterministic)
  3. Normalizes kind to kebab-case and severity to UPPERCASE
  4. Groups findings by (kind, path) and compares counts across implementations
  5. Reports mismatches
  6. Exits with code 0 if all match, non-zero otherwise
"""

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path


_CANONICAL: dict | None = None


def _load_canonical() -> dict:
    """Load the canonical finding kinds JSON from the repo root."""
    global _CANONICAL
    if _CANONICAL is None:
        path = (
            Path(__file__).resolve().parent.parent.parent
            / "test-fixtures"
            / "finding-kinds.json"
        )
        with open(path) as f:
            _CANONICAL = json.load(f)
    return _CANONICAL


def _build_kind_normalize() -> dict[str, str]:
    """Build kind normalization map from canonical JSON.

    Maps both PascalCase (Rust serde/Python) and kebab-case (TypeScript)
    serializations to canonical kebab-case labels.
    """
    canonical = _load_canonical()
    mapping: dict[str, str] = {}
    for k in canonical["finding_kinds"]:
        mapping[k["id"]] = k["label"]
        mapping[k["label"]] = k["label"]
    return mapping


def _build_severity_normalize() -> dict[str, str]:
    """Build severity normalization map from canonical JSON.

    Maps both PascalCase (Rust serde/Python) and UPPERCASE (TypeScript)
    serializations to canonical UPPERCASE labels.
    """
    canonical = _load_canonical()
    mapping: dict[str, str] = {}
    for s in canonical["severities"]:
        mapping[s["id"]] = s["label"]
        mapping[s["label"]] = s["label"]
    return mapping


KIND_NORMALIZE = _build_kind_normalize()
SEVERITY_NORMALIZE = _build_severity_normalize()


def normalize_kind(raw: str) -> str:
    """Normalize a kind value to canonical kebab-case."""
    if raw in KIND_NORMALIZE:
        return KIND_NORMALIZE[raw]
    # Fallback: convert PascalCase to kebab-case
    result = []
    for ch in raw:
        if ch.isupper() and result:
            result.append("-")
            result.append(ch.lower())
        elif ch.isupper():
            result.append(ch.lower())
        else:
            result.append(ch)
    kebab = "".join(result)
    if kebab != raw:
        sys.stderr.write(f"  [warn] unregistered kind '{raw}' normalized to '{kebab}'\n")
    return kebab


def normalize_severity(raw: str) -> str:
    """Normalize a severity value to canonical UPPERCASE."""
    if raw in SEVERITY_NORMALIZE:
        return SEVERITY_NORMALIZE[raw]
    upper = raw.upper()
    sys.stderr.write(f"  [warn] unregistered severity '{raw}' normalized to '{upper}'\n")
    return upper


def load_report(path: str) -> list[dict]:
    """Load a JSON report file and return normalized findings."""
    with open(path) as f:
        report = json.load(f)

    findings = report.get("findings", [])
    normalized = []
    for f_item in findings:
        # Strip non-deterministic IDs
        normalized.append({
            "kind": normalize_kind(f_item["kind"]),
            "severity": normalize_severity(f_item["severity"]),
            "path": f_item.get("path", ""),
            "confidence": f_item.get("confidence", 0.0),
        })
    return normalized


def build_group_map(findings: list[dict]) -> dict[tuple[str, str], list[dict]]:
    """Group findings by (kind, path) and return the grouped map."""
    groups = defaultdict(list)
    for f_item in findings:
        key = (f_item["kind"], f_item["path"])
        groups[key].append(f_item)
    return dict(groups)


def compare_reports(
    rust_findings: list[dict],
    ts_findings: list[dict],
    py_findings: list[dict],
    verbose: bool = False,
) -> bool:
    """Compare grouped findings across 3 implementations.

    Returns True if all match, False otherwise.
    """
    rust_groups = build_group_map(rust_findings)
    ts_groups = build_group_map(ts_findings)
    py_groups = build_group_map(py_findings)

    all_keys = sorted(set(rust_groups) | set(ts_groups) | set(py_groups))

    mismatches = 0

    for key in all_keys:
        kind, path = key
        rust_count = len(rust_groups.get(key, []))
        ts_count = len(ts_groups.get(key, []))
        py_count = len(py_groups.get(key, []))

        if rust_count == ts_count == py_count:
            if verbose:
                print(f"  OK: {kind} @ {path} ({rust_count})")
            continue

        mismatches += 1
        print(
            f"  MISMATCH: {kind} @ {path}: "
            f"rust={rust_count}, ts={ts_count}, py={py_count}"
        )

    return mismatches == 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Cross-language report comparison for repo-gc"
    )
    parser.add_argument("files", nargs=3, help="JSON report files (rust, ts, py)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Show all keys")
    args = parser.parse_args()

    rust_file, ts_file, py_file = args.files

    print(f"Loading reports...")
    for path in [rust_file, ts_file, py_file]:
        if not Path(path).exists():
            print(f"Error: file not found: {path}", file=sys.stderr)
            return 1

    rust_findings = load_report(rust_file)
    ts_findings = load_report(ts_file)
    py_findings = load_report(py_file)

    print(f"  Rust ({rust_file}):       {len(rust_findings)} findings")
    print(f"  TypeScript ({ts_file}):   {len(ts_findings)} findings")
    print(f"  Python ({py_file}):       {len(py_findings)} findings")

    if not rust_findings and not ts_findings and not py_findings:
        print("\nAll reports are empty — nothing to compare.")
        return 0

    print(f"\nComparing findings by (kind, path)...")
    match = compare_reports(rust_findings, ts_findings, py_findings, args.verbose)

    if match:
        print("\nAll implementations agree. (0 mismatches)")
        return 0
    else:
        print("\nMismatches found between implementations.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
