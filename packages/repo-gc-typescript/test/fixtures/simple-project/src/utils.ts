// A small utility file — should not trigger any findings
export function helper(): string {
  return "helper";
}

export function alsoUnused(): string {
  return "unused";
}

// Duplicate of math.ts doThingOne — triggers cross-file duplication detection
export function doThingOne(): string {
  const x = "hello world";
  const y = "goodbye world";
  return x + y;
}
