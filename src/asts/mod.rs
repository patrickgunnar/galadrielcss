use dashmap::DashMap;
use indexmap::IndexMap;
use lazy_static::lazy_static;

use crate::{types::{Classinator, Stylitron}, utils::generates_node_styles::generates_node_styles};

lazy_static! {
    pub static ref CLASSINATOR: DashMap<String, Classinator> = {
        let map = DashMap::new();

        map.insert("central".to_string(), Classinator::Central(IndexMap::new()));
        map.insert("layouts".to_string(), Classinator::Layouts(IndexMap::new()));
        map.insert("modules".to_string(), Classinator::Modules(IndexMap::new()));

        map
    };

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
