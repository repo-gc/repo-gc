"""Markdown reporter — mirrors Rust's markdown.rs."""

from ..types import Report, Severity


def _score_level(score: int) -> str:
    if score >= 70:
        return "CRITICAL"
    elif score >= 40:
        return "MODERATE"
    elif score >= 20:
        return "ELEVATED"
    else:
        return "LOW"


def render_markdown(report: Report) -> str:
    lines: list[str] = []
    lines.append("# repo-gc: AI Context Efficiency Report")
    lines.append("")

    lines.append(
        f"**Files analyzed:** {report.files_analyzed}  |  "
        f"**Total lines:** {report.total_lines}  |  "
        f"**Est. LLM tokens:** ~{report.total_estimated_tokens // 1000}k"
    )
    lines.append("")

    if report.files_skipped > 0:
        lines.append(f"> **Warning:** {report.files_skipped} files skipped due to parse errors.")
        lines.append("")

    gs = report.global_score
    lines.append("## Global Scores (0-100, higher = worse)")
    lines.append("")
    lines.append("| Metric | Score | Level | Meaning |")
    lines.append("|--------|-------|-------|--------|")
    lines.append(
        f"| **AI Friction** | {gs.ai_friction_score}/100 | {_score_level(gs.ai_friction_score)} "
        f"| Overall AI-friction from all detected patterns |"
    )
    lines.append(
        f"| **Context Waste** | {gs.context_waste_score}/100 | {_score_level(gs.context_waste_score)} "
        f"| Token budget wasted by oversized files |"
    )
    lines.append(
        f"| **Structural Entropy** | {gs.structural_entropy_score}/100 | "
        f"{_score_level(gs.structural_entropy_score)} "
        f"| Noise from coupling, dead code, re-exports, duplication |"
    )
    lines.append("")

    lines.append(
        f"> **Estimated context waste:** ~{gs.estimated_waste_pct}% of agent context capacity "
        f"(~{gs.context_waste_ratio:.1f}x Claude sessions)"
    )
    lines.append("")

    lines.append("## How to Interpret These Scores")
    lines.append("")
    lines.append("- **0-19:** Healthy — your codebase is well-structured for AI tooling.")
    lines.append("- **20-39:** Moderate — some patterns will slow down AI-assisted edits.")
    lines.append("- **40-69:** Concerning — AI-assisted development will be noticeably degraded.")
    lines.append("- **70-100:** Critical — significant refactoring recommended.")
    lines.append("")
    lines.append("---")
    lines.append("")

    if not report.findings:
        lines.append("## Detected Patterns")
        lines.append("")
        lines.append("No issues found.")
        return "\n".join(lines) + "\n"

    lines.append("## Detected Patterns")
    lines.append("")

    sorted_findings = sorted(report.findings, key=lambda f: f.severity.weight, reverse=True)

    for i, f in enumerate(sorted_findings):
        tok = f" (~{f.estimated_tokens // 1000}k tokens)" if f.estimated_tokens else ""
        lines.append(f"### {i + 1}. [{f.severity.label}] {f.path}")
        lines.append("")
        lines.append(f"**Kind:** {f.kind.label}  ")
        lines.append(f"**Summary:** {f.summary}{tok}  ")
        lines.append("")
        if f.reasons:
            lines.append("**Reasons:**")
            for r in f.reasons:
                lines.append(f"- {r}")
            lines.append("")
        lines.append(f"**Next step:** {f.suggested_next_step}")
        lines.append("")
        lines.append("---")
        lines.append("")

    lines.append("## Priority Cleanup Targets (fix these first)")
    lines.append("")
    for f in sorted_findings:
        if f.severity in (Severity.Critical, Severity.High):
            lines.append(f"- {f.suggested_next_step}")

    return "\n".join(lines) + "\n"
