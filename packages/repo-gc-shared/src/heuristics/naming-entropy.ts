import { Finding, FindingKind, Severity, type FileData, type Thresholds } from '../types';

/**
 * Classify an identifier string into a naming convention.
 *
 * - `snake_case`: contains `_`, all lowercase
 * - `camelCase`: starts lowercase, contains uppercase
 * - `PascalCase`: starts uppercase, contains lowercase
 * - `SCREAMING_SNAKE`: all uppercase with `_`
 * - `other`: doesn't match any convention
 */
function classifyIdentifier(name: string): string {
  if (name.length < 3) return 'other';

  const hasUnderscore = name.includes('_');
  const allUpper = name === name.toUpperCase();
  const allLower = name === name.toLowerCase();
  const firstUpper = name[0] === name[0].toUpperCase() && name[0] !== name[0].toLowerCase();
  const firstLower = name[0] === name[0].toLowerCase() && name[0] !== name[0].toUpperCase();

  if (hasUnderscore && allLower) return 'snake_case';
  if (hasUnderscore && allUpper) return 'SCREAMING_SNAKE';
  if (firstLower && /[A-Z]/.test(name)) return 'camelCase';
  if (firstUpper && /[a-z]/.test(name.slice(1))) return 'PascalCase';
  return 'other';
}

export function analyzeNamingEntropy(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  const identifiers = data.allIdentifiers;

  // Require a minimum number of identifiers to classify
  if (identifiers.size < 10) return null;

  // Classify each identifier
  const counts: Record<string, number> = {};
  let total = 0;

  for (const id of identifiers) {
    const convention = classifyIdentifier(id);
    counts[convention] = (counts[convention] || 0) + 1;
    total++;
  }

  // Find conventions that represent >5% of total
  const threshold = total * 0.05;
  const activeConventions = Object.entries(counts)
    .filter(([, count]) => count > threshold)
    .map(([conv]) => conv);

  // If 3+ conventions are in significant use, emit a finding
  if (activeConventions.length < 3) return null;

  // Find the dominant convention
  let dominantConvention = 'none';
  let maxCount = 0;
  for (const [conv, count] of Object.entries(counts)) {
    if (count > maxCount) {
      maxCount = count;
      dominantConvention = conv;
    }
  }

  return {
    id: `ne-${String(idCounter.value++).padStart(3, '0')}`,
    kind: FindingKind.NamingEntropy,
    severity: Severity.Low,
    confidence: 0.50,
    path: data.relativePath,
    summary: '',
    reasons: [],
    evidence: [
      `convention_counts: ${JSON.stringify(counts)}`,
      `dominant_convention: ${dominantConvention}`,
      `mixed_count: ${activeConventions.length}`,
    ],
    suggested_next_step: '',
  };
}
