import * as fs from 'node:fs';
import * as path from 'node:path';
import { parseSync } from 'oxc-parser';
import type {
  Program,
  Statement,
  ImportDeclaration,
  ExportNamedDeclaration,
  ExportAllDeclaration,
  ImportSpecifier,
  ImportDefaultSpecifier,
  ImportNamespaceSpecifier,
  Function as OxcFunction,
  ArrowFunctionExpression,
  IdentifierReference,
  JSXIdentifier,
} from '@oxc-project/types';
import { FileInfo, ReExportEntry, SourceFile } from '../types';

function isImportDecl(node: Statement): node is ImportDeclaration {
  return node.type === 'ImportDeclaration';
}
function isExportNamed(node: Statement): node is ExportNamedDeclaration {
  return node.type === 'ExportNamedDeclaration';
}
function isExportAll(node: Statement): node is ExportAllDeclaration {
  return node.type === 'ExportAllDeclaration';
}
function isFunction(node: unknown): node is OxcFunction {
  return (
    typeof node === 'object' &&
    node !== null &&
    (node as Record<string, unknown>).type === 'FunctionDeclaration'
  );
}
function isArrowFunction(node: unknown): node is ArrowFunctionExpression {
  return (
    typeof node === 'object' &&
    node !== null &&
    (node as Record<string, unknown>).type === 'ArrowFunctionExpression'
  );
}

/** Walk AST recursively, calling visitor for each node */
function walk(node: unknown, visitor: (node: Record<string, unknown>) => void): void {
  if (!node || typeof node !== 'object') return;
  const n = node as Record<string, unknown>;
  visitor(n);
  for (const key of Object.keys(n)) {
    if (key === 'parent') continue;
    const val = n[key];
    if (Array.isArray(val)) {
      for (const item of val) walk(item, visitor);
    } else if (typeof val === 'object' && val !== null) {
      walk(val, visitor);
    }
  }
}

function normalizeBody(body: string): string {
  return body.replace(/\s+/g, '').replace(/;/g, '');
}

function computeModulePath(relativePath: string, packageName: string): string {
  // Strip extension, replace / with ::, prepend package name
  const noExt = relativePath.replace(/\.(ts|tsx|js|jsx|mjs|cjs)$/, '');
  const parts = noExt.split('/');
  // If last segment is 'index', drop it
  if (parts[parts.length - 1] === 'index') parts.pop();
  return `${packageName}::${parts.join('::')}`;
}

/** Measure maximum control-flow nesting depth in the AST. */
function measureNestingDepth(root: unknown): number {
  let maxDepth = 0;

  function visit(node: unknown, depth: number): void {
    if (!node || typeof node !== 'object') return;
    const n = node as Record<string, unknown>;
    const type = n.type as string | undefined;
    if (!type) return;

    const isControlFlow =
      type === 'IfStatement' ||
      type === 'ForStatement' ||
      type === 'ForInStatement' ||
      type === 'ForOfStatement' ||
      type === 'WhileStatement' ||
      type === 'DoWhileStatement' ||
      type === 'SwitchStatement' ||
      type === 'TryStatement';

    const currentDepth = isControlFlow ? depth + 1 : depth;
    maxDepth = Math.max(maxDepth, currentDepth);

    for (const key of Object.keys(n)) {
      if (key === 'parent') continue;
      const val = n[key];
      if (Array.isArray(val)) {
        for (const item of val) visit(item, currentDepth);
      } else if (typeof val === 'object' && val !== null) {
        visit(val, currentDepth);
      }
    }
  }

  visit(root, 0);
  return maxDepth;
}

/** Count control-flow branches (if/else/for/while/switch/ternary). */
function countBranches(root: unknown): number {
  let count = 0;
  walk(root, (node) => {
    switch (node.type) {
      case 'IfStatement':
        count += 1;
        if (node.alternate) count += 1;
        break;
      case 'ForStatement':
      case 'ForInStatement':
      case 'ForOfStatement':
      case 'WhileStatement':
      case 'DoWhileStatement':
        count += 1;
        break;
      case 'SwitchStatement': {
        const cases = node.cases as unknown[] | undefined;
        if (Array.isArray(cases)) count += cases.length;
        break;
      }
      case 'ConditionalExpression':
        count += 1;
        break;
    }
  });
  return count;
}

/** Count empty catch blocks (catch clauses with no statements). */
function countEmptyCatches(root: unknown): number {
  let count = 0;
  walk(root, (node) => {
    if (node.type !== 'CatchClause') return;
    const body = node.body as Record<string, unknown> | undefined;
    if (!body) {
      count += 1;
      return;
    }
    const stmts = body.body as unknown[] | undefined;
    if (!Array.isArray(stmts) || stmts.length === 0) {
      count += 1;
    }
  });
  return count;
}

/** Count dangerous patterns: eval, Function constructor, any type, as any. */
function countDangerousPatterns(root: unknown): number {
  let count = 0;
  walk(root, (node) => {
    switch (node.type) {
      case 'CallExpression': {
        const callee = node.callee as Record<string, unknown> | undefined;
        if (callee?.type === 'Identifier' && callee.name === 'eval') {
          count += 1;
        }
        break;
      }
      case 'NewExpression': {
        const callee = node.callee as Record<string, unknown> | undefined;
        if (callee?.type === 'Identifier' && callee.name === 'Function') {
          count += 1;
        }
        break;
      }
      case 'TSAnyKeyword':
        count += 1;
        break;
      case 'TSAsExpression': {
        const typeAnn = node.typeAnnotation as Record<string, unknown> | undefined;
        if (typeAnn?.type === 'TSAnyKeyword') count += 1;
        break;
      }
      case 'TSTypeAssertion': {
        const typeAnn = node.typeAnnotation as Record<string, unknown> | undefined;
        if (typeAnn?.type === 'TSAnyKeyword') count += 1;
        break;
      }
    }
  });
  return count;
}

/** Count string literal comparisons (=== !== == != with a string literal operand). */
function countStringComparisons(root: unknown): number {
  let count = 0;
  const cmpOps = new Set(['==', '===', '!=', '!==']);
  walk(root, (node) => {
    if (node.type !== 'BinaryExpression') return;
    const op = node.operator as string;
    if (!cmpOps.has(op)) return;
    const left = node.left as Record<string, unknown> | undefined;
    const right = node.right as Record<string, unknown> | undefined;
    if (left?.type === 'StringLiteral' || right?.type === 'StringLiteral') {
      count += 1;
    }
  });
  return count;
}

/** Count comment-only lines from source text. */
function countCommentLines(sourceText: string): number {
  let count = 0;
  const lines = sourceText.split('\n');
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('//') || trimmed.startsWith('/*') || trimmed.startsWith('*') || trimmed === '*/') {
      count += 1;
    }
  }
  return count;
}

/** Recursively measure the nesting depth of a type expression. */
function measureTypeDepth(node: Record<string, unknown>): number {
  switch (node.type) {
    case 'TSTypeReference': {
      const typeParams = node.typeParameters as Record<string, unknown> | undefined;
      if (!typeParams) return 0;
      const params = typeParams.params as Record<string, unknown>[] | undefined;
      if (!Array.isArray(params) || params.length === 0) return 0;
      let maxInner = 0;
      for (const p of params) maxInner = Math.max(maxInner, measureTypeDepth(p));
      return 1 + maxInner;
    }
    case 'TSUnionType':
    case 'TSIntersectionType': {
      const types = node.types as Record<string, unknown>[] | undefined;
      if (!Array.isArray(types)) return 0;
      let maxInner = 0;
      for (const t of types) maxInner = Math.max(maxInner, measureTypeDepth(t));
      return 1 + maxInner;
    }
    case 'TSArrayType': {
      const elem = node.elementType as Record<string, unknown> | undefined;
      return elem ? 1 + measureTypeDepth(elem) : 0;
    }
    case 'TSTupleType': {
      const elems = node.elementTypes as Record<string, unknown>[] | undefined;
      if (!Array.isArray(elems)) return 0;
      let maxInner = 0;
      for (const e of elems) maxInner = Math.max(maxInner, measureTypeDepth(e));
      return 1 + maxInner;
    }
    default:
      return 0;
  }
}

export interface ParseResult {
  info: FileInfo;
  skipped: boolean;
  error?: string;
}

export function parseFile(file: SourceFile, workspaceRoot: string): ParseResult {
  const sourceText = fs.readFileSync(file.path, 'utf-8');

  let program: Program;
  try {
    const result = parseSync(file.path, sourceText, { showSemanticErrors: true });
    const hasErrors = result.errors.length > 0;
    if (hasErrors) {
      return {
        info: emptyFileInfo(file, workspaceRoot),
        skipped: true,
        error: result.errors.map((e) => e.message).join('; '),
      };
    }
    program = result.program;
  } catch (err) {
    return {
      info: emptyFileInfo(file, workspaceRoot),
      skipped: true,
      error: String(err),
    };
  }

  const imports: string[] = [];
  const exports: ReExportEntry[] = [];
  const importedNames: string[] = [];
  const allIdentifiers = new Set<string>();
  const functionBodies = new Map<string, string>();
  const functionBodiesRaw = new Map<string, string>();
  const jsxIdentifiers = new Set<string>();
  let functionCount = 0;
  let decoratorCount = 0;
  let maxTypeDepth = 0;

  walk(program, (node) => {
    switch (node.type) {
      case 'ImportDeclaration': {
        const decl = node as unknown as ImportDeclaration;
        const sourceVal = decl.source.value;
        imports.push(sourceVal);
        for (const spec of decl.specifiers) {
          if ((spec as ImportSpecifier).imported) {
            // Named import: import { X } or import { X as Y }
            importedNames.push((spec as ImportSpecifier).local.name);
          } else if ((spec as ImportDefaultSpecifier).local) {
            // Default import: import X from
            importedNames.push((spec as ImportDefaultSpecifier).local.name);
          } else if ((spec as ImportNamespaceSpecifier).local) {
            // Namespace import: import * as X
            importedNames.push((spec as ImportNamespaceSpecifier).local.name);
          }
        }
        break;
      }

      case 'ExportNamedDeclaration': {
        const decl = node as unknown as ExportNamedDeclaration;
        if (decl.source) {
          // Re-export: export { X } from './foo'
          const sourceVal = decl.source.value;
          let itemCount = 0;
          for (const spec of decl.specifiers) {
            itemCount++;
          }
          const hasWildcard = false;
          exports.push({ sourcePath: sourceVal, itemCount, isWildcard: hasWildcard });
        }
        // Also count local exports for re-export entropy
        if (decl.specifiers.length > 0 && !decl.source) {
          // export { X, Y } — local re-exports (without source)
          exports.push({ sourcePath: '<local>', itemCount: decl.specifiers.length, isWildcard: false });
        }
        break;
      }

      case 'ExportAllDeclaration': {
        const decl = node as unknown as ExportAllDeclaration;
        exports.push({
          sourcePath: decl.source.value,
          itemCount: -1, // unknown — wildcard re-exports everything
          isWildcard: true,
        });
        break;
      }

      case 'FunctionDeclaration': {
        const func = node as unknown as OxcFunction;
        functionCount++;
        if (func.body) {
          const bodyText = sourceText.slice(func.body.start, func.body.end);
          const normalized = normalizeBody(bodyText);
          if (normalized.length >= 40) {
            const fnName = func.id?.name || `anon_${func.start}`;
            functionBodies.set(fnName, normalized);
            functionBodiesRaw.set(fnName, bodyText);
          }
        }
        break;
      }

      case 'ArrowFunctionExpression': {
        const arrow = node as unknown as ArrowFunctionExpression;
        functionCount++;
        const bodyNode = arrow.body;
        if (bodyNode) {
          const bodyText = sourceText.slice(bodyNode.start, bodyNode.end);
          const normalized = normalizeBody(bodyText);
          if (normalized.length >= 40) {
            functionBodies.set(`anon_${arrow.start}`, normalized);
            functionBodiesRaw.set(`anon_${arrow.start}`, bodyText);
          }
        }
        break;
      }

      case 'Identifier': {
        // Only collect IdentifierReference, not IdentifierName/BindingIdentifier
        // In oxc, all Identifier* types have type "Identifier"
        // We narrow by checking parent context
        const ident = node as unknown as IdentifierReference;
        if (ident.name && typeof ident.name === 'string') {
          allIdentifiers.add(ident.name);
        }
        break;
      }

      case 'JSXIdentifier': {
        const jsxIdent = node as unknown as JSXIdentifier;
        if (jsxIdent.name && typeof jsxIdent.name === 'string') {
          jsxIdentifiers.add(jsxIdent.name);
        }
        break;
      }

      case 'Decorator': {
        decoratorCount++;
        break;
      }

      case 'TSTypeAnnotation': {
        const ta = node as Record<string, unknown>;
        const inner = ta.typeAnnotation as Record<string, unknown>;
        if (inner) {
          maxTypeDepth = Math.max(maxTypeDepth, measureTypeDepth(inner));
        }
        break;
      }
    }
  });

  // Merge JSX identifiers into all identifiers (for TSX import detection)
  for (const name of jsxIdentifiers) {
    allIdentifiers.add(name);
  }

  // Detect if this file is likely an entry point
  const basename = path.basename(file.relativePath);
  const isEntryPoint = basename.startsWith('index.') || basename === 'main.ts';

  return {
    info: {
      path: file.path,
      relativePath: file.relativePath,
      packageName: file.packageName,
      modulePath: computeModulePath(file.relativePath, file.packageName),
      functionCount,
      publicFunctionCount: 0, // would need scope analysis for export tracking
      functionBodies,
      functionBodiesRaw,
      imports,
      exports,
      allIdentifiers,
      importedNames,
      isEntryPoint,
      maxTypeDepth,
      decoratorCount,
      branchCount: countBranches(program),
      maxNestingDepth: measureNestingDepth(program),
      commentLineCount: countCommentLines(sourceText),
      emptyCatchCount: countEmptyCatches(program),
      dangerousPatternCount: countDangerousPatterns(program),
      stringComparisonCount: countStringComparisons(program),
    },
    skipped: false,
  };
}

function emptyFileInfo(file: SourceFile, workspaceRoot: string): FileInfo {
  return {
    path: file.path,
    relativePath: file.relativePath,
    packageName: file.packageName,
    modulePath: computeModulePath(file.relativePath, file.packageName),
    functionCount: 0,
    publicFunctionCount: 0,
    functionBodies: new Map(),
    functionBodiesRaw: new Map(),
    imports: [],
    exports: [],
    allIdentifiers: new Set(),
    importedNames: [],
    isEntryPoint: false,
    maxTypeDepth: 0,
    decoratorCount: 0,
    branchCount: 0,
    maxNestingDepth: 0,
    commentLineCount: 0,
    emptyCatchCount: 0,
    dangerousPatternCount: 0,
    stringComparisonCount: 0,
  };
}

export function parseAllFiles(
  files: SourceFile[],
  workspaceRoot: string,
): { parsed: ParseResult[]; skippedCount: number } {
  const parsed: ParseResult[] = [];
  let skippedCount = 0;

  for (const file of files) {
    const result = parseFile(file, workspaceRoot);
    if (result.skipped) skippedCount++;
    parsed.push(result);
  }

  return { parsed, skippedCount };
}
