"""Terminal reporter with ANSI color — mirrors Rust's terminal.rs."""

from ..types import Finding, FindingKind, Report, Severity

# ANSI escapes
BOLD = "\033[1m"
RED = "\033[31m"
YELLOW = "\033[33m"
GREEN = "\033[32m"
CYAN = "\033[36m"
MAGENTA = "\033[35m"
RESET = "\033[0m"
UNDERLINE = "\033[4m"


def _color(s: str, code: str) -> str:
    return f"{code}{s}{RESET}"


def _score_bar(score: int) -> str:
    filled = min(score // 10, 10)
    empty = 10 - filled
    return "█" * filled + "░" * empty


def _score_label(score: int) -> str:
    if score >= 70:
        return "HIGH"
    elif score >= 40:
        return "MEDIUM"
    else:
        return "LOW"


def _colorize(score: int) -> str:
    s = f"{score}/100"
    if score >= 70:
        return _color(s, RED + BOLD)
    elif score >= 40:
        return _color(s, YELLOW)
    else:
        return _color(s, GREEN)


def _sev_colored(s: Severity) -> str:
    if s == Severity.Critical:
        return _color(s.label, RED + BOLD)
    elif s == Severity.High:
        return _color(s.label, RED)
    elif s == Severity.Medium:
        return _color(s.label, YELLOW)
    else:
        return s.label


def render_terminal(report: Report, no_color: bool = False) -> str:
    use_color = not no_color
    c = lambda s, code: _color(s, code) if use_color else s

    lines: list[str] = []
    lines.append("")
    lines.append(c("═══ repo-gc — AI Context Efficiency ═══", BOLD))
    lines.append(
        f"Files: {report.files_analyzed}  Lines: {report.total_lines}  "
        f"Est. LLM tokens: ~{report.total_estimated_tokens // 1000}k"
    )
    if report.files_skipped > 0:
        lines.append(f"  ({report.files_skipped} files skipped — parse errors)\n")
    else:
        lines.append("")

    gs = report.global_score
    lines.append(c("Global Scores (0-100, higher = worse)", BOLD + UNDERLINE))
    lines.append(
        f"  AI Friction:        {_score_bar(gs.ai_friction_score)}  "
        f"{c(_colorize(gs.ai_friction_score), '') if use_color else _colorize(gs.ai_friction_score)} "
        f"({_score_label(gs.ai_friction_score)})"
    )
    lines.append(
        f"  Context Waste:      {_score_bar(gs.context_waste_score)}  "
        f"{c(_colorize(gs.context_waste_score), '') if use_color else _colorize(gs.context_waste_score)} "
        f"({_score_label(gs.context_waste_score)})"
    )
    lines.append(
        f"  Structural Entropy: {_score_bar(gs.structural_entropy_score)}  "
        f"{c(_colorize(gs.structural_entropy_score), '') if use_color else _colorize(gs.structural_entropy_score)} "
        f"({_score_label(gs.structural_entropy_score)})"
    )
    lines.append("")

    lines.append(
        f"{c('▸', YELLOW + BOLD) if use_color else '▸'}  "
        f"Your repo wastes ~{gs.estimated_waste_pct}% of agent context capacity "
        f"(~{gs.context_waste_ratio:.1f}x Claude sessions)"
    )
    lines.append("")

    if not report.findings:
        lines.append(c("No issues found — your codebase is AI-friendly.", GREEN + BOLD))
        return "\n".join(lines) + "\n"

    sorted_findings = sorted(report.findings, key=lambda f: f.severity.weight, reverse=True)

    lines.append(c(f"Detected Patterns ({len(sorted_findings)} total)", BOLD + UNDERLINE))
    for i, f in enumerate(sorted_findings):
        tok = f" (~{f.estimated_tokens // 1000}k tokens)" if f.estimated_tokens else ""
        sev_display = c(_sev_colored(f.severity), "") if use_color else f.severity.label
        lines.append(
            f"\n  {i + 1}. [{sev_display}] "
            f"{c(f.path, CYAN) if use_color else f.path} — {f.kind.label}{tok}"
        )
        for r in f.reasons:
            lines.append(f"     • {r}")
    lines.append("")

    top = [f for f in sorted_findings if f.severity in (Severity.Critical, Severity.High)][:5]
    if top:
        lines.append(c("Priority Cleanup Targets (reduce AI friction fastest)", BOLD + UNDERLINE))
        for f in top:
            lines.append(f"  → {f.suggested_next_step}")
        lines.append("")

    return "\n".join(lines) + "\n"
