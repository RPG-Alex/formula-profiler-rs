use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::{
    chemistry::normalize_element_symbol,
    config::{DataField, DatasetSource, ProfileConfig, TargetSelection},
    error::FormulaProfilerError,
};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    /// Element to profile, such as `F`, `Cl`, or `all` for all elements
    #[arg(long)]
    pub target: String,

    /// Optional maximum number of input records to process
    #[arg(long = "limit")]
    pub record_limit: Option<usize>,

    /// Root directory used for downloaded datasets (default is "cache")
    #[arg(long, default_value = "cache")]
    pub cache_root: PathBuf,

    /// Root directory used for generated reports (default is "reports")
    #[arg(long, default_value = "reports")]
    pub reports_root: PathBuf,

    #[command(subcommand)]
    pub dataset: DatasetCommand,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum DatasetCommand {
    /// Use the annotated MS2 data (will download if needed)
    #[command(name = "annotated")]
    AnnotatedMs2,

    /// Use the PubChem SMILES dataset (will download if needed)
    #[command(name = "pubchem")]
    PubChem,

    /// Process a local MGF file
    LocalMgf { path: PathBuf },

    /// Process a local gzip CID/SMILES file
    SmilesGz { path: PathBuf },

    /// Process a local CSV containing SMILES records
    SmilesCsv {
        path: PathBuf,

        /// Column labels, written in order - `id` and `smiles` are required
        #[arg(long, num_args = 1..)]
        fields: Vec<String>,

        /// Whether the provided CSV first row contains the headers
        #[arg(long)]
        has_headers: bool,
    },
}

impl TryFrom<Cli> for ProfileConfig {
    type Error = FormulaProfilerError;

    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        let Cli { target, record_limit, cache_root, reports_root, dataset } = cli;
        let record_limit: usize = record_limit.unwrap_or(usize::MAX);
        let target_selection = parse_target_selection(&target)?;
        let (dataset_name, dataset_source) = match dataset {
            DatasetCommand::AnnotatedMs2 => {
                ("annotated_ms2".to_string(), DatasetSource::AnnotatedMs2)
            }

            DatasetCommand::PubChem => ("pubchem".to_string(), DatasetSource::PubChemSmiles),

            DatasetCommand::LocalMgf { path } => {
                let dataset_name = dataset_name_from_path(&path, "local_mgf");

                (dataset_name, DatasetSource::LocalMgf(path))
            }

            DatasetCommand::SmilesGz { path } => {
                let dataset_name = dataset_name_from_path(&path, "local_smiles");

                (dataset_name, DatasetSource::LocalSmilesGz(path))
            }

            DatasetCommand::SmilesCsv { path, fields, has_headers } => {
                let dataset_name = dataset_name_from_path(&path, "local_smiles");

                let data_fields = parse_data_fields(fields)?;

                (dataset_name, DatasetSource::LocalSmilesCsv { path, data_fields, has_headers })
            }
        };
        let cache_dir = cache_root.join(&dataset_name);
        let reports_root = reports_root.join(&dataset_name);

        Ok(Self {
            dataset_name,
            dataset_source,
            target_selection,
            record_limit,
            cache_dir,
            reports_root,
        })
    }
}

fn parse_target_selection(raw_target: &str) -> Result<TargetSelection, FormulaProfilerError> {
    if raw_target.eq_ignore_ascii_case("all") {
        return Ok(TargetSelection::AllObserved);
    }
    let target_element = normalize_element_symbol(raw_target).ok_or_else(|| {
        FormulaProfilerError::InvalidElementSymbol { symbol: raw_target.to_string() }
    })?;
    Ok(TargetSelection::One(target_element))
}

fn parse_data_fields(fields: Vec<String>) -> Result<Vec<DataField>, FormulaProfilerError> {
    let data_fields: Vec<DataField> = fields
        .into_iter()
        .map(|field| {
            if field.eq_ignore_ascii_case("id") {
                DataField::Id
            } else if field.eq_ignore_ascii_case("smiles") {
                DataField::Smiles
            } else {
                DataField::Custom(field)
            }
        })
        .collect();

    let id_count = data_fields.iter().filter(|field| matches!(field, DataField::Id)).count();

    let smiles_count =
        data_fields.iter().filter(|field| matches!(field, DataField::Smiles)).count();

    if id_count != 1 || smiles_count != 1 {
        return Err(FormulaProfilerError::InvalidCsvFields { id_count, smiles_count });
    }
    Ok(data_fields)
}

fn dataset_name_from_path(path: &Path, fallback: &str) -> String {
    path.file_stem().and_then(|stem| stem.to_str()).unwrap_or(fallback).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smiles_csv_command() {
        let cli = Cli::try_parse_from([
            "spectra-profiler-rs",
            "--target",
            "all",
            "smiles-csv",
            "data/molecules.csv",
            "--fields",
            "id",
            "smiles",
            "class",
            "--has-headers",
        ])
        .unwrap();
        assert_eq!(cli.target, "all");
        assert_eq!(cli.record_limit, None);
        assert_eq!(cli.cache_root, PathBuf::from("cache"));
        assert_eq!(cli.reports_root, PathBuf::from("reports"));

        assert_eq!(
            cli.dataset,
            DatasetCommand::SmilesCsv {
                path: PathBuf::from("data/molecules.csv"),
                fields: vec!["id".to_owned(), "smiles".to_owned(), "class".to_owned()],
                has_headers: true
            }
        )
    }

    #[test]
    fn converts_smiles_csv_command_into_profile_config() {
        let cli = Cli::try_parse_from([
            "spectra-profiler-rs",
            "--target",
            "cl",
            "--limit",
            "1000",
            "smiles-csv",
            "data/molecules.csv",
            "--fields",
            "id",
            "smiles",
            "class",
            "--has-headers",
        ])
        .unwrap();
        let config = ProfileConfig::try_from(cli).unwrap();
        assert_eq!(config.dataset_name, "molecules");
        assert_eq!(config.record_limit, 1000);
        assert_eq!(config.cache_dir, PathBuf::from("cache").join("molecules"));
        assert_eq!(config.reports_root, PathBuf::from("reports").join("molecules"));

        assert_eq!(config.target_selection, TargetSelection::One("Cl".to_owned()));
        assert_eq!(
            config.dataset_source,
            DatasetSource::LocalSmilesCsv {
                path: PathBuf::from("data/molecules.csv"),
                data_fields: vec![
                    DataField::Id,
                    DataField::Smiles,
                    DataField::Custom("class".to_owned())
                ],
                has_headers: true
            }
        );
    }

    #[test]
    fn reject_csv_without_smiles_field() {
        let cli = Cli::try_parse_from([
            "spectra-profiler-rs",
            "--target",
            "F",
            "smiles-csv",
            "data/molecules.csv",
            "--fields",
            "id",
            "class",
        ])
        .unwrap();
        let error = ProfileConfig::try_from(cli).unwrap_err();
        assert!(matches!(
            error,
            FormulaProfilerError::InvalidCsvFields { id_count: 1, smiles_count: 0 }
        ));
    }

    #[test]
    fn defaults_record_limit_to_usize_max() {
        let cli = Cli::try_parse_from(["spectra-profiler-rs", "--target", "F", "pubchem"]).unwrap();

        let config = ProfileConfig::try_from(cli).unwrap();

        assert_eq!(config.record_limit, usize::MAX);
    }
}
