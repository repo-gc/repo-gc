"""Token-optimized TSV reporter for LLM consumption — mirrors Rust's llm.rs."""

from ..types import Finding, FindingKind, Report


def _ev_val(evidence: list[str], key: str) -> str:
    prefix = f"{key}: "
    for e in evidence:
        if e.startswith(prefix):
            return e[len(prefix) :]
    return "-"


def _compact_summary(f: Finding) -> str:
    if f.kind == FindingKind.ContextBomb:
        lc = _ev_val(f.evidence, "line_count")
        tk = _ev_val(f.evidence, "estimated_tokens")
        s = f"{lc}ln/{tk}tk"
        for r in f.reasons:
            if "functions defined" in r:
                n = r.split()[0]
                s += f" +{n}fn"
            elif "impl blocks" in r:
                n = r.split()[0]
                s += f" +{n}impl"
        return s

    elif f.kind == FindingKind.CouplingHotspot:
        fi = _ev_val(f.evidence, "fan_in")
        fo = _ev_val(f.evidence, "fan_out")
        istab = _ev_val(f.evidence, "instability")
        pat = _ev_val(f.evidence, "pattern")
        return f"in={fi} out={fo} I={istab} {pat}"

    elif f.kind == FindingKind.DeadWeight:
        mp = _ev_val(f.evidence, "module_path")
        lc = _ev_val(f.evidence, "line_count")
        return f"{lc}ln mod={mp}"

    elif f.kind == FindingKind.ReexportEntropy:
        pu = _ev_val(f.evidence, "pub_use_count")
        ti = _ev_val(f.evidence, "total_reexported_items")
        wc = _ev_val(f.evidence, "has_wildcard")
        w = " +*" if wc == "true" else ""
        return f"{ti}sym/{pu}pu{w}"

    elif f.kind == FindingKind.CodeDuplication:
        files = len(f.evidence)
        fn_name = "?"
        if f.evidence:
            parts = f.evidence[0].split(" :: ", 1)
            if len(parts) > 1:
                fn_name = parts[1]
        return f"fn:{fn_name} x{files}"

    elif f.kind == FindingKind.UnusedImport:
        n = len(f.evidence)
        names: list[str] = []
        for e in f.evidence:
            if e.startswith("imported but unreferenced: "):
                names.append(e[len("imported but unreferenced: ") :])
        return f"{n}: {','.join(names[:4])}"

    return "-"


def _compact_next(f: Finding) -> str:
    if f.kind == FindingKind.ContextBomb:
        lc = _ev_val(f.evidence, "line_count")
        return f"split <{lc}ln"
    elif f.kind == FindingKind.CouplingHotspot:
        pat = _ev_val(f.evidence, "pattern")
        if pat == "api":
            return "verify api"
        elif pat == "orch":
            return "split deps"
        return "decouple"
    elif f.kind == FindingKind.DeadWeight:
        return "rm or re-export"
    elif f.kind == FindingKind.ReexportEntropy:
        return "flatten re-exports"
    elif f.kind == FindingKind.CodeDuplication:
        return "DRY: shared util"
    elif f.kind == FindingKind.UnusedImport:
        return "rm imports"
    return "-"


def render_llm(report: Report) -> str:
    gs = report.global_score
    header = (
        f"repo-gc\tfric={gs.ai_friction_score}\twaste={gs.context_waste_score}"
        f"\tent={gs.structural_entropy_score}\tpct={gs.estimated_waste_pct}"
        f"\trat={gs.context_waste_ratio:.1f}\tf={report.files_analyzed}"
        f"\tln={report.total_lines}\ttok={report.total_estimated_tokens // 1000}k\n"
    )

    if not report.findings:
        return header

    lines = [header, "id\tsev\tkind\tpath\ttok\tsummary\tnext\n"]

    sorted_findings = sorted(report.findings, key=lambda f: f.severity.weight, reverse=True)

    for f in sorted_findings:
        tok = str(f.estimated_tokens) if f.estimated_tokens is not None else "-"
        summary = _compact_summary(f)
        next_step = _compact_next(f)
        lines.append(
            f"{f.id}\t{f.severity.llm_label}\t{f.kind.llm_label}\t"
            f"{f.path}\t{tok}\t{summary}\t{next_step}\n"
        )

    return "".join(lines)
