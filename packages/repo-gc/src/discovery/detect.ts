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
    const hasManifest =
      !p.requiredManifests ||
      p.requiredManifests.some((m) => fs.existsSync(path.join(root, m)));
    // Plugins with requiredManifests handle their own file discovery —
    // the manifest check alone is sufficient. For others, require matching
    // file extensions found by the umbrella runner.
    if (p.requiredManifests && p.requiredManifests.length > 0) {
      return hasManifest;
    }
    const hasFiles = p.fileExtensions.some((ext) => extensions.has(ext));
    return hasFiles && hasManifest;
  });
}
