use crate::logic::variable::{
    parse_variables, parse_variables_with_defaults, validate_variable_name,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionRequestOptions {
    pub suggested_name: Option<String>,
    pub suggested_tags: Option<String>,
    pub preserve_variables: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionPreview {
    pub source_template_id: String,
    pub source_template_name: String,
    pub source_tags: String,
    pub source_variable_names: Vec<String>,
    pub output_variable_names: Vec<String>,
    pub removed_variables: Vec<String>,
    pub added_variables: Vec<String>,
    pub script_name: String,
    pub tags: String,
    pub content: String,
    pub summary: String,
    pub variable_defaults: HashMap<String, String>,
    pub warnings: Vec<String>,
    pub source_length: usize,
    pub output_length: usize,
}

#[derive(Debug, Deserialize)]
pub struct CompressionResponseEnvelope {
    pub version: String,
    pub result: CompressionResponseResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompressionResponseResult {
    pub script_name: String,
    #[serde(default)]
    pub tags: String,
    pub content: String,
    #[serde(default)]
    pub variable_defaults: HashMap<String, String>,
    #[serde(default)]
    pub summary: String,
}

pub fn build_compression_prompt(
    template_name: &str,
    template_tags: &str,
    ordered_scripts: &[(String, String)],
    composed_content: &str,
    options: &CompressionRequestOptions,
) -> String {
    let source_variables = parse_variables_with_defaults(composed_content);
    let script_section = ordered_scripts
        .iter()
        .enumerate()
        .map(|(index, (name, content))| {
            format!(
                "Script {}\nName: {}\nContent:\n{}",
                index + 1,
                name,
                content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let variable_section = if source_variables.is_empty() {
        "[]".to_string()
    } else {
        serde_json::to_string_pretty(&source_variables).unwrap_or_else(|_| "[]".to_string())
    };

    let suggested_name = options
        .suggested_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("(none)");
    let suggested_tags = options
        .suggested_tags
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(template_tags);

    format!(
        concat!(
            "You compress prompt templates into shorter reusable prompts.\n",
            "Preserve the original task intent.\n",
            "Use double-curly placeholders like {{{{name}}}} or {{{{name:default}}}} when variables are still needed.\n",
            "Do not include markdown fences or any text outside JSON.\n",
            "Return valid JSON with this exact schema:\n",
            "{{\n",
            "  \"version\": \"1\",\n",
            "  \"result\": {{\n",
            "    \"script_name\": \"string\",\n",
            "    \"tags\": \"string\",\n",
            "    \"content\": \"string\",\n",
            "    \"variable_defaults\": {{ \"name\": \"default\" }},\n",
            "    \"summary\": \"string\"\n",
            "  }}\n",
            "}}\n\n",
            "Compression requirements:\n",
            "- Produce one final compressed prompt only.\n",
            "- Keep it reusable and concise.\n",
            "- Do not invent unnecessary variables.\n",
            "- If a variable is removed from content, omit it from variable_defaults.\n",
            "- Prefer the suggested name when it is provided.\n",
            "- preserve_variables = {preserve_variables}.\n\n",
            "Template name: {template_name}\n",
            "Template tags: {template_tags}\n",
            "Suggested script name: {suggested_name}\n",
            "Suggested tags: {suggested_tags}\n\n",
            "Ordered source scripts:\n{script_section}\n\n",
            "Composed prompt:\n{composed_content}\n\n",
            "Parsed source variables:\n{variable_section}\n"
        ),
        preserve_variables = if options.preserve_variables { "true" } else { "false" },
        template_name = template_name,
        template_tags = template_tags,
        suggested_name = suggested_name,
        suggested_tags = suggested_tags,
        script_section = script_section,
        composed_content = composed_content,
        variable_section = variable_section,
    )
}

pub fn parse_compression_response(
    raw_response: &str,
) -> Result<CompressionResponseEnvelope, String> {
    let cleaned = sanitize_model_json_response(raw_response)?;
    serde_json::from_str(&cleaned).map_err(|error| {
        format!(
            "Model response was not valid JSON: {}. Response started with: {}",
            error,
            raw_response.trim().chars().take(240).collect::<String>()
        )
    })
}

fn sanitize_model_json_response(raw_response: &str) -> Result<String, String> {
    let trimmed = raw_response.trim();
    if trimmed.is_empty() {
        return Err("Model response was empty".to_string());
    }

    if let Some(fenced) = extract_fenced_json(trimmed) {
        return Ok(fenced);
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Ok(trimmed.to_string());
    }

    if let Some(json_object) = extract_first_json_object(trimmed) {
        return Ok(json_object);
    }

    Err(format!(
        "Could not locate a JSON object in model response. Response started with: {}",
        trimmed.chars().take(240).collect::<String>()
    ))
}

fn extract_fenced_json(input: &str) -> Option<String> {
    let start = input.find("```")?;
    let rest = &input[start + 3..];
    let rest = rest.strip_prefix("json").unwrap_or(rest);
    let rest = rest.strip_prefix("JSON").unwrap_or(rest);
    let rest = rest.trim_start_matches(['\n', '\r']);
    let end = rest.find("```")?;
    Some(rest[..end].trim().to_string())
}

fn extract_first_json_object(input: &str) -> Option<String> {
    let start = input.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in input[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(input[start..start + offset + ch.len_utf8()].to_string());
                }
            }
            _ => {}
        }
    }

    None
}

pub fn format_provider_error(status: u16, url: &str, body: &str) -> String {
    let provider_hint = match status {
        400 => "Bad request. Check model name and request fields.",
        401 => "Authentication failed. Check the API key.",
        402 => "Insufficient Poe credits or subscription points.",
        403 => "Request was forbidden by the provider.",
        404 => "Endpoint or model was not found. For Poe use base URL https://api.poe.com/v1 and a valid model name like Claude-Sonnet-4.6.",
        408 => "The provider timed out before the model started.",
        413 => "The request was too large for the model context window.",
        429 => "The provider rate limit was hit. Try again shortly.",
        500 => "Provider returned an internal error, which can also happen for invalid upstream requests.",
        502 => "Upstream model backend was unavailable.",
        529 => "Provider is temporarily overloaded.",
        _ => "Provider returned an unexpected error.",
    };

    let compact_body = body.trim();
    if compact_body.is_empty() {
        format!(
            "AI request failed with status {} at {}. {}",
            status, url, provider_hint
        )
    } else {
        format!(
            "AI request failed with status {} at {}. {} Response: {}",
            status, url, provider_hint, compact_body
        )
    }
}

pub fn build_preview_from_response(
    template_id: &str,
    template_name: &str,
    template_tags: &str,
    source_content: &str,
    options: &CompressionRequestOptions,
    parsed: CompressionResponseEnvelope,
) -> Result<CompressionPreview, String> {
    if parsed.version.trim() != "1" {
        return Err("Unsupported compression response version".to_string());
    }

    let script_name = parsed.result.script_name.trim().to_string();
    if script_name.is_empty() {
        return Err("Compressed script name cannot be empty".to_string());
    }

    let content = parsed.result.content.trim().to_string();
    if content.is_empty() {
        return Err("Compressed content cannot be empty".to_string());
    }

    validate_template_placeholders(&content)?;

    let source_variable_names = parse_variables(source_content);
    let output_variable_names = parse_variables(&content);
    let output_variable_set: BTreeSet<_> = output_variable_names.iter().cloned().collect();
    let source_variable_set: BTreeSet<_> = source_variable_names.iter().cloned().collect();

    for key in parsed.result.variable_defaults.keys() {
        if !output_variable_set.contains(key) {
            return Err(format!(
                "variable_defaults contains '{}' but the variable is missing from content",
                key
            ));
        }
    }

    let removed_variables = source_variable_set
        .difference(&output_variable_set)
        .cloned()
        .collect::<Vec<_>>();
    let added_variables = output_variable_set
        .difference(&source_variable_set)
        .cloned()
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    if !removed_variables.is_empty() {
        warnings.push(format!(
            "Removed variables: {}",
            removed_variables.join(", ")
        ));
    }
    if !added_variables.is_empty() {
        warnings.push(format!("Added variables: {}", added_variables.join(", ")));
    }
    if options.preserve_variables && !removed_variables.is_empty() {
        warnings.push(
            "Variable preservation was requested, but some source variables were removed."
                .to_string(),
        );
    }
    if content.len() >= source_content.len() {
        warnings.push("Compressed result is not shorter than the source content.".to_string());
    }

    let tags = parsed.result.tags.trim().to_string();
    let tags = if tags.is_empty() {
        options
            .suggested_tags
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(template_tags)
            .to_string()
    } else {
        tags
    };

    let summary = if parsed.result.summary.trim().is_empty() {
        "Compressed template into a reusable script.".to_string()
    } else {
        parsed.result.summary.trim().to_string()
    };

    Ok(CompressionPreview {
        source_template_id: template_id.to_string(),
        source_template_name: template_name.to_string(),
        source_tags: template_tags.to_string(),
        source_variable_names,
        output_variable_names,
        removed_variables,
        added_variables,
        script_name,
        tags,
        content,
        summary,
        variable_defaults: parsed.result.variable_defaults,
        warnings,
        source_length: source_content.chars().count(),
        output_length: parsed.result.content.chars().count(),
    })
}

fn validate_template_placeholders(content: &str) -> Result<(), String> {
    let variable_pattern = regex::Regex::new(r"\{\{\s*([^}:]+)(?::([^}]+))?\s*\}\}")
        .map_err(|error| format!("Failed to compile variable regex: {}", error))?;

    for captures in variable_pattern.captures_iter(content) {
        let name = captures
            .get(1)
            .map(|m| m.as_str().trim())
            .unwrap_or_default();
        if !validate_variable_name(name) {
            return Err(format!(
                "Invalid variable name '{}' in compressed content",
                name
            ));
        }
    }

    if content.contains("{{") && !variable_pattern.is_match(content) {
        return Err("Compressed content contains malformed variable placeholders".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_compression_response() {
        let raw = r#"{
            "version": "1",
            "result": {
                "script_name": "Compressed",
                "tags": "workflow/review",
                "content": "Review {{code}}",
                "variable_defaults": {"code": "sample"},
                "summary": "done"
            }
        }"#;

        let parsed = parse_compression_response(raw).expect("should parse");
        assert_eq!(parsed.result.script_name, "Compressed");
    }

    #[test]
    fn reject_invalid_json_response() {
        let error = parse_compression_response("not json").unwrap_err();
        assert!(error.contains("valid JSON") || error.contains("Could not locate a JSON object"));
    }

    #[test]
    fn parse_fenced_json_response() {
        let raw = "```json\n{\n  \"version\": \"1\",\n  \"result\": {\n    \"script_name\": \"Compressed\",\n    \"tags\": \"workflow/review\",\n    \"content\": \"Review {{code}}\",\n    \"variable_defaults\": {\"code\": \"sample\"},\n    \"summary\": \"done\"\n  }\n}\n```";
        let parsed = parse_compression_response(raw).expect("should parse fenced JSON");
        assert_eq!(parsed.result.script_name, "Compressed");
    }

    #[test]
    fn parse_json_with_leading_text() {
        let raw = "Here is the JSON you requested:\n{\"version\":\"1\",\"result\":{\"script_name\":\"Compressed\",\"tags\":\"workflow/review\",\"content\":\"Review {{code}}\",\"variable_defaults\":{\"code\":\"sample\"},\"summary\":\"done\"}}";
        let parsed = parse_compression_response(raw).expect("should parse extracted JSON");
        assert_eq!(parsed.result.script_name, "Compressed");
    }

    #[test]
    fn build_preview_reports_variable_diffs() {
        let parsed = CompressionResponseEnvelope {
            version: "1".to_string(),
            result: CompressionResponseResult {
                script_name: "Compressed".to_string(),
                tags: "".to_string(),
                content: "Review {{code}} for {{audience}}".to_string(),
                variable_defaults: HashMap::from([("audience".to_string(), "team".to_string())]),
                summary: "summary".to_string(),
            },
        };

        let preview = build_preview_from_response(
            "template-1",
            "Template",
            "workflow/review",
            "Review {{code}} for {{language}}",
            &CompressionRequestOptions {
                suggested_name: None,
                suggested_tags: None,
                preserve_variables: true,
            },
            parsed,
        )
        .expect("preview should build");

        assert_eq!(preview.removed_variables, vec!["language".to_string()]);
        assert_eq!(preview.added_variables, vec!["audience".to_string()]);
        assert!(!preview.warnings.is_empty());
        assert_eq!(preview.tags, "workflow/review");
    }

    #[test]
    fn reject_missing_variable_default_in_content() {
        let parsed = CompressionResponseEnvelope {
            version: "1".to_string(),
            result: CompressionResponseResult {
                script_name: "Compressed".to_string(),
                tags: "workflow/review".to_string(),
                content: "Review {{code}}".to_string(),
                variable_defaults: HashMap::from([("missing".to_string(), "x".to_string())]),
                summary: "summary".to_string(),
            },
        };

        let error = build_preview_from_response(
            "template-1",
            "Template",
            "workflow/review",
            "Review {{code}}",
            &CompressionRequestOptions {
                suggested_name: None,
                suggested_tags: None,
                preserve_variables: false,
            },
            parsed,
        )
        .unwrap_err();

        assert!(error.contains("variable_defaults"));
    }
}
