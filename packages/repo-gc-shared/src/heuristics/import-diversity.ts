import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

/**
 * Extract a "domain" from an import specifier.
 *
 * - `"./utils"` / `"../foo"` → null (relative, skipped)
 * - `"react"` → `"react"`
 * - `"lodash/merge"` → `"lodash"`
 * - `"@angular/core"` → `"@angular"`  (scoped package — first segment)
 * - `"std::collections::HashMap"` → `"std"` (Rust-style path)
 * - `"crate::foo::bar"` → `"foo"`      (skip `crate`)
 * - `"super::baz"` / `"self::qux"` → null (relative)
 */
function extractDomain(spec: string): string | null {
  // Relative imports: starts with "." or ".."
  if (spec.startsWith('.')) return null;

  // Rust-style path separators
  if (spec.includes('::')) {
    const parts = spec.split('::');
    const first = parts[0];
    if (first === 'self' || first === 'super') return null;
    if (first === 'crate') {
      // Use the second segment (skip `crate`)
      if (parts.length < 2) return null;
      // Clean possible "as alias" suffix
      return parts[1].split(" as ")[0].trim();
    }
    // Clean possible "as alias" suffix from renamed imports
    return first.split(" as ")[0].trim();
  }

  // Standard /-separated import (npm-style)
  const parts = spec.split('/');
  // Handle scoped package: "@angular/core" → just "@angular"
  // The first segment already gives us "@angular"
  return parts[0];
}

export function analyzeImportDiversity(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const limit = thresholds.importDomainLimit;
  const domains = new Set<string>();

  for (const imp of data.imports) {
    const domain = extractDomain(imp);
    if (domain) {
      domains.add(domain);
    }
  }

  const distinctDomainCount = domains.size;

  if (distinctDomainCount <= limit) {
    return null;
  }

  // Severity: domains >= limit*3 → High, >= limit*2 → Medium, >= limit → Low
  let severity: Severity;
  if (distinctDomainCount >= limit * 3) {
    severity = Severity.High;
  } else if (distinctDomainCount >= limit * 2) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

  // Evidence: top 8 domains (comma-separated)
  const sortedDomains = [...domains].sort();
  const topDomains = sortedDomains.slice(0, 8).join(', ');

  return {
    id: `id-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.ImportDiversity,
    severity,
    confidence: 0.70,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `domain_count: ${distinctDomainCount}`,
      `domains: ${topDomains}`,
      `limit: ${limit}`,
    ],
    suggested_next_step: '',
    estimated_tokens: undefined,
  };
}
