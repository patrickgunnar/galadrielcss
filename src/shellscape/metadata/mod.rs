#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeMetadata {
    pub title: String,
    pub subtitle: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub footer: String,
}

#[allow(dead_code)]
impl ShellscapeMetadata {
    pub fn new(
        title: String,
        subtitle: String,
        version: String,
        author: String,
        license: String,
        footer: String,
    ) -> Self {
        Self {
            title,
            subtitle,
            version,
            author,
            license,
            footer,
        }
    }

    pub fn reset_subtitle(&mut self, subtitle: String) {
        self.subtitle = subtitle;
    }
}
