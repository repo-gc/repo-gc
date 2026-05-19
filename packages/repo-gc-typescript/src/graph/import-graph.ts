import * as path from 'node:path';
import { ResolverFactory } from 'unrs-resolver';
import { FileInfo, ImportGraph as IImportGraph } from '../types';

function buildModuleIndex(infos: FileInfo[]): Map<string, string> {
  const index = new Map<string, string>();
  for (const info of infos) {
    // Map both the module path and the relative path (without ext) to the absolute path
    index.set(info.modulePath, info.path);
    // Also map extension-stripped relative path
    const stripped = info.relativePath.replace(/\.(ts|tsx|js|jsx|mjs|cjs)$/, '');
    index.set(stripped, info.path);
  }
  return index;
}

export class ImportGraph implements IImportGraph {
  fanIn: Map<string, number> = new Map();
  fanOut: Map<string, number> = new Map();

  static build(infos: FileInfo[], workspaceRoot: string): ImportGraph {
    const graph = new ImportGraph();
    const moduleIndex = buildModuleIndex(infos);

    // Initialize all files with zero counts
    for (const info of infos) {
      graph.fanIn.set(info.path, 0);
      graph.fanOut.set(info.path, 0);
    }

    let resolver: ResolverFactory | null = null;
    try {
      resolver = new ResolverFactory({
        extensions: ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs', '.json'],
        mainFiles: ['index'],
        symlinks: true,
        conditionNames: ['import', 'require', 'node', 'default'],
        tsconfig: 'auto',
      });
    } catch {
      // unrs-resolver native addon may fail — fall back to simple resolution
    }

    for (const info of infos) {
      let fanOutCount = 0;

      for (const importPath of info.imports) {
        const resolved = resolveImport(importPath, info.path, moduleIndex, resolver, workspaceRoot);
        if (resolved) {
          fanOutCount++;
          const current = graph.fanIn.get(resolved) || 0;
          graph.fanIn.set(resolved, current + 1);
        }
      }

      // Also count re-export sources
      for (const exp of info.exports) {
        if (exp.sourcePath !== '<local>') {
          const resolved = resolveImport(exp.sourcePath, info.path, moduleIndex, resolver, workspaceRoot);
          if (resolved) {
            fanOutCount++;
            const current = graph.fanIn.get(resolved) || 0;
            graph.fanIn.set(resolved, current + 1);
          }
        }
      }

      graph.fanOut.set(info.path, fanOutCount);
    }

    return graph;
  }
}

function resolveImport(
  importPath: string,
  sourceFile: string,
  moduleIndex: Map<string, string>,
  resolver: ResolverFactory | null,
  workspaceRoot: string,
): string | null {
  // Skip bare specifiers (external packages) — they don't resolve to project files
  if (!importPath.startsWith('.') && !importPath.startsWith('/')) {
    return null;
  }

  // Try the resolver first
  if (resolver) {
    try {
      const result = resolver.sync(path.dirname(sourceFile), importPath);
      if (result.path && !result.error) {
        // Check if this resolves to one of our project files
        const resolved = path.resolve(result.path);
        if (moduleIndex.has(resolved)) return resolved;
        // Try matching by extension-stripped path
        const stripped = resolved.replace(/\.(ts|tsx|js|jsx|mjs|cjs)$/, '');
        for (const [key, val] of moduleIndex) {
          if (val === resolved || val === stripped + '.ts' || val === stripped + '.tsx' || val === stripped + '.js') {
            return val;
          }
        }
        return resolved; // return resolved path even if not in our index
      }
    } catch {
      // resolver failed, fall through
    }
  }

  // Fallback: simple relative resolution
  const sourceDir = path.dirname(sourceFile);
  const candidate = path.resolve(sourceDir, importPath);

  // Try with extensions and index files
  const extensions = ['.ts', '.tsx', '.js', '.jsx'];
  for (const ext of extensions) {
    const withExt = candidate + ext;
    if (moduleIndex.has(withExt)) return withExt;
    // Also try through the index
    for (const [key, val] of moduleIndex) {
      if (val === withExt) return val;
    }
  }

  // Try as directory with index file
  for (const ext of extensions) {
    const indexPath = path.join(candidate, `index${ext}`);
    if (moduleIndex.has(indexPath)) return indexPath;
    for (const [key, val] of moduleIndex) {
      if (val === indexPath) return val;
    }
  }

  return null;
}
