try {
  new Function(__in.script)();
} catch (e) {
  __out.error =
    e instanceof SyntaxError
      ? `Syntax error: ${e.message}`
      : String(e && e.message ? e.message : e);
}
for (const [name, passed] of Object.entries(tests)) {
  __out.tests.push({
    name,
    passed: Boolean(passed),
    error: passed ? null : "Nilai tes bernilai false",
  });
}
__out.request = { method: __req.method, url: __req.url, headers: __req.headers };
JSON.stringify(__out);
