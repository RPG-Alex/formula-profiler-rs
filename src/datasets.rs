use std::{collections::BTreeMap, ffi::OsStr, fs::File, io::Read, path::Path};

use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use mascot_rs::prelude::*;
use molecular_formulas::prelude::ChemicalFormula;
use smiles_rs::{
    ArchiveMode, DatasetFetchOptions, LOTUS_SMILES, PUBCHEM_SMILES, SmilesDatasetRecordSource,
    datasets::COCONUT_SMILES, smiles::Smiles,
};

use crate::{
    config::{DataField, DatasetSource},
    error::{FormulaProfilerError, Result},
    metadata::{metadata_value, optional_debug_label},
    records::MoleculeRecord,
};

pub(crate) async fn process_dataset<F>(
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

        DatasetSource::Smiles { path, data_fields, has_headers, dataset_name } => {
            process_smiles_file(
                path,
                data_fields,
                has_headers.to_owned(),
                record_limit,
                dataset_name,
                on_record,
            )
        }

        DatasetSource::PubChemSmiles => {
            process_smiles_cache(
                &PUBCHEM_SMILES,
                "PubChem",
                cache_dir,
                ArchiveMode::KeepCompressed,
                record_limit,
                on_record,
            )
        }

        DatasetSource::Lotus => {
            process_smiles_cache(
                &LOTUS_SMILES,
                "Lotus",
                cache_dir,
                ArchiveMode::KeepCompressed,
                record_limit,
                on_record,
            )
        }

        DatasetSource::Coconut => {
            process_smiles_cache(
                &COCONUT_SMILES,
                "Coconut",
                cache_dir,
                ArchiveMode::Decompress,
                record_limit,
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
    dataset_name: &str,
    mut on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let reader = open_smiles_reader(path)?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .delimiter(b',')
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

        let mut smiles_text = None;
        let mut metadata = BTreeMap::new();
        metadata.insert("Source dataset".to_string(), dataset_name.to_string());

        for (data_field, value) in data_fields.iter().zip(record.iter()) {
            match data_field {
                DataField::Id => {}
                DataField::Smiles => {
                    smiles_text = Some(value);
                }
                DataField::Custom(name) => {
                    metadata.insert(name.clone(), value.to_string());
                }
            }
        }

        let Some(smiles_text) = smiles_text.filter(|value| !value.is_empty()) else {
            skipped += 1;
            continue;
        };

        let Some(molecule_record) = molecule_record_from_smiles(smiles_text, metadata) else {
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
    smiles_text: &str,
    metadata: BTreeMap<String, String>,
) -> Option<MoleculeRecord> {
    let smiles = smiles_text.parse::<Smiles>().ok()?;
    let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&smiles);

    Some(MoleculeRecord::from_formula(&formula, metadata))
}

fn open_smiles_reader(path: &Path) -> Result<Box<dyn Read>> {
    let file = File::open(path)?;

    if path.extension() == Some(OsStr::new("gz")) {
        Ok(Box::new(GzDecoder::new(file)))
    } else {
        Ok(Box::new(file))
    }
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

    for record in loaded.into_spectra().into_iter().take(record_limit) {
        if let Some(mol_record) = extract_mgf_record(&record) {
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

    for record in spectra.into_iter().take(record_limit) {
        if let Some(mol_record) = extract_mgf_record(&record) {
            on_record(mol_record)?;
        }
    }
    Ok(())
}

fn extract_mgf_record(record: &MascotGenericFormat<f64>) -> Option<MoleculeRecord> {
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

    Some(MoleculeRecord::from_formula(formula, groups))
}

fn process_smiles_cache<F>(
    dataset: &dyn SmilesDatasetRecordSource,
    dataset_name: &str,
    cache_dir: &Path,
    archive_mode: ArchiveMode,
    record_limit: usize,
    mut on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let options = DatasetFetchOptions {
        cache_dir: Some(cache_dir.to_path_buf()),
        archive_mode,
        ..DatasetFetchOptions::default()
    };

    let records = dataset
        .iter_records_with_options(&options)
        .map_err(|source| FormulaProfilerError::DatasetLoad { source: source.into() })?;

    let mut skipped = 0usize;
    let mut processed = 0usize;

    let bar = if record_limit == usize::MAX {
        ProgressBar::new_spinner()
    } else {
        ProgressBar::new(record_limit as u64)
    };

    for record in records.take(record_limit) {
        bar.inc(1);

        let record =
            record.map_err(|source| FormulaProfilerError::DatasetLoad { source: source.into() })?;

        let Ok(smiles) = record.smiles().parse::<Smiles>() else {
            skipped += 1;
            continue;
        };

        let formula: ChemicalFormula<u32, i32> = ChemicalFormula::from(&smiles);
        let mut metadata = BTreeMap::new();
        metadata.insert("Source dataset".to_string(), dataset_name.to_string());

        on_record(MoleculeRecord::from_formula(&formula, metadata))?;
        processed += 1;
    }

    bar.finish_and_clear();

    println!("Processed {processed} {dataset_name} records");
    println!("Skipped {skipped} {dataset_name}");
    Ok(())
}
