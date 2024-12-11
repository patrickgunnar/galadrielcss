use indexmap::IndexMap;

use crate::{
    asts::{CLASSINATOR, GATEKEEPER, INTAKER, STYLITRON},
    types::{Classinator, Stylitron},
};

use super::generates_node_styles::generates_node_styles;

pub fn restore_abstract_syntax_trees() {
    INTAKER.clear();
    GATEKEEPER.clear();

    CLASSINATOR.insert("central".to_string(), Classinator::Central(IndexMap::new()));
    CLASSINATOR.insert("layouts".to_string(), Classinator::Layouts(IndexMap::new()));
    CLASSINATOR.insert("modules".to_string(), Classinator::Modules(IndexMap::new()));

    STYLITRON.insert("imports".to_string(), Stylitron::Imports(IndexMap::new()));
    STYLITRON.insert("aliases".to_string(), Stylitron::Aliases(IndexMap::new()));
    STYLITRON.insert("breakpoints".to_string(), Stylitron::Breakpoints(IndexMap::new()));
    STYLITRON.insert("typefaces".to_string(), Stylitron::Typefaces(IndexMap::new()));
    STYLITRON.insert("variables".to_string(), Stylitron::Variables(IndexMap::new()));
    STYLITRON.insert("themes".to_string(), Stylitron::Themes(IndexMap::new()));
    STYLITRON.insert("animations".to_string(), Stylitron::Animation(IndexMap::new()));
    STYLITRON.insert("styles".to_string(), Stylitron::Styles(generates_node_styles()));
    STYLITRON.insert("responsive".to_string(), Stylitron::ResponsiveStyles(IndexMap::new()));
}
