import type { Report } from './types';
import { renderTerminal } from './reporters/terminal';
import { renderJson } from './reporters/json';
import { renderMarkdown } from './reporters/markdown';
import { renderLlm } from './reporters/llm';

export function renderReport(report: Report, format: 'text' | 'json' | 'md' | 'llm', color: boolean): string {
  switch (format) {
    case 'json':
      return renderJson(report);
    case 'md':
      return renderMarkdown(report);
    case 'llm':
      return renderLlm(report);
    case 'text':
    default:
      return renderTerminal(report, color);
  }
}
