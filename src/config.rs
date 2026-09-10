use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum DataField {
    Id,
    Smiles,
    Custom(String),
}

impl Display for DataField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataField::Id => write!(f, "id"),
            DataField::Smiles => write!(f, "smiles"),
            DataField::Custom(val) => write!(f, "{val}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum DatasetSource {
    AnnotatedMs2,
    LocalMgf(PathBuf),
    Lotus,
    PubChemSmiles,
    Smiles { path: PathBuf, data_fields: Vec<DataField>, has_headers: bool, dataset_name: String },
}

impl DatasetSource {
    pub(crate) fn record_unit(&self) -> &str {
        match self {
            Self::AnnotatedMs2 | Self::LocalMgf(_) => return "spectra",
            Self::PubChemSmiles | Self::Lotus | Self::Smiles { .. } => return "molecules",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TargetSelection {
    One(String),
    AllObserved,
}

#[derive(Debug, Clone)]
pub(crate) struct ProfileConfig {
    pub(crate) dataset_name: String,
    pub(crate) dataset_source: DatasetSource,
    pub(crate) target_selection: TargetSelection,
    pub(crate) cache_dir: PathBuf,
    pub(crate) reports_root: PathBuf,
    pub(crate) record_limit: usize,
}

impl ProfileConfig {
    pub(crate) fn report_dir_for(&self, target_element: &str) -> PathBuf {
        self.reports_root.join(target_element.to_ascii_lowercase())
    }
}
