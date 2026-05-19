import { Command } from 'commander';
import { getThresholds, type ThresholdLevel } from 'repo-gc-shared';
import { scan } from './runner';
import type { LanguagePlugin } from './plugin';

const program = new Command();

program
  .name('repo-gc')
  .description('Multi-language repository hygiene analyzer — find patterns that waste AI context')
  .version('0.2.0');

async function loadPlugins(): Promise<LanguagePlugin[]> {
  const plugins: LanguagePlugin[] = [];
  try {
    const { typescriptPlugin } = await import('repo-gc-typescript');
    plugins.push(typescriptPlugin);
  } catch {
    // optional peer dep — not installed
  }
  return plugins;
}

program
  .command('scan')
  .description('Scan the repository for AI-context-wasting patterns')
  .option('--path <path>', 'Target directory', '.')
  .option('--format <format>', 'Output format: text, json, md, llm', 'text')
  .option('--threshold <level>', 'Sensitivity: strict, normal, relaxed', 'normal')
  .option('--include-tests', 'Include test files in analysis', false)
  .option('--no-color', 'Plain text output (no ANSI codes)', false)
  .option('--lang <languages>', 'Comma-separated language plugin names (auto-detect if omitted)')
  .action(async (opts) => {
    const plugins = await loadPlugins();
    const threshold = getThresholds(opts.threshold as ThresholdLevel);
    const languages = opts.lang ? opts.lang.split(',').map((s: string) => s.trim()) : undefined;
    const report = await scan(
      { path: opts.path, format: opts.format as 'text' | 'json' | 'md' | 'llm', threshold, includeTests: opts.includeTests, color: opts.color !== false, languages },
      plugins,
    );
    process.stdout.write(report);
  });

program
  .command('report')
  .description('Generate an AI Context Efficiency report')
  .option('--path <path>', 'Target directory', '.')
  .option('--format <format>', 'Output format: text, json, md, llm', 'text')
  .option('--threshold <level>', 'Sensitivity: strict, normal, relaxed', 'normal')
  .option('--include-tests', 'Include test files', false)
  .option('--lang <languages>', 'Comma-separated language plugin names')
  .action(async (opts) => {
    const plugins = await loadPlugins();
    const threshold = getThresholds(opts.threshold as ThresholdLevel);
    const languages = opts.lang ? opts.lang.split(',').map((s: string) => s.trim()) : undefined;
    const report = await scan(
      { path: opts.path, format: opts.format as 'text' | 'json' | 'md' | 'llm', threshold, includeTests: opts.includeTests, color: true, languages },
      plugins,
    );
    process.stdout.write(report);
  });

program
  .command('json')
  .description('Emit findings as JSON (for CI integration)')
  .option('--path <path>', 'Target directory', '.')
  .option('--threshold <level>', 'Sensitivity: strict, normal, relaxed', 'normal')
  .option('--include-tests', 'Include test files', false)
  .option('--lang <languages>', 'Comma-separated language plugin names')
  .action(async (opts) => {
    const plugins = await loadPlugins();
    const threshold = getThresholds(opts.threshold as ThresholdLevel);
    const languages = opts.lang ? opts.lang.split(',').map((s: string) => s.trim()) : undefined;
    const report = await scan(
      { path: opts.path, format: 'json', threshold, includeTests: opts.includeTests, color: false, languages },
      plugins,
    );
    process.stdout.write(report);
  });

program
  .command('explain')
  .description('Explain why a file degrades AI context efficiency')
  .argument('<path>', 'File to analyze')
  .option('--root <root>', 'Workspace root', '.')
  .action(async (filePath, opts) => {
    console.log(`explain: ${filePath} (root: ${opts.root}) — coming in v0.2.0`);
  });

program.parse();
