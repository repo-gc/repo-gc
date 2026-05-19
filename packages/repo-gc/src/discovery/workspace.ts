import * as fs from 'node:fs';
import * as path from 'node:path';

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

function readJson(filePath: string): Record<string, unknown> | null {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf-8'));
  } catch {
    return null;
  }
}

function findPackageRoots(root: string): string[] {
  const roots: string[] = [];

  const pkgJsonPath = path.join(root, 'package.json');
  const pkgJson = readJson(pkgJsonPath);
  if (pkgJson?.workspaces) {
    const ws = pkgJson.workspaces;
    if (Array.isArray(ws)) {
      for (const pattern of ws) {
        const [base, glob] = pattern.split('/*');
        if (glob === '*' || glob === undefined) {
          const basePath = path.join(root, base);
          if (fs.existsSync(basePath) && fs.statSync(basePath).isDirectory()) {
            for (const entry of fs.readdirSync(basePath)) {
              const pkgPath = path.join(basePath, entry);
              if (
                fs.existsSync(path.join(pkgPath, 'package.json')) &&
                fs.statSync(pkgPath).isDirectory()
              ) {
                roots.push(pkgPath);
              }
            }
          }
        }
      }
    }
  }

  const pnpmWs = path.join(root, 'pnpm-workspace.yaml');
  if (fs.existsSync(pnpmWs)) {
    try {
      const content = fs.readFileSync(pnpmWs, 'utf-8');
      const lines = content
        .split('\n')
        .filter((l) => l.trim().startsWith('-'))
        .map((l) => l.trim().replace(/^-\s*/, '').replace(/['"]/g, ''));
      for (const pattern of lines) {
        const [base] = pattern.split('/*');
        const basePath = path.join(root, base);
        if (fs.existsSync(basePath) && fs.statSync(basePath).isDirectory()) {
          for (const entry of fs.readdirSync(basePath)) {
            const pkgPath = path.join(basePath, entry);
            if (
              fs.existsSync(path.join(pkgPath, 'package.json')) &&
              fs.statSync(pkgPath).isDirectory()
            ) {
              if (!roots.includes(pkgPath)) roots.push(pkgPath);
            }
          }
        }
      }
    } catch {
      // ignore
    }
  }

  return roots;
}

function discoverPackage(pkgPath: string): PackageInfo {
  const pkgJson = readJson(path.join(pkgPath, 'package.json'));
  const name = (pkgJson?.name as string) || path.basename(pkgPath);

  const sourceRoots: string[] = [];
  const candidates = ['src', 'lib', 'app', '.'];
  for (const dir of candidates) {
    const full = path.join(pkgPath, dir);
    if (fs.existsSync(full) && fs.statSync(full).isDirectory()) {
      sourceRoots.push(full);
      if (dir === 'src') break;
    }
  }
  if (sourceRoots.length > 1 && sourceRoots.includes(pkgPath)) {
    sourceRoots.splice(sourceRoots.indexOf(pkgPath), 1);
  }

  const entryPoints: string[] = [];
  const main = pkgJson?.main as string | undefined;
  const bin = pkgJson?.bin as string | Record<string, string> | undefined;

  if (main) {
    const resolved = path.resolve(pkgPath, main);
    if (fs.existsSync(resolved)) entryPoints.push(resolved);
  }
  if (typeof bin === 'string') {
    const resolved = path.resolve(pkgPath, bin);
    if (fs.existsSync(resolved)) entryPoints.push(resolved);
  } else if (typeof bin === 'object' && bin) {
    for (const v of Object.values(bin)) {
      if (typeof v === 'string') {
        const resolved = path.resolve(pkgPath, v);
        if (fs.existsSync(resolved)) entryPoints.push(resolved);
      }
    }
  }

  const entryBasenames = ['index.ts', 'index.tsx', 'main.ts', 'cli.ts', 'server.ts', 'app.ts'];
  for (const searchDir of [pkgPath, ...sourceRoots]) {
    for (const ep of entryBasenames) {
      const resolved = path.join(searchDir, ep);
      if (!entryPoints.includes(resolved) && fs.existsSync(resolved)) {
        entryPoints.push(resolved);
      }
    }
  }

  return { name, manifestPath: path.join(pkgPath, 'package.json'), sourceRoots, entryPoints };
}

export function discoverWorkspace(startPath: string): WorkspaceInfo {
  const root = path.resolve(startPath);
  const pkgJsonPath = path.join(root, 'package.json');
  const rootPkg = readJson(pkgJsonPath);
  const hasWorkspaces = rootPkg?.workspaces !== undefined;
  const hasPnpmWs = fs.existsSync(path.join(root, 'pnpm-workspace.yaml'));
  const isWorkspace = hasWorkspaces || hasPnpmWs;

  let packages: PackageInfo[];
  if (isWorkspace) {
    const roots = findPackageRoots(root);
    packages = roots.map(discoverPackage);
    if (packages.length === 0) {
      packages = [discoverPackage(root)];
    }
  } else {
    packages = [discoverPackage(root)];
  }

  return { root, packages, isWorkspace };
}
