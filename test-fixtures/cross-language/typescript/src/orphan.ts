// Dead-weight: never imported by any other file
export function orphanFunction(input: string): string {
  return input.toUpperCase();
}

export class OrphanClass {
  name: string;
  value: number;

  constructor(name: string, value: number) {
    this.name = name;
    this.value = value;
  }

  process(): number {
    return this.value * 2;
  }
}
