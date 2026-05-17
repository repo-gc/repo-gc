use colored::Colorize;
use crate::types::{Report, Severity};

pub fn render(report: &Report, no_color: bool) {
    if no_color {
        colored::control::set_override(false);
    }

    println!("\n{}", "═══ repo-gc — AI Context Efficiency ═══".bold());
    println!(
        "Files: {}  Lines: {}  Est. LLM tokens: ~{}k",
        report.files_analyzed,
        report.total_lines,
        report.total_estimated_tokens / 1000
    );
    if report.files_skipped > 0 {
        println!("  ({} files skipped — parse errors)\n", report.files_skipped);
    } else {
        println!();
    }

    let gs = &report.global_score;
    println!("{}", "Global Scores (0-100, higher = worse)".bold().underline());
    println!(
        "  AI Friction:        {}  {} ({})",
        score_bar(gs.ai_friction_score),
        colorize(gs.ai_friction_score),
        score_label(gs.ai_friction_score)
    );
    println!(
        "  Context Waste:      {}  {} ({})",
        score_bar(gs.context_waste_score),
        colorize(gs.context_waste_score),
        score_label(gs.context_waste_score)
    );
    println!(
        "  Structural Entropy: {}  {} ({})\n",
        score_bar(gs.structural_entropy_score),
        colorize(gs.structural_entropy_score),
        score_label(gs.structural_entropy_score)
    );

    println!(
        "{}  Your repo wastes ~{}% of agent context capacity (~{:.1}x Claude sessions)\n",
        "▸".yellow().bold(),
        gs.estimated_waste_pct,
        gs.context_waste_ratio
    );

    if report.findings.is_empty() {
        println!("{}", "No issues found — your codebase is AI-friendly.".green().bold());
        return;
    }

    let mut sorted = report.findings.clone();
    sorted.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());

    println!("{} ({} total)", "Detected Patterns".bold().underline(), sorted.len());
    for (i, f) in sorted.iter().enumerate() {
        let tok = f
            .estimated_tokens
            .map(|t| format!(" (~{}k tokens)", t / 1000))
            .unwrap_or_default();
        println!(
            "\n  {}. [{}] {} — {}{}",
            i + 1,
            sev_colored(&f.severity),
            f.path.display().to_string().cyan(),
            f.kind.label(),
            tok
        );
        for r in &f.reasons {
            println!("     • {}", r);
        }
    }
    println!();

    let top: Vec<_> = sorted
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
        .take(5)
        .collect();
    if !top.is_empty() {
        println!("{}", "Priority Cleanup Targets (reduce AI friction fastest)".bold().underline());
        for f in top {
            println!("  → {}", f.suggested_next_step);
        }
        println!();
    }
}

fn score_bar(score: u32) -> String {
    let filled = (score / 10).min(10) as usize;
    let empty = 10 - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn score_label(score: u32) -> &'static str {
    if score >= 70 {
        "HIGH"
    } else if score >= 40 {
        "MEDIUM"
    } else {
        "LOW"
    }
}

fn colorize(score: u32) -> String {
    let s = format!("{}/100", score);
    if score >= 70 {
        s.red().bold().to_string()
    } else if score >= 40 {
        s.yellow().to_string()
    } else {
        s.green().to_string()
    }
}

fn sev_colored(s: &Severity) -> colored::ColoredString {
    match s {
        Severity::Critical => s.label().red().bold(),
        Severity::High => s.label().red(),
        Severity::Medium => s.label().yellow(),
        Severity::Low => s.label().normal(),
    }
}
