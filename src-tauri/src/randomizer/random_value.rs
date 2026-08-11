//! Parsing and validation for Archipelago's `random...` option keywords.
//!
//! Archipelago accepts these strings wherever an option value (or a
//! weighted-dict key) is expected:
//!
//! - `random`, `random-low`, `random-middle`, `random-high`
//! - `random-range-<min>-<max>`
//! - `random-range-low-<min>-<max>`, `random-range-middle-<min>-<max>`,
//!   `random-range-high-<min>-<max>`
//!
//! Sources: the "Random numbers" section of
//! <https://archipelago.gg/tutorial/Archipelago/advanced_settings/en> and
//! `Range.from_text` / `Range.custom_range` in Archipelago's `Options.py`.
//!
//! Note on strictness: `custom_range` sorts the two bounds, so upstream
//! silently accepts `random-range-7-1`. Zephyr rejects it instead — writing the
//! bounds backwards is always a mistake in a GUI, and a clear message beats a
//! silently reinterpreted value.

use super::types::{OptionDef, OptionType};

/// The four un-parameterised keywords. Archipelago resolves them against the
/// option's own declared bounds.
pub const SIMPLE_RANDOM_VARIANTS: &[&str] =
    &["random", "random-low", "random-middle", "random-high"];

const RANGE_PREFIX: &str = "random-range-";

/// Which end of the range Archipelago biases the roll towards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomSkew {
    /// `random-range-<min>-<max>`: uniform.
    Even,
    Low,
    Middle,
    High,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RandomSpec {
    /// One of [`SIMPLE_RANDOM_VARIANTS`].
    Simple,
    Range {
        skew: RandomSkew,
        min: i64,
        max: i64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum RandomParse {
    /// Not a `random` keyword at all; the caller should type-check it normally.
    NotRandom,
    Ok(RandomSpec),
    /// Looks like a `random-range-...` keyword but the bounds don't parse.
    Malformed(String),
}

/// Parse an option value / weighted key into a [`RandomSpec`].
///
/// Matching is case-insensitive because Archipelago lowercases the text before
/// dispatching in `Range.from_text`.
pub fn parse(text: &str) -> RandomParse {
    let lower = text.trim().to_ascii_lowercase();

    if SIMPLE_RANDOM_VARIANTS.contains(&lower.as_str()) {
        return RandomParse::Ok(RandomSpec::Simple);
    }

    let Some(rest) = lower.strip_prefix(RANGE_PREFIX) else {
        return RandomParse::NotRandom;
    };

    let (skew, bounds) = if let Some(b) = rest.strip_prefix("low-") {
        (RandomSkew::Low, b)
    } else if let Some(b) = rest.strip_prefix("middle-") {
        (RandomSkew::Middle, b)
    } else if let Some(b) = rest.strip_prefix("high-") {
        (RandomSkew::High, b)
    } else {
        (RandomSkew::Even, rest)
    };

    let Some((min_raw, max_raw)) = bounds.split_once('-') else {
        return RandomParse::Malformed(malformed_message(text));
    };

    match (min_raw.parse::<i64>(), max_raw.parse::<i64>()) {
        (Ok(min), Ok(max)) => RandomParse::Ok(RandomSpec::Range { skew, min, max }),
        _ => RandomParse::Malformed(malformed_message(text)),
    }
}

fn malformed_message(text: &str) -> String {
    format!(
        "'{text}' is not a valid random range; expected \
         random-range-<min>-<max> or random-range-low|middle|high-<min>-<max> \
         with whole, non-negative numbers (e.g. random-range-40-60)"
    )
}

/// Check parsed bounds against the option's declared `[opt_min, opt_max]`.
pub fn check_bounds(text: &str, min: i64, max: i64, opt_min: i64, opt_max: i64) -> Option<String> {
    if min > max {
        return Some(format!(
            "'{text}' has a minimum ({min}) greater than its maximum ({max})"
        ));
    }
    if min < opt_min || max > opt_max {
        return Some(format!(
            "'{text}' is outside the allowed range [{opt_min}, {opt_max}]"
        ));
    }
    None
}

/// Outcome of checking one string against an option definition.
#[derive(Clone, Debug, PartialEq)]
pub enum RandomCheck {
    NotRandom,
    Valid,
    Invalid(String),
}

/// Validate a string option value or weighted-dict key that may be one of
/// Archipelago's `random` keywords.
///
/// The four simple keywords are accepted for every option type, matching what
/// Zephyr has always done. The parameterised `random-range-...` forms only make
/// sense where there is an ordering to sample from, so they are rejected on
/// toggles and choices (Archipelago's `Toggle.from_text` / `Choice.from_text`
/// reject them too). Multi-selects and free text declare no bounds, so those
/// only get the syntax check.
pub fn check_for_option(opt: &OptionDef, text: &str) -> RandomCheck {
    match parse(text) {
        RandomParse::NotRandom => RandomCheck::NotRandom,
        RandomParse::Malformed(msg) => RandomCheck::Invalid(msg),
        RandomParse::Ok(RandomSpec::Simple) => RandomCheck::Valid,
        RandomParse::Ok(RandomSpec::Range { min, max, .. }) => match &opt.option_type {
            OptionType::Range {
                min: opt_min,
                max: opt_max,
                ..
            } => match check_bounds(text, min, max, *opt_min, *opt_max) {
                Some(msg) => RandomCheck::Invalid(msg),
                None => RandomCheck::Valid,
            },
            OptionType::MultiSelect { .. } | OptionType::Text { .. } => {
                if min > max {
                    RandomCheck::Invalid(format!(
                        "'{text}' has a minimum ({min}) greater than its maximum ({max})"
                    ))
                } else {
                    RandomCheck::Valid
                }
            }
            OptionType::Toggle { .. } | OptionType::Select { .. } => {
                RandomCheck::Invalid(format!("'{text}' is only valid on numeric range options"))
            }
        },
    }
}
