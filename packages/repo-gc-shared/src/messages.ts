// Centralized user-facing messages for language plugins.
// All text strings that may appear in plugin output live here,
// never in the plugin source directly. This ensures messages are
// consistent, reviewable, and easy to update in one place.

// ── Process lifecycle errors (shared across all subprocess-based plugins) ──

/** Plugin-specific runtime was not found on PATH or at its expected location. */
export function runtimeNotFound(
  runtimeName: string,
  installHint: string,
): string {
  return `${runtimeName} not found. ${installHint}`;
}

/** Plugin directory (containing source/dist) is missing. */
export function pluginDirNotFound(
  pluginName: string,
  installHint: string,
): string {
  return `${pluginName} plugin directory not found. ${installHint}`;
}

/** The subprocess started but exited with a non-zero code. */
export function processExited(
  procName: string,
  code: number | null,
  stderr: string,
): string {
  return `${procName} exited with code ${code}: ${stderr || 'unknown error'}`;
}

/** The subprocess stdout could not be parsed as expected. */
export function outputParseFailed(
  procName: string,
  raw: string,
): string {
  return `Failed to parse ${procName} output: ${raw.slice(0, 200)}`;
}

/** The subprocess failed to spawn at all (binary not found in PATH, etc.). */
export function spawnFailed(
  procName: string,
  errorMessage: string,
): string {
  return `Failed to spawn ${procName}: ${errorMessage}`;
}
