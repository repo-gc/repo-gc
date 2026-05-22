import { Finding, type FileData, type Thresholds } from '../types';

export function analyzeMutableGlobal(
  data: FileData,
  thresholds: Thresholds,
  idCounter: { value: number },
): Finding | null {
  return null;
}
