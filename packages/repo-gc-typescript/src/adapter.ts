import type { FileData, FunctionBodyData } from 'repo-gc';
import type { FileInfo, ImportGraph } from './types';

// TS/JS keywords preserved during identifier normalization
const TS_KEYWORDS = new Set([
  'if', 'else', 'for', 'while', 'do', 'switch', 'case', 'break', 'continue',
  'return', 'throw', 'try', 'catch', 'finally', 'new', 'delete', 'typeof',
  'void', 'this', 'super', 'class', 'extends', 'implements', 'interface',
  'type', 'enum', 'namespace', 'module', 'import', 'export', 'default',
  'const', 'let', 'var', 'function', 'async', 'await', 'yield', 'of', 'in',
  'instanceof', 'true', 'false', 'null', 'undefined', 'from', 'as', 'get',
  'set', 'static', 'public', 'private', 'protected', 'readonly', 'abstract',
  'implements', 'declare', 'keyof', 'typeof', 'never', 'unknown', 'any',
  'boolean', 'number', 'string', 'symbol', 'object', 'bigint',
]);

/**
 * Replace identifiers with `_` for Type 2 clone detection.
 * Preserves language keywords so structural patterns survive normalization.
 */
function normalizeIdentifiers(rawBody: string): string {
  return rawBody
    .replace(/[a-zA-Z_$][a-zA-Z0-9_$]*/g, (match) => {
      if (TS_KEYWORDS.has(match)) return match;
      return '_';
    })
    .replace(/\s+/g, '')
    .replace(/;/g, '');
}

function buildFunctionBodies(
  info: FileInfo,
): FunctionBodyData[] {
  const bodies: FunctionBodyData[] = [];
  for (const [name, _whitespaceNormalized] of info.functionBodies) {
    const rawBody = info.functionBodiesRaw.get(name);
    if (!rawBody) continue;
    bodies.push({
      name,
      rawBody,
      normalizedBody: normalizeIdentifiers(rawBody),
    });
  }
  return bodies;
}

/** Detect if a file is a package init/barrel file */
function isPackageInit(relativePath: string): boolean {
  const basename = relativePath.split('/').pop() || '';
  return basename.startsWith('index.');
}

export function toFileData(
  info: FileInfo,
  fanIn: number,
  fanOut: number,
): FileData {
  return {
    path: info.path,
    relativePath: info.relativePath,
    lineCount: 0, // filled in by caller from SourceFile
    sizeBytes: 0, // filled in by caller from SourceFile
    modulePath: info.modulePath,
    isEntryPoint: info.isEntryPoint,
    isPackageInit: isPackageInit(info.relativePath),
    functionCount: info.functionCount,
    classCount: 0,
    implBlockCount: 0,
    imports: info.imports,
    importedNames: info.importedNames,
    allIdentifiers: info.allIdentifiers,
    moduleDeclarations: [],
    allExport: null,
    exports: info.exports,
    functionBodies: buildFunctionBodies(info),
  };
}

export function buildGraphData(graph: ImportGraph, fanIn: Map<string, number>, fanOut: Map<string, number>): { fanIn: Map<string, number>; fanOut: Map<string, number> } {
  return { fanIn, fanOut };
}
