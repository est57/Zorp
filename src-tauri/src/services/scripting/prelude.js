// Builds the `pm` API from __input and collects results in __out. No I/O is exposed.
const __in = JSON.parse(globalThis.__input_json);
const __out = {
  env: {},
  envUnset: [],
  coll: {},
  collUnset: [],
  locals: {},
  tests: [],
  logs: [],
  error: null,
};

function __store(source, sets, unsets) {
  const values = Object.assign({}, source);
  return {
    get: (k) => values[k],
    has: (k) => k in values,
    set: (k, v) => {
      const value = typeof v === "string" ? v : JSON.stringify(v);
      values[k] = value;
      sets[k] = value;
      const i = unsets.indexOf(k);
      if (i >= 0) unsets.splice(i, 1);
    },
    unset: (k) => {
      delete values[k];
      delete sets[k];
      if (!unsets.includes(k)) unsets.push(k);
    },
    toObject: () => Object.assign({}, values),
  };
}

const __envStore = __store(__in.environment, __out.env, __out.envUnset);
const __env = Object.assign({}, __envStore, {
  set: (k, v) => {
    if (!__in.hasEnvironment)
      throw new Error(
        "Tidak ada environment aktif. Pilih environment sebelum memakai pm.environment.set.",
      );
    __envStore.set(k, v);
  },
});
const __coll = __store(__in.collectionVariables, __out.coll, __out.collUnset);
const __locals = __store({}, __out.locals, []);

const __req = __in.request;
const __headers = {
  get: (k) => __req.headers.find(([n]) => n.toLowerCase() === String(k).toLowerCase())?.[1],
  has: (k) => __headers.get(k) !== undefined,
  upsert: ({ key, value }) => {
    const row = __req.headers.find(([n]) => n.toLowerCase() === key.toLowerCase());
    if (row) row[1] = String(value);
    else __req.headers.push([key, String(value)]);
  },
  add: ({ key, value }) => __req.headers.push([key, String(value)]),
  remove: (k) =>
    (__req.headers = __req.headers.filter(([n]) => n.toLowerCase() !== String(k).toLowerCase())),
  toObject: () => Object.fromEntries(__req.headers),
};

function __response(res) {
  if (!res) return undefined;
  const header = (k) => res.headers.find(([n]) => n.toLowerCase() === String(k).toLowerCase())?.[1];
  const to = {
    have: {
      status: (code) => expect(typeof code === "number" ? res.code : res.status).to.equal(code),
      header: (k, v) =>
        v === undefined ? expect(header(k)).to.not.be.undefined : expect(header(k)).to.equal(v),
    },
    be: {},
  };
  Object.defineProperty(to.be, "ok", {
    get: () => expect(res.code >= 200 && res.code < 300, "").to.be.true,
  });
  return {
    code: res.code,
    status: res.status,
    responseTime: res.responseTime,
    headers: {
      get: header,
      has: (k) => header(k) !== undefined,
      toObject: () => Object.fromEntries(res.headers),
    },
    text: () => res.body,
    json: () => JSON.parse(res.body),
    to,
  };
}

const pm = {
  info: { requestName: __in.requestName, eventName: __in.kind === "pre" ? "prerequest" : "test" },
  environment: __env,
  collectionVariables: __coll,
  variables: {
    get: (k) => (__locals.has(k) ? __locals.get(k) : __in.variables[k]),
    set: (k, v) => __locals.set(k, v),
    has: (k) => __locals.has(k) || k in __in.variables,
  },
  request: {
    get method() {
      return __req.method;
    },
    set method(v) {
      __req.method = String(v).toUpperCase();
    },
    get url() {
      return __req.url;
    },
    set url(v) {
      __req.url = String(v);
    },
    headers: __headers,
    body: __req.body,
  },
  response: __response(__in.response),
  expect,
  test(name, fn) {
    try {
      fn();
      __out.tests.push({ name: String(name), passed: true, error: null });
    } catch (e) {
      __out.tests.push({
        name: String(name),
        passed: false,
        error: String(e && e.message ? e.message : e),
      });
    }
  },
  sendRequest() {
    throw new Error("pm.sendRequest belum didukung di Zorp");
  },
};

const __log =
  (level) =>
  (...args) =>
    __out.logs.push({
      level,
      message: args.map((a) => (typeof a === "string" ? a : __showValue(a))).join(" "),
    });
function __showValue(v) {
  try {
    return JSON.stringify(v);
  } catch (_) {
    return String(v);
  }
}
const console = {
  log: __log("log"),
  info: __log("info"),
  warn: __log("warn"),
  error: __log("error"),
};
const postman = {
  setEnvironmentVariable: (k, v) => pm.environment.set(k, v),
  getEnvironmentVariable: (k) => pm.environment.get(k),
};
const tests = {};
