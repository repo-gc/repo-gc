// Entry point — should not be flagged as dead weight
import { helper } from './utils';
import { bigFunction } from './big-file';
import { add, sub } from './barrel';

export function main(): string {
  return helper() + bigFunction() + add(1, 2).toString();
}
