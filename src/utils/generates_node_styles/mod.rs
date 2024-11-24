use indexmap::IndexMap;

pub fn generates_node_styles(
) -> IndexMap<String, IndexMap<String, IndexMap<String, IndexMap<String, String>>>> {
    let mut map = IndexMap::new();
    let mut importance = IndexMap::new();

    importance.insert("!importance".to_string(), IndexMap::new());
    importance.insert("_".to_string(), IndexMap::new());

    map.insert("_".to_string(), importance.clone());
    map.insert("::after".to_string(), importance.clone());
    map.insert("::before".to_string(), importance.clone());
    map.insert("::first-line".to_string(), importance.clone());
    map.insert("::first-letter".to_string(), importance.clone());
    map.insert(":hover".to_string(), importance.clone());
    map.insert(":active".to_string(), importance.clone());
    map.insert(":focus".to_string(), importance.clone());
    map.insert(":first-child".to_string(), importance.clone());
    map.insert(":last-child".to_string(), importance.clone());
    map.insert(":first-of-type".to_string(), importance.clone());
    map.insert(":last-of-type".to_string(), importance.clone());
    map.insert(":only-child".to_string(), importance.clone());
    map.insert(":only-of-type".to_string(), importance.clone());
    map.insert(":target".to_string(), importance.clone());
    map.insert(":visited".to_string(), importance.clone());
    map.insert(":checked".to_string(), importance.clone());
    map.insert(":disabled".to_string(), importance.clone());
    map.insert(":enabled".to_string(), importance.clone());
    map.insert(":read-only".to_string(), importance.clone());
    map.insert(":read-write".to_string(), importance.clone());
    map.insert(":placeholder-shown".to_string(), importance.clone());
    map.insert(":valid".to_string(), importance.clone());
    map.insert(":invalid".to_string(), importance.clone());
    map.insert(":required".to_string(), importance.clone());
    map.insert(":optional".to_string(), importance.clone());
    map.insert(":fullscreen".to_string(), importance.clone());
    map.insert(":focus-within".to_string(), importance.clone());
    map.insert(":out-of-range".to_string(), importance.clone());
    map.insert(":root".to_string(), importance.clone());
    map.insert(":empty".to_string(), importance.clone());

    map
}
