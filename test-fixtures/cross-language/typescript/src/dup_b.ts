// Code-duplication: identical function body to dup_a.ts
export function duplicateFunctionB(input: string): string {
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

export function anotherUniqueB(y: number): number {
  return y * y + 3 * y + 2;
}
