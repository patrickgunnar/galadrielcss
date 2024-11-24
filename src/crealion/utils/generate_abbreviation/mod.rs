fn is_vowel(c: char) -> bool {
    match c.to_ascii_lowercase() {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false,
    }
}

pub fn generate_abbreviation(input: &str) -> String {
    let result: String = input
        .chars()
        .filter(|c| c.is_ascii_alphabetic() && !is_vowel(*c) || *c == '-')
        .collect();

    let abbr: Vec<String> = result
        .split(&['-', ' '])
        .map(|w| {
            if w.len() > 2 {
                let half = w.len().saturating_div(2) as usize;

                let first = match w.chars().next() {
                    Some(c) => c.to_string(),
                    None => "".to_string(),
                };

                let middle = match w.chars().nth(half) {
                    Some(c) => c.to_string(),
                    None => "".to_string(),
                };

                let last = match w.chars().last() {
                    Some(c) => c.to_string(),
                    None => "".to_string(),
                };

                format!("{}{}{}", first, middle, last)
            } else {
                w.to_string()
            }
        })
        .collect();

    abbr.join("-")
}
