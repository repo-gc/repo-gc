// Internal intermediate types — AST-language-specific, not part of the output schema contract.
// Schema types (Severity, FindingKind, Finding, GlobalScore, Report) live in repo-gc.

export interface SourceFile {
  path: string;
  relativePath: string;
  packageName: string;
  sizeBytes: number;
  lineCount: number;
}

export interface ReExportEntry {
  sourcePath: string;
  itemCount: number;
  isWildcard: boolean;
}

export interface FileInfo {
  path: string;
  relativePath: string;
  packageName: string;
  modulePath: string;
  functionCount: number;
  publicFunctionCount: number;
  functionBodies: Map<string, string>;
  /** Raw (un-normalized) function body text for AST-aware analysis. */
  functionBodiesRaw: Map<string, string>;
  imports: string[];
  exports: ReExportEntry[];
  allIdentifiers: Set<string>;
  importedNames: string[];
  isEntryPoint: boolean;
}

export interface PackageInfo {
  name: string;
  manifestPath: string;
  sourceRoots: string[];
  entryPoints: string[];
}

export interface WorkspaceInfo {
  root: string;
  packages: PackageInfo[];
  isWorkspace: boolean;
}

export interface ImportGraph {
  fanIn: Map<string, number>;
  fanOut: Map<string, number>;
}
