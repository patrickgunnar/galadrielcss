use crate::{asts::STYLITRON, types::Stylitron};

const SCHEMAS: &[&str] = &["mobile-first", "desktop-first"];

#[derive(Clone, PartialEq, Debug)]
pub struct BreakpointProcessor {
    breakpoint: String,
}

impl BreakpointProcessor {
    pub fn new(breakpoint: &str) -> Self {
        Self {
            breakpoint: breakpoint.to_string(),
        }
    }

    pub fn process(&self) -> Option<String> {
        STYLITRON
            .get("breakpoints")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Breakpoints(breakpoints_definitions) => {
                    SCHEMAS.iter().find_map(|schema| {
                        breakpoints_definitions.get(&schema.to_string()).and_then(
                            |schema_breakpoints| {
                                schema_breakpoints.get(&self.breakpoint).and_then(
                                    |breakpoint_entry| {
                                        self.format_breakpoint_value(breakpoint_entry, schema)
                                    },
                                )
                            },
                        )
                    })
                }
                _ => None,
            })
    }

    fn format_breakpoint_value(&self, breakpoint_entry: &str, schema: &str) -> Option<String> {
        match schema {
            "mobile-first" => Some(format!("min-width:{}", breakpoint_entry)),
            "desktop-first" => Some(format!("max-width:{}", breakpoint_entry)),
            _ => None,
        }
    }
}
