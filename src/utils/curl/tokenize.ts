const ANSI_ESCAPES: Record<string, string> = {
  n: "\n",
  t: "\t",
  r: "\r",
  "'": "'",
  '"': '"',
  "\\": "\\",
};
const DOUBLE_QUOTE_ESCAPES = new Set(['"', "\\", "$", "`"]);

/** Splits a shell command into arguments, following POSIX quoting rules used by cURL snippets. */
export function tokenize(input: string): string[] {
  const text = input.replace(/\\\r?\n/g, " ");
  const tokens: string[] = [];
  let current = "";
  let inToken = false;
  let i = 0;

  const push = () => {
    if (inToken) tokens.push(current);
    current = "";
    inToken = false;
  };

  while (i < text.length) {
    const ch = text[i];
    if (/\s/.test(ch)) {
      push();
      i += 1;
    } else if (ch === "'") {
      const end = text.indexOf("'", i + 1);
      const stop = end < 0 ? text.length : end;
      current += text.slice(i + 1, stop);
      inToken = true;
      i = stop + 1;
    } else if (ch === "$" && text[i + 1] === "'") {
      i += 2;
      while (i < text.length && text[i] !== "'") {
        if (text[i] === "\\" && i + 1 < text.length) {
          current += ANSI_ESCAPES[text[i + 1]] ?? text[i + 1];
          i += 2;
        } else current += text[i++];
      }
      inToken = true;
      i += 1;
    } else if (ch === '"') {
      i += 1;
      while (i < text.length && text[i] !== '"') {
        if (text[i] === "\\" && DOUBLE_QUOTE_ESCAPES.has(text[i + 1])) {
          current += text[i + 1];
          i += 2;
        } else current += text[i++];
      }
      inToken = true;
      i += 1;
    } else if (ch === "\\" && i + 1 < text.length) {
      current += text[i + 1];
      inToken = true;
      i += 2;
    } else {
      current += ch;
      inToken = true;
      i += 1;
    }
  }
  push();
  return tokens;
}
