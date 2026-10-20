use serde_json::{Map, Value, json};

const MAX_DEPTH: usize = 8;
const MAX_REF_HOPS: usize = 16;

/// Read access to a specification with local `$ref` resolution.
pub struct Doc<'a> {
    pub root: &'a Value,
}

impl<'a> Doc<'a> {
    /// Follows local references such as `#/components/schemas/Pet`. External references
    /// are left as they are, since files outside the document are never read.
    pub fn resolve(&self, value: &'a Value) -> &'a Value {
        self.resolve_named(value).0
    }

    fn resolve_named(&self, mut value: &'a Value) -> (&'a Value, Option<&'a str>) {
        let mut name = None;
        for _ in 0..MAX_REF_HOPS {
            let Some(reference) = value.get("$ref").and_then(Value::as_str) else {
                break;
            };
            let Some(target) = reference
                .strip_prefix('#')
                .and_then(|p| self.root.pointer(p))
            else {
                break;
            };
            name = Some(reference);
            value = target;
        }
        (value, name)
    }

    /// An example value for a schema: its own example or default, otherwise one built
    /// from its type. Recursive schemas stop at the first repeat.
    pub fn example(&self, schema: &'a Value) -> Value {
        self.build(schema, 0, &mut Vec::new())
    }

    fn build(&self, schema: &'a Value, depth: usize, stack: &mut Vec<&'a str>) -> Value {
        let (schema, name) = self.resolve_named(schema);
        if depth > MAX_DEPTH || name.is_some_and(|n| stack.contains(&n)) {
            return Value::Null;
        }
        if let Some(n) = name {
            stack.push(n);
        }
        let value = self.build_resolved(schema, depth, stack);
        if name.is_some() {
            stack.pop();
        }
        value
    }

    fn build_resolved(&self, s: &'a Value, depth: usize, stack: &mut Vec<&'a str>) -> Value {
        for key in ["example", "default"] {
            if let Some(v) = s.get(key) {
                return v.clone();
            }
        }
        if let Some(Value::Array(examples)) = s.get("examples")
            && let Some(first) = examples.first()
        {
            return first.clone();
        }
        if let Some(first) = s
            .get("enum")
            .and_then(Value::as_array)
            .and_then(|e| e.first())
        {
            return first.clone();
        }
        if let Some(all) = s.get("allOf").and_then(Value::as_array) {
            let mut merged = Map::new();
            for part in all {
                if let Value::Object(obj) = self.build(part, depth + 1, stack) {
                    merged.extend(obj);
                }
            }
            return Value::Object(merged);
        }
        for key in ["oneOf", "anyOf"] {
            if let Some(first) = s.get(key).and_then(Value::as_array).and_then(|v| v.first()) {
                return self.build(first, depth + 1, stack);
            }
        }
        match type_of(s) {
            "object" => {
                let mut obj = Map::new();
                if let Some(props) = s.get("properties").and_then(Value::as_object) {
                    for (key, prop) in props {
                        let resolved = self.resolve(prop);
                        if resolved.get("readOnly").and_then(Value::as_bool) == Some(true) {
                            continue;
                        }
                        obj.insert(key.clone(), self.build(prop, depth + 1, stack));
                    }
                }
                Value::Object(obj)
            }
            "array" => match s.get("items") {
                Some(items) => match self.build(items, depth + 1, stack) {
                    Value::Null => json!([]),
                    item => json!([item]),
                },
                None => json!([]),
            },
            "integer" => json!(0),
            "number" => json!(0.0),
            "boolean" => json!(false),
            "string" => json!(string_example(s)),
            _ => Value::Null,
        }
    }
}

fn type_of(s: &Value) -> &str {
    match s.get("type") {
        Some(Value::String(t)) => t,
        // OpenAPI 3.1 allows a list such as ["string", "null"].
        Some(Value::Array(types)) => types
            .iter()
            .filter_map(Value::as_str)
            .find(|t| *t != "null")
            .unwrap_or(""),
        _ if s.get("properties").is_some() => "object",
        _ if s.get("items").is_some() => "array",
        _ => "",
    }
}

fn string_example(s: &Value) -> &'static str {
    match s.get("format").and_then(Value::as_str).unwrap_or("") {
        "date-time" => "2026-01-01T00:00:00Z",
        "date" => "2026-01-01",
        "email" => "user@example.com",
        "uuid" => "00000000-0000-0000-0000-000000000000",
        "uri" | "url" => "https://example.com",
        "binary" | "byte" => "",
        _ => "string",
    }
}

/// Text form of an example, used for parameters and variables.
pub fn as_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
