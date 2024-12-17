use chrono::Local;
use futures::future::join_all;
use tokio::sync::broadcast;

use crate::{
    asts::CASCADEX,
    crealion::CENTRAL_CONTEXT_NAME,
    error::{ErrorAction, ErrorKind, GaladrielError},
    events::GaladrielAlerts,
};

mod animations;
mod handlers;
mod imports;
mod responsive;
mod styles;
mod themes;
mod typefaces;
mod variables;

#[derive(Clone, Debug)]
pub struct Astroform {
    /// A `String` representing the tab character(s) used for indentation.
    tab: String,
    /// A `String` representing the space character(s) used for spacing in CSS rules.
    space: String,
    /// A `String` representing the newline character(s) used to separate lines in the output.
    newline: String,
    is_minified: bool,
    set_reset_styles: bool,
    palantir_sender: broadcast::Sender<GaladrielAlerts>,
}

impl Astroform {
    /// Constructs a new `Astroform` instance.
    ///
    /// # Arguments
    /// * `is_minified` - A boolean indicating whether the CSS should be minified.
    /// * `set_reset_styles` - A boolean indicating whether reset styles should be included.
    /// * `palantir_sender` - A `broadcast::Sender<GaladrielAlerts>` for sending error notifications.
    ///
    /// # Returns
    /// A new `Astroform` instance with appropriate configurations for minification and reset styles.
    pub fn new(
        is_minified: bool,
        set_reset_styles: bool,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) -> Self {
        // If minification is enabled, omit space, newline, and tab.
        if is_minified {
            Self {
                newline: "".to_string(),
                space: "".to_string(),
                tab: "".to_string(),
                set_reset_styles,
                palantir_sender,
                is_minified,
            }
        // If not minified, use typical whitespace characters for formatting.
        } else {
            Self {
                newline: "\\u000A".to_string(),
                space: " ".to_string(),
                tab: "\\u0009".to_string(),
                set_reset_styles,
                palantir_sender,
                is_minified,
            }
        }
    }

    /// Transforms various sections of styles into CSS rules asynchronously.
    ///
    /// This function performs concurrent tasks to convert imports, typefaces, variables, themes,
    /// animations, and other styles into corresponding CSS rules. After the tasks are completed, it
    /// joins the results and optionally prepends reset styles to the final output.
    ///
    /// # Returns
    /// A `Future` which resolves once the CSS rules have been processed and inserted into the global stylesheet.
    pub async fn transform(&self) {
        let palantir_sender = self.palantir_sender.clone();
        let mut css_rules: Vec<String> = vec![];

        tracing::info!("Starting the transformation of Galadriel CSS stylesheet into CSS rules.");

        // Perform concurrent tasks for various sections of the stylesheet.
        let astroform_tasks = join_all(vec![
            self.transform_imports(),
            self.transform_typefaces(),
            self.transform_variables(),
            self.transform_themes(),
            self.transform_animations(),
            self.transform_styles(),
            self.transform_responsive_styles(),
        ])
        .await;

        // Process each task result, handle errors, and accumulate valid CSS rules.
        for task in astroform_tasks {
            match task {
                Ok(rule) => {
                    css_rules.push(rule);
                }
                Err(err) => {
                    Self::send_palantir_error_notification(
                        ErrorKind::TaskFailure,
                        ErrorAction::Notify,
                        &err.to_string(),
                        palantir_sender.clone(),
                    );
                }
            }
        }

        // Optionally include reset styles at the beginning if the flag is set.
        if self.set_reset_styles {
            css_rules.insert(0, self.get_reset_styles_rules());
        }

        // Insert the generated CSS rules into the global cascading stylesheet.
        CASCADEX.insert("cascading_sheet".to_string(), css_rules.join(&self.newline));

        tracing::info!("CSS transformation completed and applied to the global stylesheet.");
    }

    /// Generates the CSS reset styles.
    ///
    /// This function generates a comprehensive set of reset styles for various HTML elements to ensure
    /// consistency across browsers. The reset styles are injected into the final CSS if the
    /// `set_reset_styles` flag is enabled.
    ///
    /// # Returns
    /// A `String` containing the CSS rules for resetting various HTML elements.
    fn get_reset_styles_rules(&self) -> String {
        tracing::trace!("Generating CSS reset styles.");

        format!(
            "html,{}body,{}div,{}span,{}applet,{}object,{}iframe,{}h1,{}h2,{}h3,{}h4,{}h5,{}h6,{}p,{}blockquote,{}pre,{}a,{}abbr,{}acronym,{}address,{}big,{}cite,{}code,{}del,{}dfn,{}em,{}img,{}ins,{}kbd,{}q,{}s,{}samp,{}small,{}strike,{}strong,{}sub,{}sup,{}tt,{}var,{}b,{}u,{}i,{}center,{}dl,{}dt,{}dd,{}ol,{}ul,{}li,{}fieldset,{}form,{}label,{}legend,{}table,{}caption,{}tbody,{}tfoot,{}thead,{}tr,{}th,{}td,{}article,{}aside,{}canvas,{}details,{}embed,{}figure,{}figcaption,{}footer,{}header,{}hgroup,{}menu,{}nav,{}output,{}ruby,{}section,{}summary,{}time,{}mark,{}audio,{}video{}{{{}{}margin:{}0;{}{}padding:{}0;{}{}border:{}0;{}{}font-size:{}100%;{}{}font:{}inherit;{}{}vertical-align:{}baseline;{}}}{}article,{}aside,{}details,{}figcaption,{}figure,{}footer,{}header,{}hgroup,{}menu,{}nav,{}section{}{{{}{}display:{}block;{}}}{}body{}{{{}{}line-height:{}1;{}}}{}ol,{}ul{}{{{}{}list-style:{}none;{}}}{}blockquote,{}q{}{{{}{}quotes:{}none;{}}}{}blockquote:before,{}blockquote:after,{}q:before,{}q:after{}{{{}{}content:{}'';{}{}content:{}none;{}}}{}table{}{{{}{}border-collapse:{}collapse;{}{}border-spacing:{}0;{}}}",
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.newline,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.newline,
            self.space,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.newline,
            self.space,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.newline,
            self.space,
            self.space,
            self.space,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.newline,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
            self.tab,
            self.space,
            self.newline,
        )
    }

    /// Resolves the name of a context, modifying it if necessary.
    ///
    /// If the context name matches the constant `CENTRAL_CONTEXT_NAME`, it is replaced with the string "Central".
    ///
    /// # Arguments
    /// * `context_name` - The name of the context to be resolved.
    ///
    /// # Returns
    /// A `String` representing the resolved context name.
    fn resolve_context_name(context_name: &str) -> String {
        if context_name == CENTRAL_CONTEXT_NAME {
            return "Central".to_string();
        }

        context_name.to_string()
    }

    /// Sends an error notification to Palantir if something goes wrong during the transformation.
    ///
    /// This function constructs an error notification and sends it to the `palantir_sender` channel.
    ///
    /// # Arguments
    /// * `error_kind` - The type of error that occurred.
    /// * `error_action` - The action to take for the error (e.g., notify the user, restart, etc.).
    /// * `message` - A message describing the error.
    /// * `palantir_sender` - The sender to use for sending the notification.
    fn send_palantir_error_notification(
        error_kind: ErrorKind,
        error_action: ErrorAction,
        message: &str,
        palantir_sender: broadcast::Sender<GaladrielAlerts>,
    ) {
        let error = GaladrielError::raise_general_other_error(error_kind, message, error_action);
        let notification = GaladrielAlerts::create_galadriel_error(Local::now(), error);

        if let Err(err) = palantir_sender.send(notification) {
            tracing::error!("{:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use tokio::sync::broadcast;

    use crate::{
        astroform::Astroform,
        asts::{CASCADEX, STYLITRON},
        types::Stylitron,
    };

    #[tokio::test]
    async fn astroform_succeeds() {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        mock_stylitron();

        let (palantir_sender, _) = broadcast::channel(10);

        Astroform::new(true, false, palantir_sender)
            .transform()
            .await;

        assert_eq!(
            format!("{:?}", *CASCADEX),
            "{\"cascading_sheet\": \"@import url(\\\"https://fonts.googleapis.com/css2?family=Matemasie&display=swap\\\");@font-face{font-family:roseMartin;src:url(\\\"../typefaces/rosemartin.regular.otf\\\")format(\\\"otf\\\")}:root{--gW1yAqTMgoH:#FF6677;}@media(prefers-color-scheme:light){:root{--gNKGUE7AAmy:#FFFFFF;}}@media(prefers-color-scheme:dark){:root{--gNKGUE7AAmy:#000000;}}@keyframes gsZ0H7sD2jUx{0%{background-color:red}50%{background-color:green}100%{background-color:blue}}.\\\\!bgd-rTLF{background:#00FF00!important}@media screen and (min-width:740px){.\\\\!bgd-rTLF{background:#FFF000!important}}\"}".to_string()
        );
    }

    fn mock_stylitron() {
        STYLITRON.insert(
            "imports".to_string(),
            Stylitron::Imports(IndexMap::from([(
                "https://fonts.googleapis.com/css2?family=Matemasie&display=swap".to_string(),
                (),
            )])),
        );

        STYLITRON.insert(
            "typefaces".to_string(),
            Stylitron::Typefaces(IndexMap::from([(
                "roseMartin".to_string(),
                "../typefaces/rosemartin.regular.otf".to_string(),
            )])),
        );

        STYLITRON.insert(
            "variables".to_string(),
            Stylitron::Variables(IndexMap::from([(
                "myAstroformContextName".to_string(),
                IndexMap::from([(
                    "myAstroformVar".to_string(),
                    vec!["--gW1yAqTMgoH".to_string(), "#FF6677".to_string()],
                )]),
            )])),
        );

        STYLITRON.insert(
            "themes".to_string(),
            Stylitron::Themes(IndexMap::from([(
                "myAstroformContextName".to_string(),
                IndexMap::from([
                    (
                        "light".to_string(),
                        IndexMap::from([(
                            "primaryColor".to_string(),
                            vec!["--gNKGUE7AAmy".to_string(), "#FFFFFF".to_string()],
                        )]),
                    ),
                    (
                        "dark".to_string(),
                        IndexMap::from([(
                            "primaryColor".to_string(),
                            vec!["--gNKGUE7AAmy".to_string(), "#000000".to_string()],
                        )]),
                    ),
                ]),
            )])),
        );

        STYLITRON.insert(
            "animations".to_string(),
            Stylitron::Animation(IndexMap::from([(
                "myAstroformContextName".to_string(),
                IndexMap::from([(
                    "giddyRespond".to_string(),
                    IndexMap::from([(
                        "gsZ0H7sD2jUx".to_string(),
                        IndexMap::from([
                            (
                                "0%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "red".to_string(),
                                )]),
                            ),
                            (
                                "50%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "green".to_string(),
                                )]),
                            ),
                            (
                                "100%".to_string(),
                                IndexMap::from([(
                                    "background-color".to_string(),
                                    "blue".to_string(),
                                )]),
                            ),
                        ]),
                    )]),
                )]),
            )])),
        );

        STYLITRON.insert(
            "styles".to_string(),
            Stylitron::Styles(IndexMap::from([(
                "_".to_string(),
                IndexMap::from([(
                    "!important".to_string(),
                    IndexMap::from([(
                        "background".to_string(),
                        IndexMap::from([("\\!bgd-rTLF".to_string(), "#00FF00".to_string())]),
                    )]),
                )]),
            )])),
        );

        STYLITRON.insert(
            "responsive".to_string(),
            Stylitron::ResponsiveStyles(IndexMap::from([(
                "min-width:740px".to_string(),
                IndexMap::from([(
                    "_".to_string(),
                    IndexMap::from([(
                        "!important".to_string(),
                        IndexMap::from([(
                            "background".to_string(),
                            IndexMap::from([("\\!bgd-rTLF".to_string(), "#FFF000".to_string())]),
                        )]),
                    )]),
                )]),
            )])),
        );
    }
}
