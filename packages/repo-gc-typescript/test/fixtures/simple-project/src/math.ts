// Utility math module — imported heavily by barrel
export function add(a: number, b: number): number {
  return a + b;
}
export function sub(a: number, b: number): number {
  return a - b;
}
export function mul(a: number, b: number): number {
  return a * b;
}
export function div(a: number, b: number): number {
  return a / b;
}
export function pow(a: number, b: number): number {
  return Math.pow(a, b);
}
export function sqrt(a: number): number {
  return Math.sqrt(a);
}
export function abs(a: number): number {
  return Math.abs(a);
}
export function min(a: number, b: number): number {
  return Math.min(a, b);
}
export function max(a: number, b: number): number {
  return Math.max(a, b);
}
export function round(a: number): number {
  return Math.round(a);
}
export function floor(a: number): number {
  return Math.floor(a);
}
export function ceil(a: number): number {
  return Math.ceil(a);
}
// Duplicate bodies for duplication detection
export function doThingOne(): string {
  const x = "hello world";
  const y = "goodbye world";
  return x + y;
}
export function doThingTwo(): string {
  const x = "hello world";
  const y = "goodbye world";
  return x + y;
}
export function doThingThree(): string {
  const x = "hello world";
  const y = "goodbye world";
  return x + y;
}
