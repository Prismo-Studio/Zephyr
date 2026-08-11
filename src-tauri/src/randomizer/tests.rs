use std::{collections::HashMap, path::PathBuf};

use super::{
    ap_runner,
    random_value::{self, RandomParse, RandomSkew, RandomSpec},
    types::{Condition, Dependency, GameSchema, RandomizerConfig, Value},
    validation, yaml_gen,
};

/// Build a config for alttp with a single option set.
fn cfg_with(option_id: &str, value: Value) -> RandomizerConfig {
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert(option_id.into(), value);
    RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: Some("Tester".into()),
    }
}

fn alttp_path() -> PathBuf {
    // tests run from src-tauri/, schema lives at ../data/randomizer/schemas/alttp.json
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data/randomizer/schemas/alttp.json")
}

fn load_alttp() -> GameSchema {
    let raw = std::fs::read_to_string(alttp_path()).expect("alttp.json must exist");
    serde_json::from_str(&raw).expect("alttp.json must parse")
}

#[test]
fn alttp_schema_parses() {
    let schema = load_alttp();
    assert_eq!(schema.id, "alttp");
    assert_eq!(schema.name, "A Link to the Past");
    assert!(
        schema.options.len() >= 10,
        "alttp should have at least 10 options, got {}",
        schema.options.len()
    );
}

#[test]
fn yaml_generation_includes_game_section() {
    let schema = load_alttp();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::String("ganon".into()));
    values.insert("crystals_needed_for_ganon".into(), Value::Int(7));

    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: Some("12345".into()),
        values,
        preset_id: None,
        player_name: Some("TestRunner".into()),
    };

    let yaml = yaml_gen::generate(&schema, &cfg).expect("yaml gen ok");

    // Re-parse to confirm it's valid YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml is valid");
    let map = parsed.as_mapping().expect("root is mapping");
    assert_eq!(
        map.get(serde_yaml::Value::from("name"))
            .and_then(|v| v.as_str()),
        Some("TestRunner")
    );
    assert_eq!(
        map.get(serde_yaml::Value::from("seed"))
            .and_then(|v| v.as_str()),
        Some("12345")
    );
    assert_eq!(
        map.get(serde_yaml::Value::from("game"))
            .and_then(|v| v.as_str()),
        Some("A Link to the Past")
    );
    let game_section = map
        .get(serde_yaml::Value::from("A Link to the Past"))
        .and_then(|v| v.as_mapping())
        .expect("game section present");
    assert_eq!(
        game_section
            .get(serde_yaml::Value::from("goal"))
            .and_then(|v| v.as_str()),
        Some("ganon")
    );
    // unset options fall back to defaults
    assert!(game_section.get(serde_yaml::Value::from("mode")).is_some());
}

#[test]
fn validation_flags_out_of_range() {
    let schema = load_alttp();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("crystals_needed_for_ganon".into(), Value::Int(99));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors
        .iter()
        .any(|e| e.option_id == "crystals_needed_for_ganon"));
}

#[test]
fn validation_flags_unknown_option() {
    let schema = load_alttp();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("definitely_not_a_real_option".into(), Value::Bool(true));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors
        .iter()
        .any(|e| e.option_id == "definitely_not_a_real_option"));
}

#[test]
fn validation_flags_invalid_choice() {
    let schema = load_alttp();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::String("not_a_goal".into()));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors.iter().any(|e| e.option_id == "goal"));
}

#[test]
fn validation_flags_stale_dependency() {
    // Build a synthetic schema with an explicit dependency rather than relying
    // on the auto-generated alttp.json (Archipelago option metadata does not
    // expose inter-option dependencies, so they're not in the dumped schemas).
    use crate::randomizer::types::*;
    let schema = GameSchema {
        id: "test".into(),
        name: "Test".into(),
        version: "1".into(),
        description: String::new(),
        options: vec![
            OptionDef {
                id: "mode".into(),
                label: "Mode".into(),
                description: String::new(),
                category: "general".into(),
                option_type: OptionType::Select {
                    choices: vec![
                        Choice {
                            value: "open".into(),
                            label: "Open".into(),
                            description: None,
                        },
                        Choice {
                            value: "inverted".into(),
                            label: "Inverted".into(),
                            description: None,
                        },
                    ],
                    default: "open".into(),
                },
                dependencies: None,
                advanced: false,
            },
            OptionDef {
                id: "entrance_shuffle".into(),
                label: "Entrance Shuffle".into(),
                description: String::new(),
                category: "entrances".into(),
                option_type: OptionType::Toggle { default: false },
                dependencies: Some(vec![Dependency {
                    option_id: "mode".into(),
                    condition: Condition::NotEquals {
                        value: Value::String("inverted".into()),
                    },
                }]),
                advanced: false,
            },
        ],
        presets: vec![],
        meta: GameMeta::default(),
        items: vec![],
    };

    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("mode".into(), Value::String("inverted".into()));
    values.insert("entrance_shuffle".into(), Value::Bool(true));
    let cfg = RandomizerConfig {
        game_id: "test".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors.iter().any(|e| e.option_id == "entrance_shuffle"));
}

#[test]
fn validation_accepts_random_string_for_any_option() {
    let schema = load_alttp();
    // crystals_ganon is a Range, goal is a Select
    for variant in ["random", "random-low", "random-middle", "random-high"] {
        let mut values: HashMap<String, Value> = HashMap::new();
        values.insert(
            "crystals_needed_for_ganon".into(),
            Value::String(variant.into()),
        );
        values.insert("goal".into(), Value::String("random".into()));
        let cfg = RandomizerConfig {
            game_id: "alttp".into(),
            seed: None,
            values,
            preset_id: None,
            player_name: None,
        };
        let errors = validation::validate(&schema, &cfg);
        let relevant: Vec<_> = errors
            .iter()
            .filter(|e| e.option_id == "crystals_needed_for_ganon" || e.option_id == "goal")
            .collect();
        assert!(
            relevant.is_empty(),
            "variant={variant} should be accepted, got {relevant:?}"
        );
    }
}

#[test]
fn validation_accepts_weighted_dict_for_select() {
    let schema = load_alttp();
    // goal is a select; weighted dict between two valid choices.
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("ganon".into(), Value::Int(50));
    weighted.insert("pedestal".into(), Value::Int(50));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::Map(weighted));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(
        errors.iter().all(|e| e.option_id != "goal"),
        "weighted goal should be valid, got {errors:?}"
    );
}

#[test]
fn validation_rejects_invalid_choice_in_weighted_dict() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("not_a_real_goal".into(), Value::Int(50));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::Map(weighted));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors.iter().any(|e| e.option_id == "goal"));
}

#[test]
fn validation_rejects_weighted_dict_with_no_positive_weights() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("ganon".into(), Value::Int(0));
    weighted.insert("pedestal".into(), Value::Int(0));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::Map(weighted));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors.iter().any(|e| e.option_id == "goal"));
}

#[test]
fn validation_rejects_negative_weight() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("ganon".into(), Value::Int(-10));
    weighted.insert("pedestal".into(), Value::Int(50));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::Map(weighted));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors.iter().any(|e| e.option_id == "goal"));
}

#[test]
fn validation_accepts_random_as_weighted_key() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("ganon".into(), Value::Int(50));
    weighted.insert("random".into(), Value::Int(25));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::Map(weighted));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(
        errors.iter().all(|e| e.option_id != "goal"),
        "random key in weighted dict should be allowed, got {errors:?}"
    );
}

#[test]
fn validation_rejects_out_of_range_weighted_key() {
    let schema = load_alttp();
    // crystals_ganon range is [0, 7]; key "99" is out of range.
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("99".into(), Value::Int(50));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("crystals_needed_for_ganon".into(), Value::Map(weighted));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(errors
        .iter()
        .any(|e| e.option_id == "crystals_needed_for_ganon"));
}

#[test]
fn yaml_generation_emits_random_string() {
    let schema = load_alttp();
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::String("random".into()));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: Some("Tester".into()),
    };
    let yaml = yaml_gen::generate(&schema, &cfg).expect("yaml gen ok");
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml parses");
    let game_section = parsed
        .get(serde_yaml::Value::from("A Link to the Past"))
        .and_then(|v| v.as_mapping())
        .expect("game section");
    let goal = game_section
        .get(serde_yaml::Value::from("goal"))
        .expect("goal key present");
    assert_eq!(goal.as_str(), Some("random"));
}

#[test]
fn yaml_generation_emits_toggle_weighted_dict() {
    // swordless is a Toggle in alttp; weighted random between true/false.
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("true".into(), Value::Int(50));
    weighted.insert("false".into(), Value::Int(50));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("swordless".into(), Value::Map(weighted));

    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: Some("Tester".into()),
    };
    let errors = validation::validate(&schema, &cfg);
    assert!(
        errors.iter().all(|e| e.option_id != "swordless"),
        "weighted toggle should validate, got {errors:?}"
    );
    let yaml = yaml_gen::generate(&schema, &cfg).expect("yaml gen ok");
    // Re-parse to confirm valid YAML and the structure round-trips.
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml parses");
    let game_section = parsed
        .get(serde_yaml::Value::from("A Link to the Past"))
        .and_then(|v| v.as_mapping())
        .expect("game section");
    let weighted_out = game_section
        .get(serde_yaml::Value::from("swordless"))
        .and_then(|v| v.as_mapping())
        .expect("swordless is a mapping");
    // Keys may parse as either strings or bools depending on YAML quoting,
    // but Archipelago accepts both — what matters is that both branches are present.
    assert_eq!(weighted_out.len(), 2, "two weighted entries expected");
}

#[test]
fn yaml_generation_emits_weighted_dict() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("ganon".into(), Value::Int(50));
    weighted.insert("pedestal".into(), Value::Int(25));
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("goal".into(), Value::Map(weighted));

    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: Some("Tester".into()),
    };
    let yaml = yaml_gen::generate(&schema, &cfg).expect("yaml gen ok");
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml parses");
    let game_section = parsed
        .get(serde_yaml::Value::from("A Link to the Past"))
        .and_then(|v| v.as_mapping())
        .expect("game section");
    let goal = game_section
        .get(serde_yaml::Value::from("goal"))
        .and_then(|v| v.as_mapping())
        .expect("goal is a mapping (weighted dict)");
    assert_eq!(
        goal.get(serde_yaml::Value::from("ganon"))
            .and_then(|v| v.as_i64()),
        Some(50)
    );
    assert_eq!(
        goal.get(serde_yaml::Value::from("pedestal"))
            .and_then(|v| v.as_i64()),
        Some(25)
    );
}

// --- random-range-<min>-<max> -----------------------------------------------

#[test]
fn parses_every_random_keyword_archipelago_accepts() {
    for simple in ["random", "random-low", "random-middle", "random-high"] {
        assert_eq!(
            random_value::parse(simple),
            RandomParse::Ok(RandomSpec::Simple),
            "{simple} should parse as a plain random keyword"
        );
    }

    let cases = [
        ("random-range-40-60", RandomSkew::Even),
        ("random-range-low-40-60", RandomSkew::Low),
        ("random-range-middle-40-60", RandomSkew::Middle),
        ("random-range-high-40-60", RandomSkew::High),
    ];
    for (text, skew) in cases {
        assert_eq!(
            random_value::parse(text),
            RandomParse::Ok(RandomSpec::Range {
                skew,
                min: 40,
                max: 60
            }),
            "{text} should parse as a ranged random keyword"
        );
    }

    // Archipelago lowercases before dispatching, so casing must not matter.
    assert_eq!(
        random_value::parse("Random-Range-HIGH-1-7"),
        RandomParse::Ok(RandomSpec::Range {
            skew: RandomSkew::High,
            min: 1,
            max: 7
        })
    );
}

#[test]
fn parse_rejects_malformed_random_ranges_but_ignores_other_strings() {
    for bad in [
        "random-range-",
        "random-range-40",
        "random-range-low-40",
        "random-range-40-sixty",
        "random-range-high-a-b",
    ] {
        assert!(
            matches!(random_value::parse(bad), RandomParse::Malformed(_)),
            "{bad} should be reported as malformed"
        );
    }

    for other in ["ganon", "random_range_1_7", "randomly", "7"] {
        assert_eq!(
            random_value::parse(other),
            RandomParse::NotRandom,
            "{other} is not a random keyword"
        );
    }
}

#[test]
fn validation_accepts_random_range_on_a_range_option() {
    let schema = load_alttp();
    // crystals_needed_for_ganon is a Range with bounds [0, 7].
    for variant in [
        "random-range-1-7",
        "random-range-low-1-7",
        "random-range-middle-0-7",
        "random-range-high-1-7",
        "random-range-3-3",
    ] {
        let cfg = cfg_with("crystals_needed_for_ganon", Value::String(variant.into()));
        let errors = validation::validate(&schema, &cfg);
        assert!(
            errors.is_empty(),
            "variant={variant} should be accepted, got {errors:?}"
        );
    }
}

#[test]
fn validation_rejects_random_range_outside_the_option_bounds() {
    let schema = load_alttp();
    let cfg = cfg_with(
        "crystals_needed_for_ganon",
        Value::String("random-range-1-99".into()),
    );
    let errors = validation::validate(&schema, &cfg);
    let err = errors
        .iter()
        .find(|e| e.option_id == "crystals_needed_for_ganon")
        .expect("out-of-bounds range must be reported");
    assert!(
        err.message.contains("[0, 7]"),
        "message should name the allowed range, got {:?}",
        err.message
    );
}

#[test]
fn validation_rejects_random_range_with_min_above_max() {
    let schema = load_alttp();
    let cfg = cfg_with(
        "crystals_needed_for_ganon",
        Value::String("random-range-7-1".into()),
    );
    let errors = validation::validate(&schema, &cfg);
    let err = errors
        .iter()
        .find(|e| e.option_id == "crystals_needed_for_ganon")
        .expect("reversed bounds must be reported");
    assert!(
        err.message.contains("greater than"),
        "message should explain the ordering, got {:?}",
        err.message
    );
}

#[test]
fn validation_rejects_malformed_random_range() {
    let schema = load_alttp();
    let cfg = cfg_with(
        "crystals_needed_for_ganon",
        Value::String("random-range-1".into()),
    );
    let errors = validation::validate(&schema, &cfg);
    assert!(errors
        .iter()
        .any(|e| e.option_id == "crystals_needed_for_ganon"));
}

#[test]
fn validation_rejects_random_range_on_a_choice_option() {
    // goal is a Select; Archipelago's Choice.from_text only knows "random".
    let schema = load_alttp();
    let cfg = cfg_with("goal", Value::String("random-range-1-7".into()));
    let errors = validation::validate(&schema, &cfg);
    assert!(errors.iter().any(|e| e.option_id == "goal"));
}

#[test]
fn validation_accepts_random_range_as_a_weighted_key() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("7".into(), Value::Int(50));
    weighted.insert("random-range-high-1-7".into(), Value::Int(25));
    let cfg = cfg_with("crystals_needed_for_ganon", Value::Map(weighted));
    let errors = validation::validate(&schema, &cfg);
    assert!(
        errors.is_empty(),
        "ranged random weighted key should be valid, got {errors:?}"
    );
}

#[test]
fn validation_rejects_out_of_bounds_random_range_weighted_key() {
    let schema = load_alttp();
    let mut weighted: HashMap<String, Value> = HashMap::new();
    weighted.insert("random-range-1-99".into(), Value::Int(50));
    let cfg = cfg_with("crystals_needed_for_ganon", Value::Map(weighted));
    let errors = validation::validate(&schema, &cfg);
    assert!(errors
        .iter()
        .any(|e| e.option_id == "crystals_needed_for_ganon"));
}

#[test]
fn yaml_generation_emits_random_range_verbatim() {
    let schema = load_alttp();
    let cfg = cfg_with(
        "crystals_needed_for_ganon",
        Value::String("random-range-low-1-7".into()),
    );
    let yaml = yaml_gen::generate(&schema, &cfg).expect("yaml gen ok");
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml parses");
    let game_section = parsed
        .get(serde_yaml::Value::from("A Link to the Past"))
        .and_then(|v| v.as_mapping())
        .expect("game section");
    assert_eq!(
        game_section
            .get(serde_yaml::Value::from("crystals_needed_for_ganon"))
            .and_then(|v| v.as_str()),
        Some("random-range-low-1-7"),
        "the keyword must survive the YAML round-trip unquoted and unchanged"
    );
    assert!(
        yaml_gen::lint(&yaml, &schema)
            .iter()
            .all(|i| i.level != "error"),
        "a valid random range must lint clean"
    );
}

#[test]
fn lint_flags_a_random_range_outside_the_option_bounds() {
    let schema = load_alttp();
    let yaml = "\
name: Tester
game: A Link to the Past
A Link to the Past:
  crystals_needed_for_ganon: random-range-2-99
";
    let issues = yaml_gen::lint(yaml, &schema);
    assert!(
        issues
            .iter()
            .any(|i| i.level == "error" && i.message.contains("crystals_needed_for_ganon")),
        "lint should reject the out-of-bounds range, got {issues:?}"
    );
}

#[test]
fn lint_flags_a_malformed_random_range_weighted_key() {
    let schema = load_alttp();
    let yaml = "\
name: Tester
game: A Link to the Past
A Link to the Past:
  crystals_needed_for_ganon:
    random-range-oops: 50
";
    let issues = yaml_gen::lint(yaml, &schema);
    assert!(
        issues.iter().any(|i| i.level == "error"),
        "lint should reject the malformed weighted key, got {issues:?}"
    );
}

// --- weighted entries beyond the old three-row limit -------------------------

#[test]
fn weighted_dict_supports_many_entries() {
    let schema = load_alttp();
    // Every value the [0, 7] range allows, plus two random keywords: well past
    // the three rows the editor used to cap at.
    let mut weighted: HashMap<String, Value> = HashMap::new();
    for n in 0..=7 {
        weighted.insert(n.to_string(), Value::Int(10 + n));
    }
    weighted.insert("random-low".into(), Value::Int(5));
    weighted.insert("random-range-3-6".into(), Value::Int(7));

    let cfg = cfg_with("crystals_needed_for_ganon", Value::Map(weighted));
    let errors = validation::validate(&schema, &cfg);
    assert!(
        errors.is_empty(),
        "10 weighted entries are valid: {errors:?}"
    );

    let yaml = yaml_gen::generate(&schema, &cfg).expect("yaml gen ok");
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml parses");
    let entries = parsed
        .get(serde_yaml::Value::from("A Link to the Past"))
        .and_then(|v| v.get(serde_yaml::Value::from("crystals_needed_for_ganon")))
        .and_then(|v| v.as_mapping())
        .expect("weighted dict");

    // Numeric keys may come back quoted or bare depending on how serde_yaml
    // renders them; Archipelago accepts either, so try both spellings.
    let weight_of = |key: &str| -> Option<i64> {
        entries
            .get(serde_yaml::Value::from(key))
            .or_else(|| {
                key.parse::<i64>()
                    .ok()
                    .and_then(|n| entries.get(serde_yaml::Value::from(n)))
            })
            .and_then(|v| v.as_i64())
    };

    assert_eq!(entries.len(), 10, "every entry must reach the YAML");
    for n in 0..=7i64 {
        assert_eq!(
            weight_of(&n.to_string()),
            Some(10 + n),
            "weight for {n} must round-trip"
        );
    }
    assert_eq!(weight_of("random-range-3-6"), Some(7));
    assert_eq!(weight_of("random-low"), Some(5));
    assert!(
        yaml_gen::lint(&yaml, &schema)
            .iter()
            .all(|i| i.level != "error"),
        "a ten-entry weighted dict must lint clean"
    );
}

#[test]
fn condition_equals_and_not_equals_work() {
    let dep_eq = Dependency {
        option_id: "mode".into(),
        condition: Condition::Equals {
            value: Value::String("open".into()),
        },
    };
    let dep_neq = Dependency {
        option_id: "mode".into(),
        condition: Condition::NotEquals {
            value: Value::String("inverted".into()),
        },
    };
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("mode".into(), Value::String("open".into()));
    let cfg = RandomizerConfig {
        game_id: "alttp".into(),
        seed: None,
        values,
        preset_id: None,
        player_name: None,
    };
    assert!(validation::dependencies_satisfied(
        std::slice::from_ref(&dep_eq),
        &cfg
    ));
    assert!(validation::dependencies_satisfied(
        std::slice::from_ref(&dep_neq),
        &cfg
    ));
}

#[test]
fn update_host_yaml_port_preserves_structure() {
    let tmp = std::env::temp_dir().join(format!(
        "zephyr-host-yaml-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let yaml = "\
general_options:
  output_path: \"output\"
server_options:
  host: null
  port: 38281
  password: null
  loglevel: \"info\"
";
    std::fs::write(tmp.join("host.yaml"), yaml).unwrap();

    ap_runner::update_host_yaml_port(&tmp, 54321).unwrap();
    let updated = std::fs::read_to_string(tmp.join("host.yaml")).unwrap();

    assert!(updated.contains("port: 54321"));
    assert!(!updated.contains("port: 38281"));
    // Other keys preserved
    assert!(updated.contains("host: null"));
    assert!(updated.contains("password: null"));
    assert!(updated.contains("general_options:"));

    // Now with a trailing comment. Should be preserved
    let yaml2 = "\
server_options:
  port: 38281  # some comment
  host: null
";
    std::fs::write(tmp.join("host.yaml"), yaml2).unwrap();
    ap_runner::update_host_yaml_port(&tmp, 443).unwrap();
    let updated2 = std::fs::read_to_string(tmp.join("host.yaml")).unwrap();
    assert!(updated2.contains("port: 443"));
    assert!(updated2.contains("# some comment"));

    // Missing host.yaml should be a no-op
    let empty = std::env::temp_dir().join(format!(
        "zephyr-host-yaml-empty-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&empty).unwrap();
    assert!(ap_runner::update_host_yaml_port(&empty, 1234).is_ok());

    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::remove_dir_all(&empty);
}
