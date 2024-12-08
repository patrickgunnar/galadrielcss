use indexmap::IndexMap;

#[derive(Clone, PartialEq, Debug)]
pub enum Stylitron {
    // -> Aliases          => context name: nickname: value.
    // -> Breakpoints      => schema: breakpoint name: breakpoint value.
    // -> Variables        => context name: relative name: unique name: value.
    // -> Themes           => context_name: schema: relative name: unique name: value.
    // -> Animations       => context name: relative name: unique name: stops: property: value.
    // -> Styles           => pattern name: importance: property: class name: value.
    // -> Responsive       => breakpoint: pattern name: importance: property: class name: value.
    Imports(IndexMap<String, ()>),
    Aliases(IndexMap<String, IndexMap<String, String>>),
    Breakpoints(IndexMap<String, IndexMap<String, String>>),
    Typefaces(IndexMap<String, String>),
    Variables(IndexMap<String, IndexMap<String, Vec<String>>>),
    Themes(IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>),
    Animation(
        IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
        >,
    ),
    Styles(IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>),
    ResponsiveStyles(
        IndexMap<
            String,
            IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>,
        >,
    ),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Classinator {
    // -> Central      => inherits name: class name: value
    // -> Layouts      => layout name: inherits name: class name: value
    // -> Modules      => parent name: module name: inherits name: class name: value
    Central(IndexMap<String, IndexMap<String, Vec<String>>>),
    Layouts(IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>),
    Modules(IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, Vec<String>>>>>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Clastrack {
    Central(IndexMap<String, String>),
    Layouts(IndexMap<String, IndexMap<String, String>>),
    Modules(IndexMap<String, IndexMap<String, String>>),
}
