use crate::{asts::STYLITRON, types::Stylitron};

#[derive(Clone, PartialEq, Debug)]
pub struct NicknameProcessor {
    inherited_contexts: Vec<String>,
}

impl NicknameProcessor {
    pub fn new(inherited_contexts: Vec<String>) -> Self {
        Self { inherited_contexts }
    }

    pub fn process(&self, nickname: &str) -> Option<String> {
        STYLITRON
            .get("aliases")
            .and_then(|stylitron_data| match &*stylitron_data {
                Stylitron::Aliases(aliases_definitions) => {
                    self.inherited_contexts.iter().find_map(|context_name| {
                        aliases_definitions
                            .get(context_name)
                            .and_then(|context_aliases| {
                                context_aliases
                                    .get(nickname)
                                    .and_then(|alias_entry| Some(alias_entry.to_owned()))
                            })
                    })
                }
                _ => None,
            })
    }
}
