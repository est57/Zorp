import { toCurl } from "./curl";
import { toGo } from "./go";
import { toAxios, toFetch } from "./javascript";
import type { SnippetRequest } from "./model";
import { toPython } from "./python";

export interface SnippetTarget {
  id: string;
  label: string;
  generate: (req: SnippetRequest) => string;
}

export const SNIPPET_TARGETS: SnippetTarget[] = [
  { id: "curl", label: "cURL", generate: toCurl },
  { id: "fetch", label: "JavaScript fetch", generate: toFetch },
  { id: "axios", label: "JavaScript axios", generate: toAxios },
  { id: "python", label: "Python requests", generate: toPython },
  { id: "go", label: "Go net/http", generate: toGo },
];

export { toSnippetRequest } from "./model";
export type { SnippetRequest } from "./model";
