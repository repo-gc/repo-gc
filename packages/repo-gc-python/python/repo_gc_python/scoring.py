"""Global score computation — direct port of Rust scoring.rs."""

from .types import Finding, FindingKind, GlobalScore


def compute_global_score(
    findings: list[Finding],
    total_files: int,
    total_estimated_tokens: int,
) -> GlobalScore:
    max_expected = max(total_files * 0.4, 1.0)

    def _weighted(ff: list[Finding]) -> float:
        return sum(f.severity.weight * f.confidence for f in ff)

    def _clamp(v: float) -> int:
        return min(round(v), 100)

    ai_friction = _clamp((_weighted(findings) / max_expected) * 100)

    context_waste = _clamp(
        (_weighted([f for f in findings if f.kind == FindingKind.ContextBomb]) / max_expected) * 100
    )

    # Original 5 structural signals
    structural_entropy = _clamp(
        (
            _weighted(
                [
                    f
                    for f in findings
                    if f.kind
                    in (
                        FindingKind.ReexportEntropy,
                        FindingKind.CouplingHotspot,
                        FindingKind.DeadWeight,
                        FindingKind.CodeDuplication,
                        FindingKind.UnusedImport,
                    )
                ]
            )
            / max_expected
        )
        * 100
    )

    # New signals: intra-file complexity and control-flow opacity
    reasoning_complexity = _clamp(
        (
            _weighted(
                [
                    f
                    for f in findings
                    if f.kind
                    in (
                        FindingKind.BranchDensity,
                        FindingKind.DeepNesting,
                        FindingKind.TypeComplexity,
                        FindingKind.CommentRatio,
                        FindingKind.ErrorSwallow,
                        FindingKind.DangerousPattern,
                        FindingKind.MutableGlobal,
                        FindingKind.NamingEntropy,
                        FindingKind.StringlyTyped,
                        FindingKind.ImportDiversity,
                        FindingKind.PlatformDensity,
                        FindingKind.ImplicitControl,
                    )
                ]
            )
            / max_expected
        )
        * 100
    )

    context_window_tokens = 128_000.0
    context_waste_ratio = total_estimated_tokens / context_window_tokens
    estimated_waste_pct = _clamp(
        (total_estimated_tokens / max(total_files * context_window_tokens, 1)) * 100
    )

    return GlobalScore(
        ai_friction_score=ai_friction,
        context_waste_score=context_waste,
        structural_entropy_score=structural_entropy,
        reasoning_complexity_score=reasoning_complexity,
        context_waste_ratio=context_waste_ratio,
        estimated_waste_pct=estimated_waste_pct,
    )
