import * as fs from 'node:fs';
import * as path from 'node:path';
import type { SourceFile, LanguagePlugin } from '../plugin';

export function detectLanguages(
  files: SourceFile[],
  plugins: LanguagePlugin[],
  root: string,
): LanguagePlugin[] {
  const extensions = new Set(files.map((f) => path.extname(f.path)));
  return plugins.filter((p) => {
    const hasFiles = p.fileExtensions.some((ext) => extensions.has(ext));
    const hasManifest =
      !p.requiredManifests ||
      p.requiredManifests.some((m) => fs.existsSync(path.join(root, m)));
    return hasFiles && hasManifest;
  });
}
