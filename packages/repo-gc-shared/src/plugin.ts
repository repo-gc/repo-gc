import type { Finding } from './types';
import type { Thresholds } from './threshold';

export interface SourceFile {
  path: string;
  relativePath: string;
  sizeBytes: number;
  lineCount: number;
  language: string;
  metadata: Record<string, string>;
}

export interface AnalysisResult {
  findings: Finding[];
  skipped: number;
  errors: string[];
}

export interface LanguagePlugin {
  name: string;
  displayName: string;
  fileExtensions: string[];
  testFilePatterns: RegExp[];
  requiredManifests?: string[];
  analyzeLanguage(files: SourceFile[], workspaceRoot: string, thresholds: Thresholds): AnalysisResult;
}
