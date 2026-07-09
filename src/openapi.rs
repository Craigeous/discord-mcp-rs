//! Parses Discord's OpenAPI 3.1 spec into a registry of MCP tools.
//!
//! Every operation in the spec becomes one MCP tool whose name is the
//! `operationId`. The generated JSON Schema for each tool exposes:
//!   * path parameters as required top-level properties,
//!   * query parameters as top-level properties,
//!   * the `application/json` request body (if any) under a `body` property,
//!   * file uploads (multipart `contentEncoding: binary` fields) as local
//!     filesystem paths.
//!
//! Each tool's schema is self-contained: the transitive closure of
//! `#/components/schemas/...` references it needs is inlined under `$defs`,
//! and every `$ref` is rewritten to point at `#/$defs/...`.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use rmcp::model::{JsonObject, Tool};
use serde_json::{Map, Value, json};

/// HTTP methods we treat as operations.
const HTTP_METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

/// How a multipart-capable operation exposes its file uploads.
#[derive(Clone, Debug)]
pub enum FileMode {
    /// No file uploads.
    None,
    /// Each named binary field is its own string property holding a local path.
    Named(Vec<String>),
    /// A `files` array of local paths, sent as `files[0]`, `files[1]`, ...
    Array,
}

/// A fully-resolved operation ready to dispatch.
#[derive(Clone)]
pub struct Operation {
    pub operation_id: String,
    /// Uppercase HTTP method, e.g. `POST`.
    pub method: String,
    /// Path template with `{placeholders}`, relative to the base URL.
    pub path_template: String,
    pub path_params: Vec<String>,
    /// Names of query parameters this operation accepts.
    pub query_params: Vec<String>,
    pub file_mode: FileMode,
    /// Prebuilt MCP tool definition (name + description + input schema).
    pub tool: Tool,
}

/// The registry of all operations parsed from a spec.
pub struct Registry {
    operations: Vec<Operation>,
    by_name: HashMap<String, usize>,
    pub base_url: String,
}

impl Registry {
    /// Parse an OpenAPI spec (JSON text) into a registry.
    pub fn from_spec_str(spec: &str) -> anyhow::Result<Self> {
        let doc: Value = serde_json::from_str(spec)?;

        let base_url = doc
            .get("servers")
            .and_then(|s| s.as_array())
            .and_then(|a| a.first())
            .and_then(|s| s.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("https://discord.com/api/v10")
            .trim_end_matches('/')
            .to_string();

        let empty = Map::new();
        let schemas = doc
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(|s| s.as_object())
            .unwrap_or(&empty);

        let paths = doc
            .get("paths")
            .and_then(|p| p.as_object())
            .ok_or_else(|| anyhow::anyhow!("spec has no `paths` object"))?;

        let mut operations = Vec::new();
        let mut by_name = HashMap::new();

        for (path, item) in paths {
            let Some(item) = item.as_object() else { continue };
            for method in HTTP_METHODS {
                let Some(op) = item.get(method).and_then(|o| o.as_object()) else {
                    continue;
                };
                let Some(operation_id) = op.get("operationId").and_then(|v| v.as_str()) else {
                    tracing::warn!("skipping {} {path}: no operationId", method.to_uppercase());
                    continue;
                };

                let operation =
                    build_operation(operation_id, method, path, op, schemas);
                by_name.insert(operation.operation_id.clone(), operations.len());
                operations.push(operation);
            }
        }

        Ok(Self {
            operations,
            by_name,
            base_url,
        })
    }

    /// All tool definitions, for `list_tools`.
    pub fn tools(&self) -> Vec<Tool> {
        self.operations.iter().map(|o| o.tool.clone()).collect()
    }

    /// Look up an operation by tool name (operationId).
    pub fn get(&self, name: &str) -> Option<&Operation> {
        self.by_name.get(name).map(|&i| &self.operations[i])
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

/// Build one `Operation` (including its MCP tool) from a spec operation object.
fn build_operation(
    operation_id: &str,
    method: &str,
    path: &str,
    op: &Map<String, Value>,
    schemas: &Map<String, Value>,
) -> Operation {
    let mut properties = Map::new();
    let mut required: Vec<Value> = Vec::new();
    let mut seed_refs: BTreeSet<String> = BTreeSet::new();

    let mut path_params: Vec<String> = Vec::new();
    let mut query_params: Vec<String> = Vec::new();

    // --- parameters (path + query) ---
    if let Some(params) = op.get("parameters").and_then(|p| p.as_array()) {
        for p in params {
            let Some(name) = p.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let loc = p.get("in").and_then(|v| v.as_str()).unwrap_or("");
            let is_path = loc == "path";
            let req = is_path
                || p.get("required").and_then(|b| b.as_bool()).unwrap_or(false);

            let mut schema = p
                .get("schema")
                .cloned()
                .unwrap_or_else(|| json!({ "type": "string" }));
            collect_refs(&schema, &mut seed_refs);
            annotate(&mut schema, p.get("description").and_then(|d| d.as_str()), loc);

            match loc {
                "path" => path_params.push(name.to_string()),
                "query" => query_params.push(name.to_string()),
                // Discord's spec has no header/cookie params; ignore if present.
                _ => continue,
            }

            properties.insert(name.to_string(), schema);
            if req {
                required.push(Value::String(name.to_string()));
            }
        }
    }

    // Safety net: ensure every `{placeholder}` in the path is modeled.
    for ph in path_placeholders(path) {
        if !path_params.contains(&ph) {
            path_params.push(ph.clone());
            properties.insert(
                ph.clone(),
                json!({ "type": "string", "description": "Path parameter" }),
            );
            required.push(Value::String(ph));
        }
    }

    // --- request body / file uploads ---
    let mut has_json_body = false;
    let mut file_mode = FileMode::None;

    if let Some(rb) = op.get("requestBody") {
        let body_required = rb.get("required").and_then(|b| b.as_bool()).unwrap_or(false);
        let content = rb.get("content");
        let json_schema = content
            .and_then(|c| c.get("application/json"))
            .and_then(|j| j.get("schema"));
        let mp_schema = content
            .and_then(|c| c.get("multipart/form-data"))
            .and_then(|j| j.get("schema"));

        if let Some(js) = json_schema {
            collect_refs(js, &mut seed_refs);
            properties.insert("body".to_string(), js.clone());
            has_json_body = true;
            if body_required {
                required.push(Value::String("body".to_string()));
            }
        }

        if let Some(ms) = mp_schema {
            // The multipart schema may compose the body via allOf/$ref, so
            // flatten it to reach the binary (file) fields.
            let mut all_props = Map::new();
            let mut seen = BTreeSet::new();
            flatten_props(ms, schemas, &mut all_props, &mut seen);

            let mut binary_fields: Vec<String> = Vec::new();
            let mut nonbinary = Map::new();
            for (fname, fsch) in &all_props {
                if is_binary(fsch) {
                    binary_fields.push(fname.clone());
                } else {
                    nonbinary.insert(fname.clone(), fsch.clone());
                }
            }

            // If there was no JSON body, expose the non-file multipart fields
            // as the `body` object (sent as the `payload_json` part).
            if !has_json_body && !nonbinary.is_empty() {
                let body_schema = json!({ "type": "object", "properties": nonbinary });
                collect_refs(&body_schema, &mut seed_refs);
                properties.insert("body".to_string(), body_schema);
                has_json_body = true;
                if body_required {
                    required.push(Value::String("body".to_string()));
                }
            }

            if binary_fields.len() == 1 && !binary_fields[0].contains('[') {
                let fname = binary_fields.remove(0);
                properties.insert(
                    fname.clone(),
                    json!({
                        "type": "string",
                        "description": format!(
                            "Local filesystem path to upload as the `{fname}` file part"
                        ),
                    }),
                );
                if body_required && !has_json_body {
                    required.push(Value::String(fname.clone()));
                }
                file_mode = FileMode::Named(vec![fname]);
            } else if !binary_fields.is_empty() {
                properties.insert(
                    "files".to_string(),
                    json!({
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Local filesystem paths to attach as file uploads (sent as files[0], files[1], ...)",
                    }),
                );
                file_mode = FileMode::Array;
            }
        }
    }

    // --- assemble self-contained input schema with inlined $defs ---
    let closure = ref_closure(&seed_refs, schemas);
    let mut defs = Map::new();
    for name in &closure {
        if let Some(def) = schemas.get(name) {
            defs.insert(name.clone(), def.clone());
        }
    }

    let mut input = Map::new();
    input.insert("type".to_string(), Value::String("object".to_string()));
    input.insert("properties".to_string(), Value::Object(properties));
    if !required.is_empty() {
        input.insert("required".to_string(), Value::Array(required));
    }
    if !defs.is_empty() {
        input.insert("$defs".to_string(), Value::Object(defs));
    }

    let mut input_val = Value::Object(input);
    rewrite_refs(&mut input_val);
    let input_obj: JsonObject = match input_val {
        Value::Object(m) => m,
        _ => unreachable!("input schema is always an object"),
    };

    let description = build_description(op, method, path);
    let tool = Tool::new(
        operation_id.to_string(),
        description,
        Arc::new(input_obj),
    );

    Operation {
        operation_id: operation_id.to_string(),
        method: method.to_uppercase(),
        path_template: path.to_string(),
        path_params,
        query_params,
        file_mode,
        tool,
    }
}

/// Human-readable tool description. Discord's spec omits summaries, so we
/// synthesize `METHOD /path` and fold in a summary/description when present.
fn build_description(op: &Map<String, Value>, method: &str, path: &str) -> String {
    let route = format!("{} {}", method.to_uppercase(), path);
    let text = op
        .get("summary")
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            op.get("description")
                .and_then(|s| s.as_str())
                .filter(|s| !s.is_empty())
        });
    match text {
        Some(t) => format!("{t} [{route}]"),
        None => route,
    }
}

/// Attach a description noting where the parameter goes, if the schema is an
/// object we can annotate without clobbering an existing description.
fn annotate(schema: &mut Value, desc: Option<&str>, loc: &str) {
    if let Value::Object(m) = schema
        && !m.contains_key("description") {
            let note = match desc {
                Some(d) => format!("{d} ({loc} parameter)"),
                None => format!("{loc} parameter"),
            };
            m.insert("description".to_string(), Value::String(note));
        }
}

/// Is this multipart field a binary file upload?
fn is_binary(schema: &Value) -> bool {
    schema.get("contentEncoding").and_then(|e| e.as_str()) == Some("binary")
        || schema.get("format").and_then(|e| e.as_str()) == Some("binary")
}

/// Flatten a (possibly `allOf`/`$ref`-composed) object schema into a flat map
/// of property name -> property schema. Used to find file fields inside
/// multipart bodies, which Discord composes from the JSON body plus a
/// `files[n]` block. Cyclic `$ref`s are guarded by `seen`.
fn flatten_props(
    schema: &Value,
    schemas: &Map<String, Value>,
    out: &mut Map<String, Value>,
    seen: &mut BTreeSet<String>,
) {
    let Value::Object(m) = schema else { return };

    if let Some(name) = m
        .get("$ref")
        .and_then(|v| v.as_str())
        .and_then(|r| r.strip_prefix("#/components/schemas/"))
        && seen.insert(name.to_string())
            && let Some(def) = schemas.get(name) {
                flatten_props(def, schemas, out, seen);
            }
    if let Some(all) = m.get("allOf").and_then(|v| v.as_array()) {
        for member in all {
            flatten_props(member, schemas, out, seen);
        }
    }
    if let Some(props) = m.get("properties").and_then(|v| v.as_object()) {
        for (k, v) in props {
            out.insert(k.clone(), v.clone());
        }
    }
}

/// Extract `{placeholder}` names from a path template.
fn path_placeholders(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        if let Some(end) = rest[start..].find('}') {
            out.push(rest[start + 1..start + end].to_string());
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }
    out
}

/// Collect the names of all `#/components/schemas/...` refs in a value.
fn collect_refs(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(m) => {
            for (k, vv) in m {
                if k == "$ref" {
                    if let Some(name) = vv
                        .as_str()
                        .and_then(|r| r.strip_prefix("#/components/schemas/"))
                    {
                        out.insert(name.to_string());
                    }
                } else {
                    collect_refs(vv, out);
                }
            }
        }
        Value::Array(a) => {
            for vv in a {
                collect_refs(vv, out);
            }
        }
        _ => {}
    }
}

/// Transitive closure of schema references reachable from a seed set.
fn ref_closure(seed: &BTreeSet<String>, schemas: &Map<String, Value>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<String> = seed.iter().cloned().collect();
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(def) = schemas.get(&name) {
            let mut refs = BTreeSet::new();
            collect_refs(def, &mut refs);
            for r in refs {
                if !seen.contains(&r) {
                    stack.push(r);
                }
            }
        }
    }
    seen
}

/// Rewrite every `#/components/schemas/X` ref to `#/$defs/X` in place.
fn rewrite_refs(v: &mut Value) {
    match v {
        Value::Object(m) => {
            for (k, vv) in m.iter_mut() {
                if k == "$ref" {
                    if let Value::String(r) = vv
                        && let Some(name) = r.strip_prefix("#/components/schemas/") {
                            *r = format!("#/$defs/{name}");
                        }
                } else {
                    rewrite_refs(vv);
                }
            }
        }
        Value::Array(a) => {
            for vv in a.iter_mut() {
                rewrite_refs(vv);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = include_str!("../spec/openapi.json");

    #[test]
    fn parses_all_operations() {
        let reg = Registry::from_spec_str(SPEC).expect("spec parses");
        // The vendored spec exposes 242 operations.
        assert!(reg.len() >= 240, "expected ~242 tools, got {}", reg.len());
        assert_eq!(reg.base_url, "https://discord.com/api/v10");
    }

    #[test]
    fn every_tool_schema_is_a_valid_object_with_no_dangling_refs() {
        let reg = Registry::from_spec_str(SPEC).expect("spec parses");
        for op in &reg.operations {
            let schema = &op.tool.input_schema;
            assert_eq!(
                schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "{} schema must be an object",
                op.operation_id
            );
            // No ref may still point at components; all must be rebased to $defs.
            let text = serde_json::to_string(schema).unwrap();
            assert!(
                !text.contains("#/components/schemas/"),
                "{} has unrewritten component refs",
                op.operation_id
            );
        }
    }

    #[test]
    fn create_message_has_body_and_file_uploads() {
        let reg = Registry::from_spec_str(SPEC).expect("spec parses");
        let op = reg.get("create_message").expect("create_message exists");
        assert_eq!(op.method, "POST");
        assert!(op.path_params.iter().any(|p| p == "channel_id"));
        assert!(matches!(op.file_mode, FileMode::Array));
        // Body property + its transitive $defs are inlined.
        let props = op.tool.input_schema.get("properties").unwrap();
        assert!(props.get("body").is_some(), "create_message exposes `body`");
        assert!(op.tool.input_schema.contains_key("$defs"));
    }

    #[test]
    fn single_file_upload_is_a_named_path() {
        let reg = Registry::from_spec_str(SPEC).expect("spec parses");
        let op = reg
            .get("upload_application_attachment")
            .expect("op exists");
        match &op.file_mode {
            FileMode::Named(fields) => assert_eq!(fields, &vec!["file".to_string()]),
            other => panic!("expected Named file mode, got {other:?}"),
        }
    }
}
