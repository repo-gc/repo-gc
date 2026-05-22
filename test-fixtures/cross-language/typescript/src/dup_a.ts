// Code-duplication: identical function body to dup_b.ts
export function duplicateFunctionA(input: string): string {
  let result = '';
  for (const ch of input) {
    if (/[a-zA-Z]/.test(ch)) {
      result += ch.toUpperCase();
    } else if (/[0-9]/.test(ch)) {
      result += ch;
    } else {
      result += '_';
    }
  }
  return result;
}

export function anotherUniqueA(x: number): number {
  return x * x + 2 * x + 1;
}
