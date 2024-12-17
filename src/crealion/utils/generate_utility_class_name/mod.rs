use crate::crealion::utils::generate_prefix::generate_prefix;

use super::generate_abbreviation::generate_abbreviation;

pub fn generate_utility_class_name(
    breakpoint: &Option<String>,
    is_important: bool,
    pattern: &str,
    property: &str,
    value: &str,
) -> String {
    let abbr_breakpoint = match breakpoint {
        Some(value) => format!("{}\\\\.", generate_abbreviation(value)),
        None => "".to_string(),
    };

    let importance_prefix = match is_important {
        true => "\\\\!",
        false => "",
    };

    let abbr_pattern = match pattern {
        "_" => "".to_string(),
        v => format!("{}\\\\.", generate_abbreviation(v)),
    };

    let abbr_property = generate_abbreviation(property);
    let value_prefix = generate_prefix(value, false, 4);

    format!(
        "{}{}{}{}-{}",
        abbr_breakpoint, importance_prefix, abbr_pattern, abbr_property, value_prefix
    )
}
