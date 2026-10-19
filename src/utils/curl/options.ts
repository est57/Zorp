/** cURL options that take a value, mapped to a canonical name. */
export const VALUE_OPTIONS: Record<string, string> = {
  "-X": "request",
  "--request": "request",
  "-H": "header",
  "--header": "header",
  "-d": "data",
  "--data": "data",
  "--data-raw": "data-raw",
  "--data-ascii": "data",
  "--data-binary": "data",
  "--data-urlencode": "data-urlencode",
  "--json": "json",
  "-F": "form",
  "--form": "form",
  "--form-string": "form-string",
  "-u": "user",
  "--user": "user",
  "--url": "url",
  "-A": "user-agent",
  "--user-agent": "user-agent",
  "-b": "cookie",
  "--cookie": "cookie",
  "-e": "referer",
  "--referer": "referer",
  "-m": "max-time",
  "--max-time": "max-time",
  "-o": "ignored",
  "--output": "ignored",
  "-w": "ignored",
  "--write-out": "ignored",
  "--connect-timeout": "ignored",
  "-x": "proxy",
  "--proxy": "proxy",
  "-E": "cert",
  "--cert": "cert",
  "--cacert": "cert",
  "--key": "cert",
};

/** Flags without a value that change the request. */
export const FLAGS: Record<string, string> = {
  "-G": "get",
  "--get": "get",
  "-I": "head",
  "--head": "head",
  "-k": "insecure",
  "--insecure": "insecure",
  "-L": "location",
  "--location": "location",
};

const SHORT_WITH_VALUE = new Set(Object.keys(VALUE_OPTIONS).filter((o) => /^-[a-zA-Z]$/.test(o)));

export interface Option {
  name: string;
  value?: string;
}

/** Turns argv into options and positionals, handling `-XPOST`, `-sSL` and `--opt=value`. */
export function readOptions(args: string[]): {
  options: Option[];
  positionals: string[];
  unknown: string[];
} {
  const options: Option[] = [];
  const positionals: string[] = [];
  const unknown: string[] = [];
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (!arg.startsWith("-") || arg === "-") {
      positionals.push(arg);
      continue;
    }
    const eq = arg.startsWith("--") ? arg.indexOf("=") : -1;
    const flag = eq > 0 ? arg.slice(0, eq) : arg;
    if (VALUE_OPTIONS[flag]) {
      options.push({
        name: VALUE_OPTIONS[flag],
        value: eq > 0 ? arg.slice(eq + 1) : (args[++i] ?? ""),
      });
    } else if (FLAGS[flag]) {
      options.push({ name: FLAGS[flag] });
    } else if (!arg.startsWith("--") && SHORT_WITH_VALUE.has(arg.slice(0, 2))) {
      options.push({ name: VALUE_OPTIONS[arg.slice(0, 2)], value: arg.slice(2) });
    } else if (!arg.startsWith("--") && arg.length > 2) {
      // Combined short flags such as -sSL; the last one may take a value.
      for (const letter of arg.slice(1)) {
        const short = `-${letter}`;
        if (FLAGS[short]) options.push({ name: FLAGS[short] });
        else if (VALUE_OPTIONS[short])
          options.push({ name: VALUE_OPTIONS[short], value: args[++i] ?? "" });
      }
    } else {
      unknown.push(arg);
    }
  }
  return { options, positionals, unknown };
}
