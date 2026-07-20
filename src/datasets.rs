use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use mascot_rs::prelude::*;
use molecular_formulas::prelude::ChemicalFormula;
use smiles_parser::{
    DatasetFetchOptions, PUBCHEM_SMILES, SmilesDatasetRecordSource, smiles::Smiles,
};

use crate::{
    chemistry::element_counts_in_formula,
    config::{DataField, DatasetSource},
    error::{FormulaProfilerError, Result},
    metadata::{metadata_value, optional_debug_label},
    records::MoleculeRecord,
};

pub async fn process_dataset<F>(
    dataset_name: &str,
    source: &DatasetSource,
    cache_dir: &Path,
    record_limit: usize,
    on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    match source {
        DatasetSource::AnnotatedMs2 => {
            process_annotated_ms2(cache_dir, record_limit, on_record).await
        }

        DatasetSource::LocalMgf(path) => process_local_mgf(path, record_limit, on_record),

        DatasetSource::PubChemSmiles => process_pubchem_smiles(cache_dir, record_limit, on_record),

        DatasetSource::LocalSmilesGz(path) => {
            process_smiles_gz(dataset_name, record_limit, path, on_record)
        }

        DatasetSource::LocalSmilesCsv { path, data_fields, has_headers } => {
            process_smiles_csv(
                dataset_name,
                path,
                data_fields,
                on_record,
                record_limit,
                *has_headers,
            )
        }
        DatasetSource::Smiles { path, data_fields, has_headers, delimiter, dataset_name } => {
            process_smiles_file(
                path,
                data_fields,
                has_headers.to_owned(),
                record_limit,
                delimiter.to_owned(),
                dataset_name,
                on_record,
            )
        }
    }
}

fn process_smiles_file<F>(
    path: &Path,
    data_fields: &[DataField],
    has_headers: bool,
    record_limit: usize,
    delimiter: u8,
    dataset_name: &str,
    mut on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let reader = open_smiles_reader(path)?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .delimiter(delimiter)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    let mut skipped = 0usize;
    let mut processed = 0usize;

    for result in csv_reader.records().take(record_limit) {
        let record = result?;
        if record.len() != data_fields.len() {
            skipped += 1;
            continue;
        }

        let mut id = None;
        let mut smiles_text = None;
        let mut metadata = BTreeMap::new();
        metadata.insert("Source dataset".to_string(), dataset_name.to_string());

        for (data_field, value) in data_fields.iter().zip(record.iter()) {
            match data_field {
                DataField::Id => {
                    id = Some(value);
                }
                DataField::Smiles => {
                    smiles_text = Some(value);
                }
                DataField::Custom(name) => {
                    metadata.insert(name.clone(), value.to_string());
                }
            }
        }

        let Some(id) = id.filter(|value| !value.is_empty()) else {
            skipped += 1;
            continue;
        };
        let Some(smiles_text) = smiles_text.filter(|value| !value.is_empty()) else {
            skipped += 1;
            continue;
        };

        let Some(molecule_record) =
            molecule_record_from_smiles(id.to_string(), smiles_text, metadata)
        else {
            skipped += 1;
            continue;
        };
        on_record(molecule_record)?;
        processed += 1;
    }

    println!("Processed {processed} SMILES records");
    println!("Skipped {skipped} SMILES records");

    Ok(())
}

fn molecule_record_from_smiles(
    id: String,
    smiles_text: &str,
    metadata: BTreeMap<String, String>,
) -> Option<MoleculeRecord> {
    let smiles = smiles_text.parse::<Smiles>().ok()?;
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&smiles);

    Some(MoleculeRecord {
        id,
        element_counts: element_counts_in_formula(&formula),
        metadata,
        peak_count: None,
    })
}

fn open_smiles_reader(path: &Path) -> Result<Box<dyn Read>> {
    let file = File::open(path)?;

    if path.extension() == Some(OsStr::new("gz")) {
        Ok(Box::new(GzDecoder::new(file)))
    } else {
        Ok(Box::new(file))
    }
}

fn process_smiles_csv<F>(
    dataset_name: &str,
    path: &Path,
    data_fields: &[DataField],
    mut on_record: F,
    record_limit: usize,
    has_headers: bool,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut csv_reader =
        csv::ReaderBuilder::new().has_headers(has_headers).trim(csv::Trim::All).from_reader(reader);

    let mut skipped = 0usize;
    let mut processed = 0usize;

    for line in csv_reader.records().take(record_limit) {
        let record = line?;
        // first check that the record is the same length as the data fields
        if record.len() != data_fields.len() {
            skipped += 1;
            continue;
        }
        let mut id = None;
        let mut smiles = None;
        let mut metadata = BTreeMap::new();
        metadata.insert("Source dataset".to_string(), dataset_name.to_string());
        for (data_field, value) in data_fields.iter().zip(record.iter()) {
            match data_field {
                DataField::Id => {
                    id = Some(value);
                }
                DataField::Smiles => {
                    smiles = Some(value);
                }
                DataField::Custom(name) => {
                    metadata.insert(name.clone(), value.to_string());
                }
            }
        }

        let Some(id) = id.filter(|id| !id.is_empty()) else {
            skipped += 1;
            continue;
        };
        let Some(smiles) = smiles.filter(|smiles_text| !smiles_text.is_empty()) else {
            skipped += 1;
            continue;
        };
        let Ok(smiles_record) = smiles.parse::<Smiles>() else {
            skipped += 1;
            continue;
        };
        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&smiles_record);
        on_record(MoleculeRecord {
            id: id.to_string(),
            element_counts: element_counts_in_formula(&formula),
            metadata,
            peak_count: None,
        })?;
        processed += 1;
    }

    println!("Processed {processed} SMILES records");
    println!("Skipped {skipped} SMILES records");

    Ok(())
}

async fn process_annotated_ms2<F>(
    cache_dir: &Path,
    record_limit: usize,
    mut on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let loaded = MGFVec::<f64>::annotated_ms2()
        .target_directory(cache_dir)
        .verbose()
        .load()
        .await
        .map_err(|source| FormulaProfilerError::DatasetLoad { source: source.into() })?;

    println!("Skipped {} malformed records", loaded.skipped_records());
    println!("Dataset path: {}", loaded.path().display());

    for (index, record) in loaded.into_spectra().into_iter().enumerate().take(record_limit) {
        if let Some(mol_record) = extract_mgf_record(index, &record) {
            on_record(mol_record)?;
        }
    }
    Ok(())
}

fn process_local_mgf<F>(path: &Path, record_limit: usize, mut on_record: F) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let spectra = MGFVec::<f64>::from_path(path)
        .map_err(|source| FormulaProfilerError::DatasetLoad { source: source.into() })?;

    for (index, record) in spectra.into_iter().enumerate().take(record_limit) {
        if let Some(mol_record) = extract_mgf_record(index, &record) {
            on_record(mol_record)?;
        }
    }
    Ok(())
}

fn extract_mgf_record(index: usize, record: &MascotGenericFormat<f64>) -> Option<MoleculeRecord> {
    let formula = record.metadata().formula()?;
    let metadata = record.metadata();
    let mut groups = BTreeMap::new();

    groups.insert("Source dataset".to_string(), metadata_value(metadata, "SOURCE_DATASET"));
    groups.insert("Organism".to_string(), metadata_value(metadata, "ORGANISM"));
    groups.insert("NPC pathways".to_string(), metadata_value(metadata, "NPC_PATHWAYS"));
    groups.insert("NPC superclasses".to_string(), metadata_value(metadata, "NPC_SUPERCLASSES"));
    groups.insert("NPC classes".to_string(), metadata_value(metadata, "NPC_CLASSES"));
    groups.insert("Library quality".to_string(), metadata_value(metadata, "LIBRARYQUALITY"));
    groups.insert("Ion mode".to_string(), optional_debug_label(record.ion_mode()));
    groups
        .insert("Source instrument".to_string(), optional_debug_label(record.source_instrument()));

    Some(MoleculeRecord {
        id: record
            .feature_id()
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("record-{index}")),
        element_counts: element_counts_in_formula(formula),
        metadata: groups,
        peak_count: Some(record.len()),
    })
}

fn process_smiles_gz<F>(
    dataset_name: &str,
    record_limit: usize,
    path: &Path,
    mut on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let mut skipped = 0usize;
    let mut processed = 0usize;

    for line in reader.lines().take(record_limit) {
        let line = line?;

        let Some((cid, smiles_text)) = line.split_once(char::is_whitespace) else {
            skipped += 1;
            continue;
        };

        let Ok(smiles) = smiles_text.trim().parse::<Smiles>() else {
            skipped += 1;
            continue;
        };

        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&smiles);
        let mut metadata = BTreeMap::new();
        metadata.insert("Source dataset".to_string(), dataset_name.to_string());

        on_record(MoleculeRecord {
            id: cid.to_string(),
            element_counts: element_counts_in_formula(&formula),
            metadata,
            peak_count: None,
        })?;
        processed += 1;
    }

    println!("Processed {processed} local SMILES records");
    println!("Skipped {skipped} local SMILES records");
    Ok(())
}

fn process_pubchem_smiles<F>(cache_dir: &Path, record_limit: usize, mut on_record: F) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let options = DatasetFetchOptions {
        cache_dir: Some(cache_dir.to_path_buf()),
        ..DatasetFetchOptions::default()
    };

    let pubchem_records = PUBCHEM_SMILES
        .iter_records_with_options(&options)
        .map_err(|source| FormulaProfilerError::DatasetLoad { source: source.into() })?;

    let mut skipped = 0usize;
    let mut processed = 0usize;

    let bar = if record_limit == usize::MAX {
        ProgressBar::new_spinner()
    } else {
        ProgressBar::new(record_limit as u64)
    };

    for record in pubchem_records.take(record_limit) {
        bar.inc(1);

        let record =
            record.map_err(|source| FormulaProfilerError::DatasetLoad { source: source.into() })?;

        let Ok(smiles) = record.smiles().parse::<Smiles>() else {
            skipped += 1;
            continue;
        };

        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&smiles);
        let mut metadata = BTreeMap::new();
        metadata.insert("Source dataset".to_string(), "PubChem".to_string());

        on_record(MoleculeRecord {
            id: record.id().to_string(),
            element_counts: element_counts_in_formula(&formula),
            metadata,
            peak_count: None,
        })?;
        processed += 1;
    }

    bar.finish_and_clear();

    println!("Processed {processed} PubChem records");
    println!("Skipped {skipped} PubChem records");
    Ok(())
}
