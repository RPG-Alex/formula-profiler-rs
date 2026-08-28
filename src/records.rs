use std::collections::BTreeMap;

use molecular_formulas::prelude::{MolecularFormula, MolecularFormulaMetadata};

use crate::chemistry::element_counts_in_formula;

#[derive(Debug, Clone)]
pub(crate) struct MoleculeRecord {
    pub(crate) element_counts: BTreeMap<String, usize>,
    pub(crate) monoisotopic_mass_da: f64,
    pub(crate) metadata: BTreeMap<String, String>,
}

impl MoleculeRecord {
    pub(crate) fn from_formula<F>(formula: &F, metadata: BTreeMap<String, String>) -> Self
    where
        F: MolecularFormula,
        u32: From<<F as MolecularFormulaMetadata>::Count>,
    {
        Self {
            element_counts: element_counts_in_formula(formula),
            monoisotopic_mass_da: formula.isotopologue_mass(),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use molecular_formulas::prelude::ChemicalFormula;

    use super::*;

    #[test]
    fn from_formula_derivs_count_and_monoisotopic_mass() {
        let formula = ChemicalFormula::<u32, i32>::from_str("C6H12O6").unwrap();
        let record = MoleculeRecord::from_formula(&formula, BTreeMap::new());
        assert_eq!(record.element_counts.get("C"), Some(&6));
        assert_eq!(record.element_counts.get("H"), Some(&12));
        assert_eq!(record.element_counts.get("O"), Some(&6));

        assert!((180.06..180.07).contains(&record.monoisotopic_mass_da));
    }
}
