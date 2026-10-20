import type { Auth } from "@/bindings/Auth";
import type { Body } from "@/bindings/Body";
import type { KeyValue } from "@/bindings/KeyValue";
import type { RequestSpec } from "@/bindings/RequestSpec";

const SENSITIVE =
  /pass(word)?|pwd|secret|token|api[-_]?key|access[-_]?key|private[-_]?key|authorization|credential/i;
const JSON_PAIR = /"([^"\\]+)"\s*:\s*"((?:[^"\\]|\\.)*)"/g;

/** A value typed in directly; anything that uses a `{{variable}}` is left alone. */
const isLiteral = (value: string) => value.trim() !== "" && !value.includes("{{");

const sensitivePairs = (pairs: Iterable<[string, string]>, place: string) =>
  [...pairs]
    .filter(([key, value]) => SENSITIVE.test(key) && isLiteral(value))
    .map(([key]) => `${place} ${key}`);

const rows = (items: KeyValue[]) =>
  items.filter((i) => i.enabled).map((i): [string, string] => [i.key, i.value]);

const jsonPairs = (text: string) =>
  [...text.matchAll(JSON_PAIR)].map((m): [string, string] => [m[1], m[2]]);

/** Places in an auth setting that hold a credential typed in directly. */
export function authCredentials(auth: Auth | null): string[] {
  if (!auth) return [];
  switch (auth.type) {
    case "bearer":
      return isLiteral(auth.token) ? ["Auth Bearer token"] : [];
    case "basic":
      return isLiteral(auth.password) ? ["Auth Basic password"] : [];
    case "api_key":
      return isLiteral(auth.value) ? [`Auth API key ${auth.key}`] : [];
    case "oauth2":
      return isLiteral(auth.clientSecret) ? ["Auth OAuth 2.0 client secret"] : [];
    case "digest":
      return isLiteral(auth.password) ? ["Auth Digest password"] : [];
    case "jwt":
      return isLiteral(auth.secret) ? ["Auth JWT secret"] : [];
    case "aws_sigv4":
      return [
        ...(isLiteral(auth.secretKey) ? ["Auth AWS secret key"] : []),
        ...(isLiteral(auth.sessionToken) ? ["Auth AWS session token"] : []),
      ];
    case "oauth1":
      return [
        ...(isLiteral(auth.consumerSecret) ? ["Auth OAuth 1.0 consumer secret"] : []),
        ...(isLiteral(auth.tokenSecret) ? ["Auth OAuth 1.0 token secret"] : []),
      ];
    default:
      return [];
  }
}

function bodyCredentials(body: Body): string[] {
  switch (body.type) {
    case "json":
    case "raw":
      return sensitivePairs(jsonPairs(body.text), "Body");
    case "graphql":
      return sensitivePairs(jsonPairs(body.variables), "Variabel GraphQL");
    case "form_url_encoded":
      return sensitivePairs(rows(body.fields), "Body");
    case "multipart":
      return sensitivePairs(
        body.parts
          .filter((p) => p.type === "text" && p.enabled)
          .map((p): [string, string] => [p.key, p.type === "text" ? p.value : ""]),
        "Body",
      );
    default:
      return [];
  }
}

/** Places in a request where a credential is typed in directly instead of using a variable. */
export function requestCredentials(spec: RequestSpec): string[] {
  return [
    ...authCredentials(spec.auth),
    ...sensitivePairs(rows(spec.headers), "Header"),
    ...sensitivePairs(rows(spec.params), "Param"),
    ...bodyCredentials(spec.body),
  ];
}
