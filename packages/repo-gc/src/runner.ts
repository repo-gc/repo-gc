import {
  type Report,
  type Finding,
  type Thresholds,
  SEVERITY_WEIGHT,
  computeGlobalScore,
  estimateTokens,
  renderReport,
} from 'repo-gc-shared';
import { discoverWorkspace, enumerateFiles, detectLanguages } from './discovery';
import type { LanguagePlugin, AnalysisResult, SourceFile } from './plugin';

let _idSeq = 0;
function nextId(plugin: string): string {
  return `${plugin}-${String(++_idSeq).padStart(3, '0')}`;
}

export interface ScanOptions {
  path: string;
  format: 'text' | 'json' | 'md' | 'llm';
  threshold: Thresholds;
  includeTests: boolean;
  color: boolean;
  languages?: string[];
}

export async function scan(opts: ScanOptions, plugins: LanguagePlugin[]): Promise<string> {
  const report = await analyze(opts, plugins);
  return renderReport(report, opts.format, opts.color);
}

async function analyze(opts: ScanOptions, plugins: LanguagePlugin[]): Promise<Report> {
  _idSeq = 0;

  // 1. Workspace discovery
  const workspace = discoverWorkspace(opts.path);

  // 2. File enumeration per plugin
  const allFiles: SourceFile[] = [];
  let totalSkipped = 0;
  const allSourceRoots = new Set<string>();

  for (const pkg of workspace.packages) {
    for (const root of pkg.sourceRoots) {
      allSourceRoots.add(root);
    }
  }

  const activePlugins = opts.languages
    ? plugins.filter((p) => opts.languages!.includes(p.name))
    : plugins;

  for (const plugin of activePlugins) {
    // Self-discovering plugins handle their own file discovery — skip
    // umbrella enumeration since their source roots differ from JS packages
    if (plugin.selfDiscovers) continue;

    const { files, skipped } = enumerateFiles(
      [...allSourceRoots],
      workspace.root,
      plugin,
      opts.includeTests,
    );
    // Tag files with package metadata
    for (const pkg of workspace.packages) {
      for (const f of files) {
        if (f.path.startsWith(pkg.manifestPath.replace('/package.json', ''))) {
          f.metadata['npmPackage'] = pkg.name;
        }
      }
    }
    allFiles.push(...files);
    totalSkipped += skipped;
  }

  // 3. Auto-detect which plugins actually match
  const detected = detectLanguages(allFiles, activePlugins, workspace.root);

  if (allFiles.length === 0 && detected.length === 0) {
    return emptyReport(totalSkipped);
  }

  // 4. Run each detected plugin (with try/catch isolation)
  const allFindings: Finding[] = [];
  const allErrors: string[] = [];
  let filesAnalyzed = 0;

  for (const plugin of detected) {
    const pluginFiles = allFiles.filter((f) => f.language === plugin.name);
    // Self-discovering plugins don't need pre-enumerated files
    if (!plugin.selfDiscovers && pluginFiles.length === 0) continue;

    let result: AnalysisResult;
    try {
      result = await plugin.analyzeLanguage(pluginFiles, workspace.root, opts.threshold);
    } catch (err) {
      allErrors.push(`${plugin.name}: ${err instanceof Error ? err.message : String(err)}`);
      continue;
    }

    filesAnalyzed += pluginFiles.length;
    allErrors.push(...result.errors);

    // 5. Assign globally unique IDs
    for (const f of result.findings) {
      allFindings.push({ ...f, id: nextId(plugin.name) });
    }
  }

  // 6. Scoring
  const totalLines = allFiles.reduce((sum, f) => sum + f.lineCount, 0);
  const totalEstimatedTokens = allFiles.reduce((sum, f) => sum + estimateTokens(f.sizeBytes), 0);

  allFindings.sort((a, b) => SEVERITY_WEIGHT[b.severity] - SEVERITY_WEIGHT[a.severity]);

  const globalScore = computeGlobalScore(allFindings, allFiles.length, totalEstimatedTokens);

  return {
    findings: allFindings,
    global_score: globalScore,
    files_analyzed: filesAnalyzed,
    files_skipped: totalSkipped,
    total_lines: totalLines,
    total_estimated_tokens: totalEstimatedTokens,
  };
}

function emptyReport(skipped: number): Report {
  return {
    findings: [],
    global_score: {
      ai_friction_score: 0,
      context_waste_score: 0,
      structural_entropy_score: 0,
      context_waste_ratio: 0,
      estimated_waste_pct: 0,
    },
    files_analyzed: 0,
    files_skipped: skipped,
    total_lines: 0,
    total_estimated_tokens: 0,
  };
}
