use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{crealion::Crealion, shellscape::alerts::ShellscapeAlerts};

use super::types::UtilityClass;

impl Crealion {
    pub fn process_responsive_styles(
        inherited_contexts: Vec<String>,
        class_name: String,
        is_important: bool,
        responsive_patterns: Option<IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
    ) -> JoinHandle<(Vec<UtilityClass>, Vec<ShellscapeAlerts>)> {
        tokio::spawn(async move {
            let mut alerts: Vec<ShellscapeAlerts> = vec![];
            let mut classes: Vec<UtilityClass> = vec![];

            match responsive_patterns {
                Some(patterns) => {
                    Self::match_style_breakpoint(
                        &inherited_contexts,
                        &class_name,
                        is_important,
                        &mut alerts,
                        &mut classes,
                        patterns,
                    )
                    .await;
                }
                _ => {}
            }

            (classes, alerts)
        })
    }

    async fn match_style_breakpoint(
        inherited_contexts: &Vec<String>,
        class_name: &String,
        is_important: bool,
        alerts: &mut Vec<ShellscapeAlerts>,
        classes: &mut Vec<UtilityClass>,
        responsive_patterns: IndexMap<String, IndexMap<String, IndexMap<String, String>>>,
    ) {
        for (breakpoint, patterns) in responsive_patterns {
            Self::match_style_patterns(
                inherited_contexts,
                Some(breakpoint),
                class_name,
                is_important,
                alerts,
                classes,
                patterns,
            )
            .await;
        }
    }
}
