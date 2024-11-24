/// Represents a utility class used for styling.
///
/// A `UtilityClass` encapsulates properties, values, and metadata
/// related to a single utility class, including breakpoints, patterns,
/// and whether the class is marked as `!important`.
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
    /// Creates a new `UtilityClass` instance.
    ///
    /// # Parameters
    /// - `breakpoint`: An optional breakpoint for the class.
    /// - `pattern`: The pattern name.
    /// - `class_name`: The name of the utility class.
    /// - `is_important`: A flag indicating if the class is `!important`.
    /// - `property`: The CSS property.
    /// - `value`: The CSS value.
    ///
    /// # Returns
    /// A new instance of `UtilityClass`.
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

    /// Retrieves the breakpoint associated with the utility class.
    pub fn get_breakpoint(&self) -> Option<String> {
        self.breakpoint.clone()
    }

    /// Retrieves the class name.
    pub fn get_class_name(&self) -> String {
        self.class_name.clone()
    }

    /// Checks if the class is marked as `!important`.
    pub fn is_important(&self) -> bool {
        self.is_important
    }

    /// Retrieves the CSS property.
    pub fn get_property(&self) -> String {
        self.property.clone()
    }

    /// Retrieves the CSS value.
    pub fn get_value(&self) -> String {
        self.value.clone()
    }
}

/// Represents a class that aggregates utility classes.
///
/// A `Class` groups multiple utility classes and provides methods for
/// managing their names and associated styling rules.
#[derive(Clone, PartialEq, Debug)]
pub struct Class {
    /// The name of the Nenyr class.
    class_name: String,
    /// Optional derivation metadata, indicating a base class.
    deriving_from: Option<String>,
    /// List of utility names associated with the Nenyr class.
    utility_names: Vec<String>,
    /// List of utility classes associated with the Nenyr class.
    classes: Vec<UtilityClass>,
}

#[allow(dead_code)]
impl Class {
    /// Creates a new `Class` instance.
    ///
    /// # Parameters
    /// - `class_name`: The name of the Nenyr class.
    /// - `deriving_from`: Optional metadata indicating inheritance.
    ///
    /// # Returns
    /// A new instance of `Class`.
    pub fn new(class_name: &str, deriving_from: Option<String>) -> Self {
        Self {
            class_name: class_name.to_string(),
            utility_names: vec![],
            classes: vec![],
            deriving_from,
        }
    }

    /// Sets the utility names for the class.
    ///
    /// Appends the given utility names to the class's existing list.
    pub fn set_utility_names(&mut self, utility_names: &mut Vec<String>) {
        self.utility_names.append(utility_names);
    }

    /// Sets the utility classes for the class.
    ///
    /// Appends the given utility classes to the class's existing list.
    pub fn set_classes(&mut self, classes: &mut Vec<UtilityClass>) {
        self.classes.append(classes);
    }

    /// Retrieves the class name.
    pub fn get_class_name(&self) -> String {
        self.class_name.clone()
    }

    /// Retrieves the derivation metadata, if any.
    pub fn get_deriving_from(&self) -> Option<String> {
        self.deriving_from.clone()
    }

    /// Retrieves the list of utility names.
    pub fn get_utility_names(&self) -> Vec<String> {
        self.utility_names.clone()
    }

    /// Retrieves the list of utility classes.
    pub fn get_classes(&self) -> Vec<UtilityClass> {
        self.classes.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::crealion::class::types::{Class, UtilityClass};

    #[test]
    fn test_create_class_initialization() {
        let breakpoint = Some("lg".to_string());
        let pattern = "::after";
        let class_name = "lg-af-wd4";
        let property = "width";
        let value = "4rem";
        let is_important = true;

        let utility_class = UtilityClass::create_class(
            &breakpoint,
            pattern,
            class_name,
            is_important,
            property,
            value,
        );

        assert_eq!(utility_class.get_breakpoint(), Some("lg".to_string()));
        assert_eq!(utility_class.get_class_name(), "lg-af-wd4");
        assert!(utility_class.is_important());
        assert_eq!(utility_class.get_property(), "width");
        assert_eq!(utility_class.get_value(), "4rem");
    }

    #[test]
    fn test_class_name_generation_rules() {
        let breakpoint = Some("md".to_string());
        let pattern = ":hover";
        let class_name = "md-hv-clr1";
        let property = "color";
        let value = "red";
        let is_important = false;

        let utility_class = UtilityClass::create_class(
            &breakpoint,
            pattern,
            class_name,
            is_important,
            property,
            value,
        );

        assert_eq!(utility_class.get_class_name(), "md-hv-clr1");
    }

    #[test]
    fn test_class_name_with_important_flag() {
        let class_name = "!md-hv-bg1";
        let utility_class = UtilityClass::create_class(
            &Some("md".to_string()),
            ":hover",
            class_name,
            true,
            "background",
            "blue",
        );

        assert!(utility_class.is_important());
        assert_eq!(utility_class.get_class_name(), "!md-hv-bg1");
    }

    #[test]
    fn test_class_initialization() {
        let class = Class::new("btnPrimary", Some("btn".to_string()));

        assert_eq!(class.get_class_name(), "btnPrimary");
        assert_eq!(class.get_deriving_from(), Some("btn".to_string()));
        assert!(class.get_utility_names().is_empty());
        assert!(class.get_classes().is_empty());
    }

    #[test]
    fn test_set_utility_names() {
        let mut class = Class::new("btnPrimary", None);
        let mut utilities = vec!["text-white".to_string(), "bg-blue".to_string()];
        class.set_utility_names(&mut utilities);

        assert_eq!(
            class.get_utility_names(),
            vec!["text-white".to_string(), "bg-blue".to_string()]
        );
    }

    #[test]
    fn test_set_classes() {
        let mut class = Class::new("btnPrimary", None);
        let utility = UtilityClass::create_class(&None, "_", "text-white", false, "color", "white");
        let mut classes = vec![utility.clone()];
        class.set_classes(&mut classes);

        assert_eq!(class.get_classes(), vec![utility]);
    }
}
