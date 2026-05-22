import { Finding, FindingKind, Severity, type FileData } from '../types';
import { type Thresholds } from '../threshold';

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
  if (spec.startsWith('.')) return null;

  if (spec.includes('::')) {
    const parts = spec.split('::');
    const first = parts[0];
    if (first === 'self' || first === 'super') return null;
    if (first === 'crate') {
      if (parts.length < 2) return null;
      return parts[1].split(" as ")[0].trim();
    }
    return first.split(" as ")[0].trim();
  }

  const parts = spec.split('/');
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

  let severity: Severity;
  if (distinctDomainCount >= limit * 3) {
    severity = Severity.High;
  } else if (distinctDomainCount >= limit * 2) {
    severity = Severity.Medium;
  } else {
    severity = Severity.Low;
  }

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
