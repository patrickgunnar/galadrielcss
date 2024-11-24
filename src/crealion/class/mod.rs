use chrono::Local;
use futures::future::join_all;
use nenyr::types::class::NenyrStyleClass;
use tokio::task::JoinHandle;
use types::Class;

use crate::{
    error::{ErrorAction, ErrorKind, GaladrielError},
    shellscape::alerts::ShellscapeAlerts,
};

use super::Crealion;

mod patterns;
mod responsive;
mod styles;
mod types;

impl Crealion {
    pub fn process_class(
        &self,
        inherited_contexts: Vec<String>,
        class: NenyrStyleClass,
    ) -> JoinHandle<Vec<ShellscapeAlerts>> {
        tokio::task::spawn(async move {
            let mut alerts = vec![];

            let class_name = class.class_name.clone();
            let is_important = match class.is_important {
                Some(v) => v,
                None => false,
            };

            let mut _my_class = Class::new(&class_name, class.deriving_from);
            let non_responsive_style = class.style_patterns.to_owned();
            let responsive_styles = class.responsive_patterns.to_owned();

            let mut future_result = join_all(vec![
                Self::collect_non_responsive_styles(
                    inherited_contexts.clone(),
                    class_name.clone(),
                    is_important,
                    non_responsive_style,
                ),
                Self::process_responsive_styles(
                    inherited_contexts,
                    class_name,
                    is_important,
                    responsive_styles,
                ),
            ])
            .await;

            match future_result[0].as_mut() {
                Ok((_data, alerts_1)) => {
                    alerts.append(alerts_1);
                    //println!("{:#?}", _data);
                }
                Err(err) => {
                    alerts.push(ShellscapeAlerts::create_galadriel_error(
                        Local::now(),
                        GaladrielError::raise_general_other_error(
                            ErrorKind::Other,
                            &err.to_string(),
                            ErrorAction::Notify,
                        ),
                    ));
                }
            }

            match future_result[1].as_mut() {
                Ok((_data, alerts_2)) => {
                    alerts.append(alerts_2);
                    //println!("{:#?}", _data);
                }
                Err(err) => alerts.push(ShellscapeAlerts::create_galadriel_error(
                    Local::now(),
                    GaladrielError::raise_general_other_error(
                        ErrorKind::Other,
                        &err.to_string(),
                        ErrorAction::Notify,
                    ),
                )),
            }

            alerts
        })
    }
}
