export interface Segment {
  text: string;
  variable?: string;
}

const PATTERN = /\{\{\s*([^{}\s]+)\s*\}\}/g;

export function tokenize(text: string): Segment[] {
  const segments: Segment[] = [];
  let last = 0;
  for (const match of text.matchAll(PATTERN)) {
    const index = match.index ?? 0;
    if (index > last) segments.push({ text: text.slice(last, index) });
    segments.push({ text: match[0], variable: match[1] });
    last = index + match[0].length;
  }
  if (last < text.length) segments.push({ text: text.slice(last) });
  return segments;
}

export const isDynamic = (name: string) => name.startsWith("$");
