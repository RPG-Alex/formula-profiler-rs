use std::collections::BTreeMap;

use molecular_formulas::prelude::{MolecularFormula, MolecularFormulaMetadata};

/// Canonical chemical element symbols.
pub(crate) const ELEMENT_SYMBOLS: &[&str] = &[
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

/// Returns `true` if `symbol` is a canonical element symbol.
pub(crate) fn is_valid_element_symbol(symbol: &str) -> bool {
    ELEMENT_SYMBOLS.contains(&symbol)
}

/// Normalizes and validates an element symbol.
///
/// # Parameters
/// - `input`: Element symbol passed from CLI input.
pub(crate) fn normalize_element_symbol(input: &str) -> Option<String> {
    let input = input.trim();

    if input.is_empty() {
        return None;
    }

    let mut chars = input.chars();

    let first = chars.next()?.to_ascii_uppercase();
    let rest = chars.as_str().to_ascii_lowercase();
    let symbol = format!("{first}{rest}");

    is_valid_element_symbol(&symbol).then_some(symbol)
}

/// Counts atoms for each valid element symbol in a structured molecular
/// formula.
pub(crate) fn element_counts_in_formula<F>(formula: &F) -> BTreeMap<String, usize>
where
    F: MolecularFormula,
    u32: From<<F as MolecularFormulaMetadata>::Count>,
{
    let Ok(counts) = formula.element_count_vector::<u32>() else {
        return BTreeMap::new();
    };

    counts
        .into_iter()
        .enumerate()
        .filter_map(|(index, count)| {
            if count == 0 {
                return None;
            }

            let symbol = ELEMENT_SYMBOLS.get(index)?;

            Some(((*symbol).to_string(), count as usize))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use molecular_formulas::prelude::ChemicalFormula;

    use super::*;

    fn formula(input: &str) -> ChemicalFormula<u32, i32> {
        ChemicalFormula::from_str(input).unwrap()
    }

    #[test]
    fn counts_atoms_in_formula() {
        let formula = formula("C6H5ClBrNO2");
        let counts = element_counts_in_formula(&formula);

        assert_eq!(counts.get("C"), Some(&6));
        assert_eq!(counts.get("H"), Some(&5));
        assert_eq!(counts.get("Cl"), Some(&1));
        assert_eq!(counts.get("Br"), Some(&1));
        assert_eq!(counts.get("N"), Some(&1));
        assert_eq!(counts.get("O"), Some(&2));
    }
}
