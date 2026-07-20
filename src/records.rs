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

