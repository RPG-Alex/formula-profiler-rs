use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use flate2::read::GzDecoder;
use mascot_rs::prelude::*;
use molecular_formulas::prelude::ChemicalFormula;
use smiles_parser::{
    DatasetFetchOptions, PUBCHEM_SMILES, SmilesDatasetRecordSource, smiles::Smiles,
};

use crate::{
    chemistry::element_counts_in_formula,
    config::DatasetSource,
    error::{Result, SpectraProfilerError},
    metadata::{metadata_value, optional_debug_label},
    records::MoleculeRecord,
};

pub async fn process_dataset<F>(
    dataset_name: &str,
    source: &DatasetSource,
    cache_dir: &Path,
    on_record: F,
) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    match source {
        DatasetSource::AnnotatedMs2 => process_annotated_ms2(cache_dir, on_record).await,
        DatasetSource::LocalMgf(path) => process_local_mgf(path, on_record),
        DatasetSource::PubChemSmiles => process_pubchem_smiles(cache_dir, on_record),
        DatasetSource::LocalSmilesGz(path) => process_smiles_gz(dataset_name, path, on_record),
    }
}

async fn process_annotated_ms2<F>(cache_dir: &Path, mut on_record: F) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let loaded = MGFVec::<f64>::annotated_ms2()
        .target_directory(cache_dir)
        .verbose()
        .load()
        .await
        .map_err(|source| SpectraProfilerError::DatasetLoad { source: source.into() })?;

    println!("Skipped {} malformed records", loaded.skipped_records());
    println!("Dataset path: {}", loaded.path().display());

    for (index, record) in loaded.into_spectra().into_iter().enumerate() {
        if let Some(mol_record) = extract_mgf_record(index, &record) {
            on_record(mol_record)?;
        }
    }
    Ok(())
}

fn process_local_mgf<F>(path: &Path, mut on_record: F) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let spectra = MGFVec::<f64>::from_path(path)
        .map_err(|source| SpectraProfilerError::DatasetLoad { source: source.into() })?;

    for (index, record) in spectra.into_iter().enumerate() {
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

fn process_smiles_gz<F>(dataset_name: &str, path: &Path, mut on_record: F) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let limit = record_limit();
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);

    let mut skipped = 0usize;
    let mut processed = 0usize;

    for line in reader.lines().take(limit.unwrap_or(usize::MAX)) {
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

fn process_pubchem_smiles<F>(cache_dir: &Path, mut on_record: F) -> Result<()>
where
    F: FnMut(MoleculeRecord) -> Result<()>,
{
    let limit = record_limit();
    let options = DatasetFetchOptions {
        cache_dir: Some(cache_dir.to_path_buf()),
        ..DatasetFetchOptions::default()
    };

    let pubchem_records = PUBCHEM_SMILES
        .iter_records_with_options(&options)
        .map_err(|source| SpectraProfilerError::DatasetLoad { source: source.into() })?;

    let mut skipped = 0usize;
    let mut processed = 0usize;

    for record in pubchem_records.take(limit.unwrap_or(usize::MAX)) {
        let record =
            record.map_err(|source| SpectraProfilerError::DatasetLoad { source: source.into() })?;

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

    println!("Processed {processed} PubChem records");
    println!("Skipped {skipped} PubChem records");
    Ok(())
}

fn record_limit() -> Option<usize> {
    std::env::var("PROFILE_LIMIT")
        .or_else(|_| std::env::var("PUBCHEM_LIMIT"))
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
}
