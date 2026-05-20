"""JSON reporter — primary integration path for the TypeScript runner."""

from ..types import Report, report_to_json


def render_json(report: Report) -> str:
    return report_to_json(report)
