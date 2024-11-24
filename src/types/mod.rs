use indexmap::IndexMap;

#[derive(Clone, PartialEq, Debug)]
pub enum Stylitron {
    Imports(IndexMap<String, ()>),
    Aliases(IndexMap<String, IndexMap<String, String>>),
    Breakpoints(IndexMap<String, IndexMap<String, String>>),
    Typefaces(IndexMap<String, String>),
    Variables(IndexMap<String, IndexMap<String, IndexMap<String, String>>>),
    Themes(IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>>),
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

/*
    -> Aliases          => context name: nickname: value.
    -> Breakpoints      => schema: breakpoint name: breakpoint value.
    -> Variables        => context name: relative name: unique name: value.
    -> Themes           => context_name: schema: relative name: unique name: value.
    -> Animations       => context name: relative name: unique name: stops: property: value.
    -> Styles           => pattern name: importance: property: class name: value.
    -> Responsive       => breakpoint: pattern name: importance: property: class name: value.
*/
