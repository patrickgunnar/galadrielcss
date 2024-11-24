use indexmap::IndexMap;
use tokio::task::JoinHandle;

use crate::{crealion::Crealion, shellscape::alerts::ShellscapeAlerts};

use super::types::UtilityClass;

impl Crealion {
    pub fn collect_non_responsive_styles(
        inherited_contexts: Vec<String>,
        class_name: String,
        is_important: bool,
        style_patterns: Option<IndexMap<String, IndexMap<String, String>>>,
    ) -> JoinHandle<(Vec<UtilityClass>, Vec<ShellscapeAlerts>)> {
        tokio::spawn(async move {
            let mut alerts: Vec<ShellscapeAlerts> = vec![];
            let mut classes: Vec<UtilityClass> = vec![];

            match style_patterns {
                Some(patterns) => {
                    Self::match_style_patterns(
                        &inherited_contexts,
                        None,
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
}
