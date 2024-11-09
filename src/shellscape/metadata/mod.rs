#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeMetadata {
    pub title: String,
    pub subtitle: String,
    pub server_heading: Option<String>,
    pub observer_heading: Option<String>,
    pub version: String,
    pub author: String,
    pub license: String,
    pub footer: String,
}

impl ShellscapeMetadata {
    pub fn new(
        title: String,
        subtitle: String,
        server_heading: Option<String>,
        observer_heading: Option<String>,
        version: String,
        author: String,
        license: String,
        footer: String,
    ) -> Self {
        Self {
            title,
            subtitle,
            server_heading,
            observer_heading,
            version,
            author,
            license,
            footer,
        }
    }

    pub fn reset_subtitle(&mut self, subtitle: String) {
        self.subtitle = subtitle;
    }

    pub fn reset_server_heading(&mut self, heading: String) {
        self.server_heading = Some(heading);
    }

    pub fn reset_observer_heading(&mut self, heading: String) {
        self.observer_heading = Some(heading);
    }
}
