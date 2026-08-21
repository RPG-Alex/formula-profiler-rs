use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct MoleculeRecord {
    pub(crate) element_counts: BTreeMap<String, usize>,
    pub(crate) metadata: BTreeMap<String, String>,
}
