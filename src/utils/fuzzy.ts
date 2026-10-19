export interface FuzzyMatch {
  score: number;
  /** Indices of matched characters in the text, for highlighting. */
  indices: number[];
}

const isWordStart = (text: string, index: number) =>
  index === 0 || /[\s/_\-.:]/.test(text[index - 1]);

/** Subsequence match favouring consecutive characters and word starts. Returns null when not matched. */
export function fuzzyMatch(query: string, text: string): FuzzyMatch | null {
  const q = query.trim().toLowerCase();
  if (!q) return { score: 0, indices: [] };
  const t = text.toLowerCase();
  const indices: number[] = [];
  let score = 0;
  let from = 0;
  for (const ch of q) {
    if (ch === " ") continue;
    const index = t.indexOf(ch, from);
    if (index < 0) return null;
    const previous = indices.at(-1);
    score += 1;
    if (previous !== undefined && index === previous + 1) score += 4;
    if (isWordStart(text, index)) score += 3;
    if (previous !== undefined) score -= Math.min(index - previous - 1, 3) * 0.5;
    indices.push(index);
    from = index + 1;
  }
  if (t.startsWith(q)) score += 5;
  else if (t.includes(q)) score += 3;
  return { score: score - text.length * 0.01, indices };
}

/** Splits text into plain and highlighted segments. */
export function highlight(text: string, indices: number[]): { text: string; hit: boolean }[] {
  const hits = new Set(indices);
  const segments: { text: string; hit: boolean }[] = [];
  for (let i = 0; i < text.length; i += 1) {
    const hit = hits.has(i);
    const last = segments.at(-1);
    if (last && last.hit === hit) last.text += text[i];
    else segments.push({ text: text[i], hit });
  }
  return segments;
}
