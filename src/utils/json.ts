// Unquoted {{variables}} make JSON invalid until they are resolved, so replace them for validation.
const withoutVariables = (text: string) => text.replace(/\{\{[^{}]+\}\}/g, "0");

export function jsonError(text: string): string | null {
  if (text.trim() === "") return null;
  try {
    JSON.parse(withoutVariables(text));
    return null;
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

export function prettyJson(text: string): string | null {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return null;
  }
}
