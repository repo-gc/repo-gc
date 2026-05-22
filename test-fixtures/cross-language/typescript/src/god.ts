// God module — high fan-in AND high fan-out → coupling-hotspot
import { helper } from './barrel';
import { bigFunction } from './big';
import { readFileSilent } from './barrel';
import { dangerousFunction } from './dangerous';

export function orchestrator(): void {
  helper();
  bigFunction();
  readFileSilent('/dev/null');
  dangerousFunction();
}
