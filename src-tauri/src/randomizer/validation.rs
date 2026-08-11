use std::collections::HashMap;
use std::collections::HashSet;

use super::random_value::{self, RandomCheck};
use super::types::{
    Condition, Dependency, GameSchema, OptionDef, OptionType, RandomizerConfig, ValidationError,
    Value,
};
use super::yaml_gen::EXTRA_GAME_SECTION_KEYS;

fn matches_condition(actual: &Value, condition: &Condition) -> bool {
    match condition {
        Condition::Equals { value } => actual == value,
        Condition::NotEquals { value } => actual != value,
        Condition::In { values } => values.iter().any(|v| v == actual),
    }
}

/// Returns true if all of the option's dependencies are satisfied by the current config.
pub fn dependencies_satisfied(deps: &[Dependency], config: &RandomizerConfig) -> bool {
    deps.iter().all(|dep| {
        config
            .values
            .get(&dep.option_id)
            .map(|v| matches_condition(v, &dep.condition))
            .unwrap_or(false)
    })
}

fn validate_weighted_key(opt: &OptionDef, key: &str) -> Option<String> {
    // "random" and the random-range-... forms are valid weighted keys wherever
    // the option itself accepts them.
    match random_value::check_for_option(opt, key) {
        RandomCheck::Valid => return None,
        RandomCheck::Invalid(msg) => return Some(msg),
        RandomCheck::NotRandom => {}
    }
    match &opt.option_type {
        OptionType::Toggle { .. } => {
            if matches!(key, "true" | "false" | "on" | "off") {
                None
            } else {
                Some(format!(
                    "'{key}' is not a valid toggle key (use true/false)"
                ))
            }
        }
        OptionType::Range { min, max, .. } => match key.parse::<i64>() {
            Ok(i) if i >= *min && i <= *max => None,
            Ok(i) => Some(format!("weighted key {i} out of range [{min}, {max}]")),
            Err(_) => Some(format!(
                "'{key}' is not a valid range key (expected a number in [{min}, {max}], \
                 'random' or 'random-range-{min}-{max}')"
            )),
        },
        OptionType::Select { choices, .. } => {
            if choices.iter().any(|c| c.value == key) {
                None
            } else {
                Some(format!("'{key}' is not a valid choice"))
            }
        }
        // Weighted dicts on multi_select / text are unusual; allow any key.
        OptionType::MultiSelect { .. } | OptionType::Text { .. } => None,
    }
}

fn validate_weighted_map(opt: &OptionDef, m: &HashMap<String, Value>) -> Option<String> {
    if m.is_empty() {
        return Some("weighted dict must have at least one entry".to_string());
    }
    let mut any_positive = false;
    for (key, weight) in m {
        let w = match weight {
            Value::Int(i) => *i,
            _ => return Some(format!("weight for '{key}' must be an integer")),
        };
        if w < 0 {
            return Some(format!("weight for '{key}' cannot be negative"));
        }
        if w > 0 {
            any_positive = true;
        }
        if let Some(msg) = validate_weighted_key(opt, key) {
            return Some(msg);
        }
    }
    if !any_positive {
        return Some("weighted dict must have at least one positive weight".to_string());
    }
    None
}

fn validate_value(opt: &OptionDef, value: &Value) -> Option<String> {
    // "random" (and its skewed / ranged variants) is accepted by Archipelago
    // in place of a concrete value.
    if let Value::String(s) = value {
        match random_value::check_for_option(opt, s) {
            RandomCheck::Valid => return None,
            RandomCheck::Invalid(msg) => return Some(msg),
            RandomCheck::NotRandom => {}
        }
    }

    // Weighted random dict — accepted for any option type that supports it.
    if let Value::Map(m) = value {
        return validate_weighted_map(opt, m);
    }

    match (&opt.option_type, value) {
        (OptionType::Toggle { .. }, Value::Bool(_)) => None,
        (OptionType::Range { min, max, .. }, Value::Int(i)) => {
            if i < min || i > max {
                Some(format!("value {i} out of range [{min}, {max}]"))
            } else {
                None
            }
        }
        (OptionType::Select { choices, .. }, Value::String(s)) => {
            if choices.iter().any(|c| &c.value == s) {
                None
            } else {
                Some(format!("'{s}' is not a valid choice"))
            }
        }
        (OptionType::MultiSelect { choices, .. }, Value::List(items)) => {
            for item in items {
                if let Value::String(s) = item {
                    if !choices.iter().any(|c| &c.value == s) {
                        return Some(format!("'{s}' is not a valid choice"));
                    }
                } else {
                    return Some("multi-select expects string items".to_string());
                }
            }
            None
        }
        (OptionType::Text { .. }, Value::String(_)) => None,
        _ => Some("value type does not match option type".to_string()),
    }
}

pub fn validate(schema: &GameSchema, config: &RandomizerConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let known_ids: HashSet<&str> = schema.options.iter().map(|o| o.id.as_str()).collect();

    // 1. Unknown keys in config.values
    for key in config.values.keys() {
        if known_ids.contains(key.as_str()) {
            continue;
        }
        if EXTRA_GAME_SECTION_KEYS.contains(&key.as_str()) {
            continue;
        }
        errors.push(ValidationError {
            option_id: key.clone(),
            message: format!("unknown option '{key}' (not in schema)"),
        });
    }

    // 2. Per-option checks (type/range/choices + dep-stale state)
    for opt in &schema.options {
        let dep_ok = opt
            .dependencies
            .as_deref()
            .map(|d| dependencies_satisfied(d, config))
            .unwrap_or(true);

        let value = match config.values.get(&opt.id) {
            Some(v) => v,
            None => continue,
        };

        if !dep_ok {
            errors.push(ValidationError {
                option_id: opt.id.clone(),
                message: "dependency not satisfied, this option should not be set".to_string(),
            });
            continue;
        }

        if let Some(msg) = validate_value(opt, value) {
            errors.push(ValidationError {
                option_id: opt.id.clone(),
                message: msg,
            });
        }
    }

    errors
}
