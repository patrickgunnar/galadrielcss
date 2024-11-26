#[cfg(test)]
pub(crate) mod test_helpers {
    use indexmap::IndexMap;

    use crate::{asts::STYLITRON, types::Stylitron};

    pub(crate) fn mock_variable_node() {
        // -> Variables        => context name: relative name: unique name: value.

        let map = IndexMap::from([
            (
                "myGlacialContext".to_string(),
                IndexMap::from([
                    (
                        "primaryColor".to_string(),
                        IndexMap::from([("s7sj3d".to_string(), "#00ff11".to_string())]),
                    ),
                    (
                        "secondaryColor".to_string(),
                        IndexMap::from([("ste62jh".to_string(), "#00ff99".to_string())]),
                    ),
                    (
                        "foregroundColor".to_string(),
                        IndexMap::from([("s83ikk2h".to_string(), "#00aa99".to_string())]),
                    ),
                ]),
            ),
            (
                "galaxyContext".to_string(),
                IndexMap::from([
                    (
                        "primaryColor".to_string(),
                        IndexMap::from([("a73jsi2d4w".to_string(), "#000000".to_string())]),
                    ),
                    (
                        "secondaryColor".to_string(),
                        IndexMap::from([("b3282733w".to_string(), "#aaaaaa".to_string())]),
                    ),
                    (
                        "foregroundColor".to_string(),
                        IndexMap::from([("c83jf93jd9".to_string(), "#ab14ae".to_string())]),
                    ),
                    (
                        "galaxyForegroundColor".to_string(),
                        IndexMap::from([("d8373jd79".to_string(), "#14ae".to_string())]),
                    ),
                ]),
            ),
        ]);

        STYLITRON.insert("variables".to_string(), Stylitron::Variables(map));
    }

    pub(crate) fn mock_themes_node() {
        // -> Themes           => context_name: schema: relative name: unique name: value.
        let map = IndexMap::from([
            (
                "myGlacialContext".to_string(),
                IndexMap::from([
                    (
                        "light".to_string(),
                        IndexMap::from([
                            (
                                "primaryColor".to_string(),
                                IndexMap::from([("s7sj3d".to_string(), "#00ff11".to_string())]),
                            ),
                            (
                                "secondaryColor".to_string(),
                                IndexMap::from([("ste62jh".to_string(), "#00ff99".to_string())]),
                            ),
                            (
                                "foregroundColor".to_string(),
                                IndexMap::from([("s83ikk2h".to_string(), "#00aa99".to_string())]),
                            ),
                        ]),
                    ),
                    (
                        "dark".to_string(),
                        IndexMap::from([
                            (
                                "primaryColor".to_string(),
                                IndexMap::from([("s7sj3d".to_string(), "#ff1100".to_string())]),
                            ),
                            (
                                "secondaryColor".to_string(),
                                IndexMap::from([("ste62jh".to_string(), "#ff9900".to_string())]),
                            ),
                            (
                                "foregroundColor".to_string(),
                                IndexMap::from([("s83ikk2h".to_string(), "#aa9900".to_string())]),
                            ),
                        ]),
                    ),
                ]),
            ),
            (
                "galaxyContext".to_string(),
                IndexMap::from([
                    (
                        "light".to_string(),
                        IndexMap::from([
                            (
                                "primaryColor".to_string(),
                                IndexMap::from([("a73jsi2d4w".to_string(), "#000000".to_string())]),
                            ),
                            (
                                "secondaryColor".to_string(),
                                IndexMap::from([("b3282733w".to_string(), "#aaaaaa".to_string())]),
                            ),
                            (
                                "foregroundColor".to_string(),
                                IndexMap::from([("c83jf93jd9".to_string(), "#ab14ae".to_string())]),
                            ),
                            (
                                "galaxyForegroundColor".to_string(),
                                IndexMap::from([("d8373jd79".to_string(), "#14ae".to_string())]),
                            ),
                            (
                                "galaxyForegroundColorThemed".to_string(),
                                IndexMap::from([("d8373jd79".to_string(), "#14ae".to_string())]),
                            ),
                        ]),
                    ),
                    (
                        "dark".to_string(),
                        IndexMap::from([
                            (
                                "primaryColor".to_string(),
                                IndexMap::from([("a73jsi2d4w".to_string(), "#dddddd".to_string())]),
                            ),
                            (
                                "secondaryColor".to_string(),
                                IndexMap::from([("b3282733w".to_string(), "#ffffff".to_string())]),
                            ),
                            (
                                "foregroundColor".to_string(),
                                IndexMap::from([("c83jf93jd9".to_string(), "#000000".to_string())]),
                            ),
                            (
                                "galaxyForegroundColor".to_string(),
                                IndexMap::from([("d8373jd79".to_string(), "#111111".to_string())]),
                            ),
                            (
                                "galaxyForegroundColorThemed".to_string(),
                                IndexMap::from([("d8373jd79".to_string(), "#111111".to_string())]),
                            ),
                        ]),
                    ),
                ]),
            ),
        ]);

        STYLITRON.insert("themes".to_string(), Stylitron::Themes(map));
    }

    pub(crate) fn mock_animations_node() {
        // -> Animations       => context name: relative name: unique name: stops: property: value.

        let map = IndexMap::from([
            (
                "myGlacialContext".to_string(),
                IndexMap::from([
                    (
                        "myPrimaryAnimation".to_string(),
                        IndexMap::from([("xsj3544sxa58w".to_string(), IndexMap::new())]),
                    ),
                    (
                        "mySecondaryAnimation".to_string(),
                        IndexMap::from([("ch8725sdw2cs5w".to_string(), IndexMap::new())]),
                    ),
                ]),
            ),
            (
                "galaxyContext".to_string(),
                IndexMap::from([
                    (
                        "myPrimaryAnimation".to_string(),
                        IndexMap::from([("j5sd45ses6bss".to_string(), IndexMap::new())]),
                    ),
                    (
                        "mySecondaryAnimation".to_string(),
                        IndexMap::from([("mds5fiu6w5xsa".to_string(), IndexMap::new())]),
                    ),
                    (
                        "myUniqueAnimation".to_string(),
                        IndexMap::from([("d72jd5fkw54k5w".to_string(), IndexMap::new())]),
                    ),
                ]),
            ),
        ]);

        STYLITRON.insert("animations".to_string(), Stylitron::Animation(map));
    }

    pub(crate) fn mock_breakpoints_node() {
        // -> Breakpoints      => schema: breakpoint name: breakpoint value.

        let map = IndexMap::from([
            (
                "mobile-first".to_string(),
                IndexMap::from([
                    ("myMob01".to_string(), "min-width:430px".to_string()),
                    ("myMob02".to_string(), "min-width:720px".to_string()),
                    ("myMob03".to_string(), "min-width:1024px".to_string()),
                ]),
            ),
            (
                "desktop-first".to_string(),
                IndexMap::from([
                    ("myDesk01".to_string(), "max-width:430px".to_string()),
                    ("myDesk02".to_string(), "max-width:720px".to_string()),
                    ("myDesk03".to_string(), "max-width:1024px".to_string()),
                ]),
            ),
        ]);

        STYLITRON.insert("breakpoints".to_string(), Stylitron::Breakpoints(map));
    }

    pub(crate) fn mock_aliases_node() {
        // -> Aliases          => context name: nickname: value.

        let map = IndexMap::from([
            (
                "myGlacialContext".to_string(),
                IndexMap::from([
                    ("bgdColor".to_string(), "background-color".to_string()),
                    ("pdg".to_string(), "padding".to_string()),
                    ("dsp".to_string(), "display".to_string()),
                ]),
            ),
            (
                "galaxyContext".to_string(),
                IndexMap::from([
                    ("bgdColor".to_string(), "background-color".to_string()),
                    ("pdg".to_string(), "padding".to_string()),
                    ("dsp".to_string(), "display".to_string()),
                    ("br".to_string(), "border-radius".to_string()),
                ]),
            ),
        ]);

        STYLITRON.insert("aliases".to_string(), Stylitron::Aliases(map));
    }
}
