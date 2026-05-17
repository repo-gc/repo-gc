mod cli;
mod discovery;
mod graph;
mod heuristics;
mod parsing;
mod reporting;
mod scoring;
mod types;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands, OutputFormat, Threshold};
use rayon::prelude::*;
use std::path::Path;
use types::Report;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan { path, format, threshold, include_tests, no_color } => {
            render_report(&analyze(&path, &threshold, include_tests)?, &format, no_color);
        }
        Commands::Report { path, format, threshold, include_tests } => {
            render_report(&analyze(&path, &threshold, include_tests)?, &format, false);
        }
        Commands::Json { path, threshold, include_tests } => {
            println!("{}", reporting::json_report::render(&analyze(&path, &threshold, include_tests)?)?);
        }
        Commands::Explain { path, root } => explain_file(&path, &root)?,
    }
    Ok(())
}

struct ParsedFile {
    file: discovery::RustFile,
    structure: parsing::FileStructure,
}

fn analyze(start: &Path, threshold: &Threshold, include_tests: bool) -> Result<Report> {
    let workspace = discovery::discover_workspace(start)
        .context("Failed to discover workspace")?;

    let files = discovery::enumerate_rust_files(&workspace.packages, &workspace.root, include_tests);

    if files.is_empty() {
        return Ok(Report {
            findings: vec![],
            global_score: types::GlobalScore {
                ai_friction_score: 0,
                context_waste_score: 0,
                structural_entropy_score: 0,
                context_waste_ratio: 0.0,
                estimated_waste_pct: 0,
            },
            files_analyzed: 0,
            files_skipped: 0,
            total_lines: 0,
            total_estimated_tokens: 0,
        });
    }

    // Parse in parallel; collect parse failures as files_skipped
    let (parsed, files_skipped): (Vec<_>, usize) = {
        let results: Vec<Result<ParsedFile, _>> = files
            .par_iter()
            .map(|f| {
                parsing::extract_structure(
                    &f.path,
                    f.relative_path.clone(),
                    f.package_name.clone(),
                )
                .map(|structure| ParsedFile { file: f.clone(), structure })
                .map_err(|e| (f.path.clone(), e))
            })
            .collect();

        let mut ok = vec![];
        let mut skipped = 0usize;
        for r in results {
            match r {
                Ok(pf) => ok.push(pf),
                Err((path, err)) => {
                    eprintln!("warn: skipping {} — {}", path.display(), err);
                    skipped += 1;
                }
            }
        }
        (ok, skipped)
    };

    let all_structures: Vec<_> = parsed.iter().map(|pf| pf.structure.clone()).collect();
    let graph = graph::ImportGraph::build(&all_structures, &workspace.root);

    let total_lines: usize = files.iter().map(|f| f.line_count).sum();
    let total_estimated_tokens: usize = files
        .iter()
        .map(|f| heuristics::context_bombs::estimate_tokens(f.size_bytes))
        .sum();

    let mut findings = vec![];
    let mut cb = 0usize;
    let mut re = 0usize;
    let mut ch = 0usize;
    let mut dw = 0usize;
    let mut ui = 0usize;

    for pf in &parsed {
        if let Some(f) =
            heuristics::context_bombs::analyze(&pf.file, &pf.structure, threshold, &mut cb)
        {
            findings.push(f);
        }
        if let Some(f) =
            heuristics::reexport_entropy::analyze(&pf.file, &pf.structure, threshold, &mut re)
        {
            findings.push(f);
        }
        if let Some(f) = heuristics::coupling::analyze(&pf.file, &graph, threshold, &mut ch) {
            findings.push(f);
        }
        if let Some(f) =
            heuristics::unused_imports::analyze(&pf.file, &pf.structure, threshold, &mut ui)
        {
            findings.push(f);
        }
    }

    let all_files: Vec<_> = parsed.iter().map(|pf| pf.file.clone()).collect();
    findings.extend(heuristics::dead_weight::analyze_orphaned_files(
        &all_files,
        &all_structures,
        &mut dw,
    ));
    let mut dup = 0usize;
    findings.extend(heuristics::duplication::analyze_duplicates(&all_structures, &mut dup));
    findings.sort_by(|a, b| b.severity.weight().partial_cmp(&a.severity.weight()).unwrap());

    Ok(Report {
        global_score: scoring::compute_global_score(&findings, files.len(), total_estimated_tokens),
        findings,
        files_analyzed: parsed.len(),
        files_skipped,
        total_lines,
        total_estimated_tokens,
    })
}

fn render_report(report: &Report, format: &OutputFormat, no_color: bool) {
    match format {
        OutputFormat::Text => reporting::terminal::render(report, no_color),
        OutputFormat::Json => println!(
            "{}",
            reporting::json_report::render(report).unwrap_or_else(|e| e.to_string())
        ),
        OutputFormat::Md => println!("{}", reporting::markdown::render(report)),
        OutputFormat::Llm => println!("{}", reporting::llm::render(report)),
    }
}

fn explain_file(target: &Path, root: &Path) -> Result<()> {
    let workspace = discovery::discover_workspace(root)?;
    let files = discovery::enumerate_rust_files(&workspace.packages, &workspace.root, true);
    let matching: Vec<_> = files
        .iter()
        .filter(|f| f.path == target || f.relative_path == target)
        .collect();

    if matching.is_empty() {
        eprintln!("File not found: {}", target.display());
        return Ok(());
    }
    let file = matching[0];
    let structure =
        parsing::extract_structure(&file.path, file.relative_path.clone(), file.package_name.clone())?;
    let all_structures: Vec<_> = files
        .par_iter()
        .filter_map(|f| {
            parsing::extract_structure(&f.path, f.relative_path.clone(), f.package_name.clone()).ok()
        })
        .collect();
    let graph = graph::ImportGraph::build(&all_structures, &workspace.root);

    println!("\n═══ {} ═══\n", file.relative_path.display());
    println!("Module path:           {}", structure.module_path);
    println!("Lines of code:         {}", file.line_count);
    println!(
        "Est. LLM tokens:       ~{}",
        heuristics::context_bombs::estimate_tokens(file.size_bytes)
    );
    println!(
        "Functions:             {} ({} public)",
        structure.function_count, structure.public_function_count
    );
    println!("Impl blocks:           {}", structure.impl_block_count);
    println!("Pub re-exports:        {}", structure.pub_use_paths.len());
    println!("Import fan-in:         {}", graph.get_fan_in(&file.path));
    println!("Import fan-out:        {}", graph.get_fan_out(&file.path));
    Ok(())
}
