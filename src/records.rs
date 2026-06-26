use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct MoleculeRecord {
    #[allow(dead_code)]
    pub id: String,
    pub element_counts: BTreeMap<String, usize>,
    pub metadata: BTreeMap<String, String>,
    #[allow(dead_code)]
    pub peak_count: Option<usize>,
}

impl MoleculeRecord {
    pub fn atom_count(&self, element: &str) -> usize {
        self.element_counts.get(element).copied().unwrap_or_default()
    }
}
