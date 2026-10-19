// Chai-style assertions used by pm.expect and pm.response.to.
class AssertionError extends Error {}

function show(value) {
  try {
    return typeof value === "string"
      ? JSON.stringify(value)
      : (JSON.stringify(value) ?? String(value));
  } catch (_) {
    return String(value);
  }
}

function deepEqual(a, b) {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  const keys = Object.keys(a);
  if (keys.length !== Object.keys(b).length) return false;
  return keys.every((k) => deepEqual(a[k], b[k]));
}

function typeOf(value) {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  return typeof value;
}

function expect(actual) {
  const state = { negate: false };
  const chain = {};
  const check = (ok, message) => {
    if (ok === state.negate)
      throw new AssertionError(`expected ${show(actual)} ${state.negate ? "not " : ""}${message}`);
    return chain;
  };
  for (const word of ["to", "be", "been", "have", "has", "that", "and", "is", "which", "deep"]) {
    Object.defineProperty(chain, word, { get: () => chain });
  }
  Object.defineProperty(chain, "not", { get: () => ((state.negate = !state.negate), chain) });
  const flags = {
    ok: () => check(Boolean(actual), "to be truthy"),
    true: () => check(actual === true, "to be true"),
    false: () => check(actual === false, "to be false"),
    null: () => check(actual === null, "to be null"),
    undefined: () => check(actual === undefined, "to be undefined"),
    empty: () => check((actual?.length ?? Object.keys(actual ?? {}).length) === 0, "to be empty"),
  };
  for (const [name, fn] of Object.entries(flags)) Object.defineProperty(chain, name, { get: fn });

  chain.equal = chain.equals = (v) => check(actual === v, `to equal ${show(v)}`);
  chain.eql = (v) => check(deepEqual(actual, v), `to deeply equal ${show(v)}`);
  chain.a = chain.an = (type) => check(typeOf(actual) === type.toLowerCase(), `to be a ${type}`);
  chain.include =
    chain.contain =
    chain.includes =
      (v) =>
        check(
          typeof actual === "string" || Array.isArray(actual)
            ? actual.includes(v)
            : typeof v === "object"
              ? Object.entries(v).every(([k, val]) => deepEqual(actual?.[k], val))
              : actual != null && v in actual,
          `to include ${show(v)}`,
        );
  chain.property = (name, ...value) => {
    check(
      actual != null && Object.prototype.hasOwnProperty.call(actual, name),
      `to have property ${show(name)}`,
    );
    if (value.length)
      check(
        deepEqual(actual[name], value[0]),
        `to have property ${show(name)} of ${show(value[0])}`,
      );
    return chain;
  };
  chain.above = chain.greaterThan = chain.gt = (n) => check(actual > n, `to be above ${n}`);
  chain.below = chain.lessThan = chain.lt = (n) => check(actual < n, `to be below ${n}`);
  chain.least = chain.gte = (n) => check(actual >= n, `to be at least ${n}`);
  chain.most = chain.lte = (n) => check(actual <= n, `to be at most ${n}`);
  chain.lengthOf = chain.length = (n) => check(actual?.length === n, `to have length ${n}`);
  chain.match = (re) => check(re.test(String(actual)), `to match ${re}`);
  chain.oneOf = (list) =>
    check(
      list.some((v) => deepEqual(v, actual)),
      `to be one of ${show(list)}`,
    );
  return chain;
}
