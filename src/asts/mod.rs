use dashmap::DashMap;
use indexmap::IndexMap;
use lazy_static::lazy_static;

use crate::{
    configatron::GaladrielConfig,
    events::GaladrielAlerts,
    types::{Classinator, Clastrack, Stylitron},
    utils::{generates_node_styles::generates_node_styles, generates_words::generates_words},
};

lazy_static! {
    /// A static reference to a thread-safe `DashMap` that holds a bank of words for name generation.
    ///
    /// The `NAMER` map organizes words into two categories:
    /// - **Adjectives**: A list of descriptive words used in name generation.
    /// - **Nouns**: A list of objects or entities used in name generation.
    ///
    /// These words are loaded into the `DashMap` upon initialization via the `generates_words()`
    /// function, which returns tuples containing vectors of adjectives and nouns. The words are
    /// stored with the following keys:
    /// - `"adjectives"`: Maps to a `Vec<String>` containing adjectives.
    /// - `"nouns"`: Maps to a `Vec<String>` containing nouns.
    ///
    /// This setup enables efficient, concurrent access to the word bank for tasks requiring
    /// dynamically generated names.
    pub static ref NAMER: DashMap<String, Vec<String>> = {
        let dash_map = DashMap::new();
        let (adjectives, nouns) = generates_words();

        dash_map.insert("adjectives".to_string(), adjectives);
        dash_map.insert("nouns".to_string(), nouns);

        dash_map
    };

    /// Stores the Galadriel CSS configurations. This `DashMap` contains key-value pairs
    /// where the keys are configuration names as `String` and the values are different
    /// `GaladrielConfig` variants. These configurations are used globally within the
    /// Galadriel CSS framework to customize various behaviors and settings.
    ///
    /// The following configurations are included:
    /// - `exclude`: A list of strings representing excluded path rules or files.
    /// - `autoNaming`: A boolean indicating whether to automatically generate class/context/animation names.
    /// - `resetStyles`: A boolean specifying whether to reset default CSS styles.
    /// - `minifiedStyles`: A boolean indicating whether the generated CSS should be minified.
    /// - `port`: A string representing the port for the server.
    pub static ref CONFIGATRON: DashMap<String, GaladrielConfig> = {
        let map = DashMap::new();

        map.insert("exclude".to_string(), GaladrielConfig::Exclude(vec![]));
        map.insert("autoNaming".to_string(), GaladrielConfig::AutoNaming(false));
        map.insert("resetStyles".to_string(), GaladrielConfig::ResetStyles(true));
        map.insert("minifiedStyles".to_string(), GaladrielConfig::MinifiedStyles(true));
        map.insert("port".to_string(), GaladrielConfig::Port("0".to_string()));

        map
    };

    /// Stores Galadriel CSS alerts. This `DashMap` holds alerts related to Galadriel CSS,
    /// where the key is a string identifier (e.g., "alerts") and the value is a vector
    /// of `GaladrielAlerts` which represent specific alert data.
    ///
    /// Alerts are typically used to notify users of important events or issues
    /// related to the CSS generation and application process.
    pub static ref PALANTIR_ALERTS: DashMap<String, Vec<GaladrielAlerts>> = {
        let map = DashMap::new();

        map.insert("alerts".to_string(), vec![]);

        map
    };

    /// `CASCADEX` is a static variable that stores the generated CSS content from
    /// the Galadriel CSS framework. The map holds a key-value pair where the key is
    /// a string identifier (e.g., "cascading_sheet") and the value is the corresponding
    /// generated CSS as a `String`.
    ///
    /// The generated CSS is stored in this map after all transformations and is ready
    /// to be applied in the final output of the application.
    pub static ref CASCADEX: DashMap<String, String> = {
        let dash_map = DashMap::new();

        dash_map.insert("cascading_sheet".to_string(), String::new());

        dash_map
    };

    /// `CLASTRACK` tracks the association between Nenyr classes and CSS utility
    /// class names. This `DashMap` holds mappings where the key is a string identifier
    /// (e.g., "central", "layouts", "modules") and the value is an `Clastrack` that
    /// contains class names and their associated utility classes in the framework.
    ///
    /// This map is essential for linking Nenyr-defined classes to the actual CSS utility
    /// classes that will be applied to HTML elements.
    pub static ref CLASTRACK: DashMap<String, Clastrack> = {
        let dash_map = DashMap::new();

        dash_map.insert("central".to_string(), Clastrack::Central(IndexMap::new()));
        dash_map.insert("layouts".to_string(), Clastrack::Layouts(IndexMap::new()));
        dash_map.insert("modules".to_string(), Clastrack::Modules(IndexMap::new()));

        dash_map
    };

    /// `INTAKER` stores the names of contexts used in Nenyr classes, with the key
    /// being the path of the context (e.g., "path/context_name") and the value being
    /// the corresponding context name.
    ///
    /// This map helps manage and track the context names applied within the Galadriel CSS framework
    /// for styling purposes.
    pub static ref INTAKER: DashMap<String, String> = DashMap::new();

    /// `GATEKEEPER` tracks relationships between module contexts and the layout contexts
    /// that they receive extension. It is represented as a `DashMap` where the key is a
    /// string representing the layout context and the value is a vector of strings representing
    /// the module contexts that the layout gives extension.
    ///
    /// This helps ensure the correct application of styles based on the layout contexts
    /// and module contexts within the framework.
    pub static ref GATEKEEPER: DashMap<String, Vec<String>> = DashMap::new();

    /// `CLASSINATOR` tracks the mapping between Nenyr classes and their corresponding
    /// CSS utility classes, including inheritance. This `DashMap` contains keys such as
    /// "central", "layouts", and "modules", each holding a `Classinator` enum variant
    /// that includes the respective classes and their mappings.
    ///
    /// The mapping facilitates the creation and inheritance of classes based on their
    /// contextual relationships within the framework.
    pub static ref CLASSINATOR: DashMap<String, Classinator> = {
        let map = DashMap::new();

        map.insert("central".to_string(), Classinator::Central(IndexMap::new()));
        map.insert("layouts".to_string(), Classinator::Layouts(IndexMap::new()));
        map.insert("modules".to_string(), Classinator::Modules(IndexMap::new()));

        map
    };

    /// `STYLITRON` is the main abstract syntax tree (AST) for the generated styles in
    /// Galadriel CSS. This `DashMap` contains various style categories and their
    /// corresponding data, each represented as a `Stylitron` enum variant. The categories
    /// include imports, aliases, breakpoints, typefaces, variables, themes, animations,
    /// styles, and responsive styles.
    ///
    /// Each category contains an `IndexMap` or other appropriate data structure that holds
    /// the specific style information, and the styles are generated and populated into the
    /// `STYLITRON` map during the build or dev processes.
    pub static ref STYLITRON: DashMap<String, Stylitron> = {
        let map = DashMap::new();

        map.insert("imports".to_string(), Stylitron::Imports(IndexMap::new()));
        map.insert("aliases".to_string(), Stylitron::Aliases(IndexMap::new()));
        map.insert("breakpoints".to_string(), Stylitron::Breakpoints(IndexMap::new()));
        map.insert("typefaces".to_string(), Stylitron::Typefaces(IndexMap::new()));
        map.insert("variables".to_string(), Stylitron::Variables(IndexMap::new()));
        map.insert("themes".to_string(), Stylitron::Themes(IndexMap::new()));
        map.insert("animations".to_string(), Stylitron::Animation(IndexMap::new()));
        map.insert("styles".to_string(), Stylitron::Styles(generates_node_styles()));
        map.insert("responsive".to_string(), Stylitron::ResponsiveStyles(IndexMap::new()));

        map
    };
}
