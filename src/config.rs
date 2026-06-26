use std::path::PathBuf;

use crate::{
    chemistry::normalize_element_symbol,
    error::{Result, SpectraProfilerError},
};

#[derive(Debug, Clone)]
pub enum DatasetSource {
    AnnotatedMs2,
    LocalMgf(PathBuf),
    PubChemSmiles,
    LocalSmilesGz(PathBuf),
}

#[derive(Debug, Clone)]
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
}

impl ProfileConfig {
    pub fn from_args() -> Result<Self> {
        Self::from_iter(std::env::args().skip(1))
    }

    pub fn from_iter<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut args = args.into_iter().map(Into::into);

        let raw_target = args.next().unwrap_or_else(|| "F".to_string());

        let target_selection = if raw_target.eq_ignore_ascii_case("all") {
            TargetSelection::AllObserved
        } else {
            let target_element = normalize_element_symbol(&raw_target).ok_or_else(|| {
                SpectraProfilerError::InvalidElementSymbol { symbol: raw_target.clone() }
            })?;

            TargetSelection::One(target_element)
        };

        let dataset_selector = args.next();

        let (dataset_name, dataset_source) = match dataset_selector.as_deref() {
            Some("pubchem") | Some("pubchem-smiles") => {
                ("pubchem".to_string(), DatasetSource::PubChemSmiles)
            }

            Some("smiles-gz")
            | Some("local-smiles-gz")
            | Some("smi-gz")
            | Some("pubchem-gz")
            | Some("pubchem-smiles-gz") => {
                let path = args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("CID-SMILES.gz"));

                let dataset_name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("local_smiles")
                    .to_string();

                (dataset_name, DatasetSource::LocalSmilesGz(path))
            }

            Some(path) => {
                let path = PathBuf::from(path);

                let dataset_name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("local_mgf")
                    .to_string();

                (dataset_name, DatasetSource::LocalMgf(path))
            }

            None => ("annotated_ms2".to_string(), DatasetSource::AnnotatedMs2),
        };

        Ok(Self {
            dataset_name: dataset_name.clone(),
            dataset_source,
            target_selection,
            cache_dir: PathBuf::from("cache").join(&dataset_name),
            reports_root: PathBuf::from("reports").join(&dataset_name),
        })
    }

    pub fn report_dir_for(&self, target_element: &str) -> PathBuf {
        self.reports_root.join(target_element.to_ascii_lowercase())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_fluorine_and_annotated_ms2() {
        let config = ProfileConfig::from_iter(Vec::<String>::new()).unwrap();

        assert_eq!(config.dataset_name, "annotated_ms2");
        assert_eq!(config.cache_dir, PathBuf::from("cache").join("annotated_ms2"));
        assert_eq!(config.reports_root, PathBuf::from("reports").join("annotated_ms2"));

        match config.target_selection {
            TargetSelection::One(element) => assert_eq!(element, "F"),
            TargetSelection::AllObserved => panic!("expected one target element"),
        }

        match config.dataset_source {
            DatasetSource::AnnotatedMs2 => {}
            DatasetSource::LocalMgf(_)
            | DatasetSource::PubChemSmiles
            | DatasetSource::LocalSmilesGz(_) => {
                panic!("expected annotated_ms2 source")
            }
        }
    }

    #[test]
    fn parses_single_target_element() {
        let config = ProfileConfig::from_iter(["cl"]).unwrap();

        match config.target_selection {
            TargetSelection::One(element) => assert_eq!(element, "Cl"),
            TargetSelection::AllObserved => panic!("expected one target element"),
        }
    }

    #[test]
    fn parses_all_observed_target_selection() {
        let config = ProfileConfig::from_iter(["all"]).unwrap();

        match config.target_selection {
            TargetSelection::AllObserved => {}
            TargetSelection::One(element) => panic!("expected all, got {element}"),
        }
    }

    #[test]
    fn parses_local_mgf_path() {
        let config = ProfileConfig::from_iter(["F", "data/example_file.mgf"]).unwrap();

        assert_eq!(config.dataset_name, "example_file");
        assert_eq!(config.cache_dir, PathBuf::from("cache").join("example_file"));
        assert_eq!(config.reports_root, PathBuf::from("reports").join("example_file"));

        match config.dataset_source {
            DatasetSource::LocalMgf(path) => {
                assert_eq!(path, PathBuf::from("data/example_file.mgf"));
            }
            DatasetSource::AnnotatedMs2
            | DatasetSource::PubChemSmiles
            | DatasetSource::LocalSmilesGz(_) => {
                panic!("expected local MGF source")
            }
        }
    }

    #[test]
    fn parses_downloaded_pubchem_smiles_source() {
        let config = ProfileConfig::from_iter(["all", "pubchem"]).unwrap();

        assert_eq!(config.dataset_name, "pubchem");
        assert_eq!(config.cache_dir, PathBuf::from("cache").join("pubchem"));
        assert_eq!(config.reports_root, PathBuf::from("reports").join("pubchem"));

        match config.target_selection {
            TargetSelection::AllObserved => {}
            TargetSelection::One(element) => panic!("expected all, got {element}"),
        }

        match config.dataset_source {
            DatasetSource::PubChemSmiles => {}
            DatasetSource::AnnotatedMs2
            | DatasetSource::LocalMgf(_)
            | DatasetSource::LocalSmilesGz(_) => {
                panic!("expected downloaded PubChem SMILES source")
            }
        }
    }

    #[test]
    fn parses_downloaded_pubchem_smiles_alias() {
        let config = ProfileConfig::from_iter(["F", "pubchem-smiles"]).unwrap();

        assert_eq!(config.dataset_name, "pubchem");

        match config.target_selection {
            TargetSelection::One(element) => assert_eq!(element, "F"),
            TargetSelection::AllObserved => panic!("expected one target element"),
        }

        match config.dataset_source {
            DatasetSource::PubChemSmiles => {}
            DatasetSource::AnnotatedMs2
            | DatasetSource::LocalMgf(_)
            | DatasetSource::LocalSmilesGz(_) => {
                panic!("expected downloaded PubChem SMILES source")
            }
        }
    }

    #[test]
    fn parses_local_smiles_gz_source() {
        let config = ProfileConfig::from_iter(["all", "smiles-gz", "data/CID-SMILES.gz"]).unwrap();

        assert_eq!(config.dataset_name, "CID-SMILES");
        assert_eq!(config.cache_dir, PathBuf::from("cache").join("CID-SMILES"));
        assert_eq!(config.reports_root, PathBuf::from("reports").join("CID-SMILES"));

        match config.target_selection {
            TargetSelection::AllObserved => {}
            TargetSelection::One(element) => panic!("expected all, got {element}"),
        }

        match config.dataset_source {
            DatasetSource::LocalSmilesGz(path) => {
                assert_eq!(path, PathBuf::from("data/CID-SMILES.gz"));
            }
            DatasetSource::AnnotatedMs2
            | DatasetSource::LocalMgf(_)
            | DatasetSource::PubChemSmiles => {
                panic!("expected local SMILES gzip source")
            }
        }
    }

    #[test]
    fn parses_local_smiles_gz_legacy_pubchem_alias() {
        let config = ProfileConfig::from_iter(["all", "pubchem-gz", "data/CID-SMILES.gz"]).unwrap();

        assert_eq!(config.dataset_name, "CID-SMILES");

        match config.dataset_source {
            DatasetSource::LocalSmilesGz(path) => {
                assert_eq!(path, PathBuf::from("data/CID-SMILES.gz"));
            }
            DatasetSource::AnnotatedMs2
            | DatasetSource::LocalMgf(_)
            | DatasetSource::PubChemSmiles => {
                panic!("expected local SMILES gzip source")
            }
        }
    }

    #[test]
    fn local_smiles_gz_defaults_to_cid_smiles_gz_when_path_is_omitted() {
        let config = ProfileConfig::from_iter(["F", "smiles-gz"]).unwrap();

        assert_eq!(config.dataset_name, "CID-SMILES");

        match config.dataset_source {
            DatasetSource::LocalSmilesGz(path) => {
                assert_eq!(path, PathBuf::from("CID-SMILES.gz"));
            }
            DatasetSource::AnnotatedMs2
            | DatasetSource::LocalMgf(_)
            | DatasetSource::PubChemSmiles => {
                panic!("expected local SMILES gzip source")
            }
        }
    }

    #[test]
    fn builds_report_directory_for_element() {
        let config = ProfileConfig::from_iter(["F"]).unwrap();

        assert_eq!(
            config.report_dir_for("Cl"),
            PathBuf::from("reports").join("annotated_ms2").join("cl")
        );
    }

    #[test]
    fn rejects_invalid_target_element() {
        let error = ProfileConfig::from_iter(["Bl"]).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("invalid element symbol `Bl`"));
    }
}
