use std::collections::BTreeSet;

pub fn normalize_element_symbol(input: &str) -> Option<String> {
    let input = input.trim();

    if input.is_empty() {
        return None;
    }

    let mut chars = input.chars();

    let first = chars.next()?.to_ascii_uppercase();
    let rest = chars.as_str().to_ascii_lowercase();

    Some(format!("{first}{rest}"))
}

pub fn contains_element(formula: &str, target: &str) -> bool {
    formula_symbols(formula).any(|symbol| symbol == target)
}

pub fn element_symbols_in_formula(formula: &str) -> BTreeSet<String> {
    formula_symbols(formula).collect()
}

fn formula_symbols(formula: &str) -> impl Iterator<Item = String> + '_ {
    let mut chars = formula.chars().peekable();

    std::iter::from_fn(move || {
        while let Some(ch) = chars.next() {
            if !ch.is_ascii_uppercase() {
                continue;
            }

            let mut symbol = String::from(ch);

            if let Some(next) = chars.peek()
                && next.is_ascii_lowercase()
            {
                symbol.push(*next);
                chars.next();
            }

            return Some(symbol);
        }

        None
    })
}
