#[derive(Clone, PartialEq, Debug)]
pub struct UtilityClass {
    breakpoint: Option<String>,
    pattern: String,
    class_name: String,
    is_important: bool,
    property: String,
    value: String,
}

#[allow(dead_code)]
impl UtilityClass {
    pub fn create_class(
        breakpoint: &Option<String>,
        pattern: &str,
        class_name: &str,
        is_important: bool,
        property: &str,
        value: &str,
    ) -> Self {
        Self {
            breakpoint: breakpoint.clone(),
            pattern: pattern.to_string(),
            class_name: class_name.to_string(),
            property: property.to_string(),
            value: value.to_string(),
            is_important,
        }
    }

    pub fn get_breakpoint(&self) -> Option<String> {
        self.breakpoint.clone()
    }

    pub fn get_class_name(&self) -> String {
        self.class_name.clone()
    }

    pub fn is_important(&self) -> bool {
        self.is_important
    }

    pub fn get_property(&self) -> String {
        self.property.clone()
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ResponsiveClass {
    breakpoint: String,
    styles: Vec<UtilityClass>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Class {
    class_name: String,
    deriving_from: Option<String>,
    utility_classes: Vec<String>,
    styles: Vec<UtilityClass>,
    responsive_styles: Vec<ResponsiveClass>,
}

#[allow(dead_code)]
impl Class {
    pub fn new(class_name: &str, deriving_from: Option<String>) -> Self {
        Self {
            class_name: class_name.to_string(),
            utility_classes: vec![],
            responsive_styles: vec![],
            styles: vec![],
            deriving_from,
        }
    }

    pub fn push_utility_class(&mut self, class_name: String) {
        self.utility_classes.push(class_name);
    }

    pub fn get_class_name(&self) -> String {
        self.class_name.clone()
    }

    pub fn get_deriving_from(&self) -> Option<String> {
        self.deriving_from.clone()
    }

    pub fn get_utility_classes(&self) -> Vec<String> {
        self.utility_classes.clone()
    }

    pub fn get_styles(&self) -> Vec<UtilityClass> {
        self.styles.clone()
    }

    pub fn get_responsive_styles(&self) -> Vec<ResponsiveClass> {
        self.responsive_styles.clone()
    }
}
