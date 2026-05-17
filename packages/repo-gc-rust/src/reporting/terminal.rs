use colored::Colorize;
use crate::types::{Report, Severity};

pub fn render(report: &Report, no_color: bool) {
    if no_color {
        colored::control::set_override(false);
    }

    println!("\n{}", "═══ repo-gc ═══".bold());
    println!(
        "Files: {}  Lines: {}  Est. tokens: ~{}k",
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
    println!("{}", "Global Scores".bold().underline());
    println!("  AI Hostility:  {}", colorize(gs.ai_hostility_score));
    println!("  Context Waste: {}", colorize(gs.context_waste_score));
    println!("  Entropy:       {}\n", colorize(gs.entropy_score));

    if report.findings.is_empty() {
        println!("{}", "No issues found.".green().bold());
        return;
    }

    let mut sorted = report.findings.clone();
    sorted.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());

    println!("{} ({} total)", "Findings".bold().underline(), sorted.len());
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
        println!("{}", "Next Cleanup Targets".bold().underline());
        for f in top {
            println!("  → {}", f.suggested_next_step);
        }
        println!();
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
