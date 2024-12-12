use crate::custom_regex::CustomRegex;
use regex::Regex;

pub enum EngineChoice {
    Builtin,
    Custom,
    Dfa,
    Hybrid,
    Onepass,
    Boundedbacktracker,
    Pikevm,
    Meta,
    Custommeta, // New engine
}

pub fn parse_engine_choice(engine_str: &str) -> EngineChoice {
    match engine_str {
        "builtin" => EngineChoice::Builtin,
        "custom" => EngineChoice::Custom,
        "dfa" => EngineChoice::Dfa,
        "hybrid" => EngineChoice::Hybrid,
        "onepass" => EngineChoice::Onepass,
        "boundedbacktracker" => EngineChoice::Boundedbacktracker,
        "pikevm" => EngineChoice::Pikevm,
        "meta" => EngineChoice::Meta,
        "custommeta" => EngineChoice::Custommeta,
        _ => EngineChoice::Builtin,
    }
}

pub fn apply_pattern(pattern: &str, text: &str, engine_choice: &EngineChoice) -> String {
    match engine_choice {
        EngineChoice::Builtin => apply_pattern_builtin(pattern, text),
        EngineChoice::Custom => apply_pattern_custom(pattern, text),
        EngineChoice::Dfa => apply_pattern_builtin(pattern, text),
        EngineChoice::Hybrid => "Hybrid (placeholder)".to_string(),
        EngineChoice::Meta => "Meta (placeholder)".to_string(),
        EngineChoice::Onepass => "One-pass (placeholder)".to_string(),
        EngineChoice::Boundedbacktracker => "Bounded backtracking (placeholder)".to_string(),
        EngineChoice::Pikevm => "Pikevm (placeholder)".to_string(),
        EngineChoice::Custommeta => apply_pattern_custommeta(pattern, text),
    }
}

fn apply_pattern_builtin(pattern: &str, text: &str) -> String {
    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return format!("Invalid pattern: {}", e),
    };
    let mut all_matches = Vec::new();
    for mat in regex.find_iter(text) {
        all_matches.push(mat.as_str().to_string());
    }
    if all_matches.is_empty() {
        "No matches found.".to_string()
    } else {
        format!("Matches: {:?}", all_matches)
    }
}

fn apply_pattern_custom(pattern: &str, text: &str) -> String {
    let cr = CustomRegex::new(pattern);
    match cr {
        Ok(parser) => {
            let all_matches = parser.find_iter(text);
            if all_matches.is_empty() {
                "No matches found.".to_string()
            } else {
                format!("Matches: {:?}", all_matches)
            }
        }
        Err(e) => format!("Invalid pattern: {}", e),
    }
}

fn apply_pattern_custommeta(pattern: &str, text: &str) -> String {
    let cr = CustomRegex::new(pattern);
    match cr {
        Ok(parser) => {
            let custom_matches = parser.find_iter(text);
            if custom_matches.is_empty() {
                eprintln!("customMeta: CustomRegex no matches, verify builtin.");
                let builtin_result = apply_pattern_builtin(pattern, text);
                if builtin_result.contains("No matches found")
                    || builtin_result.contains("Invalid pattern")
                {
                    eprintln!("customMeta: Builtin also failed.");
                    return builtin_result;
                } else {
                    // Return custom (no matches found) anyway:
                    return "No matches found.".to_string();
                }
            } else {
                let builtin_result = apply_pattern_builtin(pattern, text);
                if builtin_result.contains("Invalid pattern") {
                    eprintln!("customMeta: fallback to builtin invalid pattern.");
                    return builtin_result;
                }

                if builtin_result.contains("No matches found") {
                    eprintln!("customMeta: mismatch between custom and builtin. Using builtin.");
                    return builtin_result;
                }

                // Both found matches:
                format!("Matches: {:?}", custom_matches)
            }
        }
        Err(e) => {
            eprintln!("customMeta: error: {}. fallback builtin.", e);
            apply_pattern_builtin(pattern, text)
        }
    }
}
