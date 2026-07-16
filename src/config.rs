use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq, Clone)]
pub enum DataField {
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
pub enum DatasetSource {
    AnnotatedMs2,
    LocalMgf(PathBuf),
    PubChemSmiles,
    LocalSmilesGz(PathBuf),
    LocalSmilesCsv { path: PathBuf, data_fields: Vec<DataField>, has_headers: bool },
    Smiles{
        path: PathBuf,
        data_fields: Vec<DataField>,
        has_headers: bool,
        delimiter: u8,
        dataset_name: String,
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RecordKind {
    Spectrum,
    Molecule,
}

impl DatasetSource {
    pub(crate) fn record_kind(&self) -> RecordKind {
        match self {
            Self::AnnotatedMs2 | Self::LocalMgf(_) => RecordKind::Spectrum,
            Self::PubChemSmiles | Self::LocalSmilesGz(_) | Self::LocalSmilesCsv { .. } | Self::Smiles{ .. } => {
                RecordKind::Molecule
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TargetSelection {
    One(String),
    AllObserved,
}

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub dataset_name: String,
    pub dataset_source: DatasetSource,
    pub target_selection: TargetSelection,
    pub cache_dir: PathBuf,
    pub reports_root: PathBuf,
    pub record_limit: usize,
}

impl ProfileConfig {
    pub fn report_dir_for(&self, target_element: &str) -> PathBuf {
        self.reports_root.join(target_element.to_ascii_lowercase())
    }
}
