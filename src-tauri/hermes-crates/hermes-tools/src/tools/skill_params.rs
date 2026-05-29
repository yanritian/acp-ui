//! Skill Parameter Parsing & Validation Module
//!
//! Parses parameter schemas from SKILL.md frontmatter and validates
//! user-provided parameters against those schemas.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Parameter type definition from SKILL.md frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ParamType {
    String {
        #[serde(default)]
        min_length: Option<usize>,
        #[serde(default)]
        max_length: Option<usize>,
        #[serde(default)]
        pattern: Option<String>,
    },
    Number {
        #[serde(default)]
        minimum: Option<f64>,
        #[serde(default)]
        maximum: Option<f64>,
    },
    Integer {
        #[serde(default)]
        minimum: Option<i64>,
        #[serde(default)]
        maximum: Option<i64>,
    },
    Boolean,
    Array {
        #[serde(default)]
        items: Option<Box<ParamType>>,
        #[serde(default)]
        min_items: Option<usize>,
        #[serde(default)]
        max_items: Option<usize>,
    },
    Object {
        #[serde(default)]
        properties: Option<HashMap<String, ParamDef>>,
    },
    #[serde(rename = "enum")]
    Enum {
        values: Vec<Value>,
    },
}

/// Full parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    #[serde(flatten)]
    pub param_type: ParamType,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
}

/// Parsed parameter schema from SKILL.md frontmatter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillParamSchema {
    pub parameters: HashMap<String, ParamDef>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Parse YAML frontmatter from SKILL.md content
pub fn parse_frontmatter(content: &str) -> Option<SkillParamSchema> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_first = &trimmed[3..];
    if let Some(end_pos) = after_first.find("\n---") {
        let yaml_str = &after_first[..end_pos].trim();
        serde_yaml::from_str::<SkillParamSchema>(yaml_str).ok()
    } else {
        None
    }
}

/// Validate parameters against a schema
pub fn validate_params(
    schema: &SkillParamSchema,
    params: &Map<String, Value>,
) -> Result<Map<String, Value>, Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut validated = Map::new();

    // Check required parameters
    for (name, def) in &schema.parameters {
        if def.required && !params.contains_key(name) {
            if let Some(default_val) = &def.default {
                validated.insert(name.clone(), default_val.clone());
            } else {
                errors.push(ValidationError {
                    field: name.clone(),
                    message: "required parameter missing".to_string(),
                });
            }
            continue;
        }

        if let Some(value) = params.get(name) {
            if let Err(e) = validate_value(&def.param_type, value, name) {
                errors.push(e);
            } else {
                validated.insert(name.clone(), value.clone());
            }
        } else if let Some(default_val) = &def.default {
            validated.insert(name.clone(), default_val.clone());
        }
    }

    // Pass through extra parameters not in schema
    for (key, value) in params {
        if !schema.parameters.contains_key(key) {
            validated.insert(key.clone(), value.clone());
        }
    }

    if errors.is_empty() {
        Ok(validated)
    } else {
        Err(errors)
    }
}

/// Validate a single value against its type definition
fn validate_value(param_type: &ParamType, value: &Value, field: &str) -> Result<(), ValidationError> {
    match param_type {
        ParamType::String { min_length, max_length, pattern } => {
            let s = value.as_str().ok_or_else(|| ValidationError {
                field: field.to_string(),
                message: "expected string".to_string(),
            })?;
            if let Some(min) = min_length {
                if s.len() < *min {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("string length {} < minimum {}", s.len(), min),
                    });
                }
            }
            if let Some(max) = max_length {
                if s.len() > *max {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("string length {} > maximum {}", s.len(), max),
                    });
                }
            }
            if let Some(pat) = pattern {
                let re = regex::Regex::new(pat).map_err(|e| ValidationError {
                    field: field.to_string(),
                    message: format!("invalid regex pattern: {}", e),
                })?;
                if !re.is_match(s) {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("value '{}' does not match pattern '{}'", s, pat),
                    });
                }
            }
            Ok(())
        }
        ParamType::Number { minimum, maximum } => {
            let n = value.as_f64().ok_or_else(|| ValidationError {
                field: field.to_string(),
                message: "expected number".to_string(),
            })?;
            if let Some(min) = minimum {
                if n < *min {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("value {} < minimum {}", n, min),
                    });
                }
            }
            if let Some(max) = maximum {
                if n > *max {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("value {} > maximum {}", n, max),
                    });
                }
            }
            Ok(())
        }
        ParamType::Integer { minimum, maximum } => {
            let n = value.as_i64().ok_or_else(|| ValidationError {
                field: field.to_string(),
                message: "expected integer".to_string(),
            })?;
            if let Some(min) = minimum {
                if n < *min {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("value {} < minimum {}", n, min),
                    });
                }
            }
            if let Some(max) = maximum {
                if n > *max {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("value {} > maximum {}", n, max),
                    });
                }
            }
            Ok(())
        }
        ParamType::Boolean => {
            if !value.is_boolean() {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: "expected boolean".to_string(),
                });
            }
            Ok(())
        }
        ParamType::Array { items: _, min_items, max_items } => {
            let arr = value.as_array().ok_or_else(|| ValidationError {
                field: field.to_string(),
                message: "expected array".to_string(),
            })?;
            if let Some(min) = min_items {
                if arr.len() < *min {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("array length {} < minimum {}", arr.len(), min),
                    });
                }
            }
            if let Some(max) = max_items {
                if arr.len() > *max {
                    return Err(ValidationError {
                        field: field.to_string(),
                        message: format!("array length {} > maximum {}", arr.len(), max),
                    });
                }
            }
            Ok(())
        }
        ParamType::Object { properties: _ } => {
            if !value.is_object() {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: "expected object".to_string(),
                });
            }
            Ok(())
        }
        ParamType::Enum { values } => {
            if !values.contains(value) {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: format!("value not in allowed enum values: {:?}", values),
                });
            }
            Ok(())
        }
    }
}

/// Apply defaults to missing parameters
pub fn apply_defaults(
    schema: &SkillParamSchema,
    params: &mut Map<String, Value>,
) {
    for (name, def) in &schema.parameters {
        if !params.contains_key(name) {
            if let Some(default_val) = &def.default {
                params.insert(name.clone(), default_val.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_string_param(required: bool, default: Option<Value>) -> ParamDef {
        ParamDef {
            param_type: ParamType::String {
                min_length: None,
                max_length: None,
                pattern: None,
            },
            description: None,
            required,
            default,
        }
    }

    #[test]
    fn test_validate_required_present() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert("name".into(), make_string_param(true, None));

        let mut params = Map::new();
        params.insert("name".into(), json!("test"));

        let result = validate_params(&schema, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_missing() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert("name".into(), make_string_param(true, None));

        let params = Map::new();
        let result = validate_params(&schema, &params);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn test_validate_with_default() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert(
            "name".into(),
            make_string_param(true, Some(json!("default_val"))),
        );

        let params = Map::new();
        let result = validate_params(&schema, &params);
        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.get("name").unwrap(), &json!("default_val"));
    }

    #[test]
    fn test_validate_number_range() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert(
            "count".into(),
            ParamDef {
                param_type: ParamType::Number {
                    minimum: Some(0.0),
                    maximum: Some(100.0),
                },
                description: None,
                required: true,
                default: None,
            },
        );

        let mut params = Map::new();
        params.insert("count".into(), json!(50));
        assert!(validate_params(&schema, &params).is_ok());

        params.insert("count".into(), json!(150));
        assert!(validate_params(&schema, &params).is_err());

        params.insert("count".into(), json!(-5));
        assert!(validate_params(&schema, &params).is_err());
    }

    #[test]
    fn test_validate_string_length() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert(
            "code".into(),
            ParamDef {
                param_type: ParamType::String {
                    min_length: Some(1),
                    max_length: Some(1000),
                    pattern: None,
                },
                description: None,
                required: true,
                default: None,
            },
        );

        let mut params = Map::new();
        params.insert("code".into(), json!("hello"));
        assert!(validate_params(&schema, &params).is_ok());

        params.insert("code".into(), json!(""));
        assert!(validate_params(&schema, &params).is_err());
    }

    #[test]
    fn test_validate_enum() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert(
            "mode".into(),
            ParamDef {
                param_type: ParamType::Enum {
                    values: vec![json!("fast"), json!("slow"), json!("auto")],
                },
                description: None,
                required: true,
                default: None,
            },
        );

        let mut params = Map::new();
        params.insert("mode".into(), json!("fast"));
        assert!(validate_params(&schema, &params).is_ok());

        params.insert("mode".into(), json!("invalid"));
        assert!(validate_params(&schema, &params).is_err());
    }

    #[test]
    fn test_validate_array() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert(
            "files".into(),
            ParamDef {
                param_type: ParamType::Array {
                    items: None,
                    min_items: Some(1),
                    max_items: Some(10),
                },
                description: None,
                required: true,
                default: None,
            },
        );

        let mut params = Map::new();
        params.insert("files".into(), json!(["a.txt", "b.txt"]));
        assert!(validate_params(&schema, &params).is_ok());

        params.insert("files".into(), json!([]));
        assert!(validate_params(&schema, &params).is_err());
    }

    #[test]
    fn test_apply_defaults() {
        let mut schema = SkillParamSchema::default();
        schema.parameters.insert(
            "timeout".into(),
            ParamDef {
                param_type: ParamType::Integer {
                    minimum: None,
                    maximum: None,
                },
                description: None,
                required: false,
                default: Some(json!(30)),
            },
        );

        let mut params = Map::new();
        apply_defaults(&schema, &mut params);
        assert_eq!(params.get("timeout").unwrap(), &json!(30));
    }

    #[test]
    fn test_extra_params_pass_through() {
        let schema = SkillParamSchema::default();
        let mut params = Map::new();
        params.insert("extra_key".into(), json!("extra_value"));

        let result = validate_params(&schema, &params);
        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.get("extra_key").unwrap(), &json!("extra_value"));
    }
}
