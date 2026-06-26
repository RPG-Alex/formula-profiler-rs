use std::{fs::File, io::Write};

use super::data::{
    EnrichedGroupSummary, NumericSummary, TOP_ENRICHED_MIN_TOTAL_SUPPORT, WarningSummary,
};
use crate::{config::DatasetSource, error::Result};

pub(super) fn write_numeric_summary(
    file: &mut File,
    summary: &NumericSummary,
    source: &DatasetSource,
) -> Result<()> {
    let unit = match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => "spectra",
        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => "molecules",
    };

    writeln!(file)?;
    writeln!(file, "## Numeric summary")?;
    writeln!(file)?;
    writeln!(file, "| Metric | Value |")?;
    writeln!(file, "|---|---:|")?;
    writeln!(file, "| Total {unit} | {} |", summary.total_spectra)?;
    writeln!(file, "| Positive count | {} |", summary.positive_count)?;
    writeln!(file, "| Negative count | {} |", summary.negative_count)?;
    writeln!(file, "| Positive percentage | {:.4}% |", summary.positive_percentage)?;

    Ok(())
}

pub(super) fn write_atom_count_distribution_section(
    file: &mut File,
    source: &DatasetSource,
    target_element: &str,
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Atom-count distribution")?;
    writeln!(file)?;

    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => {
            writeln!(
                file,
                "This section shows how many formula-bearing spectra have exactly `k` atoms of `{target_element}`."
            )?;
            writeln!(
                file,
                "The `0` row represents formulas that do not contain `{target_element}`."
            )?;
        }

        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => {
            writeln!(
                file,
                "This section shows how many formula-bearing molecules have exactly `k` atoms of `{target_element}`."
            )?;
            writeln!(
                file,
                "The `0` row represents formulas that do not contain `{target_element}`."
            )?;
        }
    }

    writeln!(file)?;
    writeln!(file, "[CSV table](tables/target_atom_count_distribution.csv)")?;
    writeln!(file)?;

    writeln!(
        file,
        "<img src=\"figures/target_atom_count_distribution.svg\" alt=\"{target_element} atom-count distribution\" />"
    )?;

    Ok(())
}

pub(super) fn write_top_enriched_groups(
    file: &mut File,
    groups: &[EnrichedGroupSummary],
    source: &DatasetSource,
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Top enriched groups")?;
    writeln!(file)?;

    writeln!(
        file,
        "This table compares **metadata groups** across all population-map tables. \
         A metadata group is one field/value pair, such as `NPC classes = Carboline alkaloids` \
         or `Ion mode = Positive`."
    )?;
    writeln!(file)?;

    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => {
            writeln!(
                file,
                "The table is sorted by **Positive %**, meaning the percentage of spectra inside that \
                 group whose formulas contain the target element. Only groups with at least \
                 `{TOP_ENRICHED_MIN_TOTAL_SUPPORT}` total spectra are included."
            )?;
        }

        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => {
            writeln!(
                file,
                "The table is sorted by **Positive %**, meaning the percentage of molecules inside that \
                 group whose formulas contain the target element. Only groups with at least \
                 `{TOP_ENRICHED_MIN_TOTAL_SUPPORT}` total molecules are included."
            )?;
        }
    }

    writeln!(file)?;
    writeln!(
        file,
        "This table answers: **where is the target element unusually common?** \
         It does not necessarily show the groups with the largest absolute number of positives."
    )?;
    writeln!(file)?;

    if groups.is_empty() {
        writeln!(file, "No enriched groups met the minimum support threshold.")?;
        return Ok(());
    }

    writeln!(file, "| Metadata group | Value | Total | Positive | Positive % | % of positives |")?;
    writeln!(file, "|---|---|---:|---:|---:|---:|")?;

    for group in groups {
        writeln!(
            file,
            "| {} | {} | {} | {} | {:.2}% | {:.2}% |",
            group.metadata_group,
            group.value,
            group.total_count,
            group.target_count,
            group.percent_target_within_group,
            group.percent_of_all_target
        )?;
    }

    Ok(())
}

pub(super) fn write_warning_summary(
    file: &mut File,
    source: &DatasetSource,
    warnings: &[WarningSummary],
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Low-support warning summary")?;
    writeln!(file)?;

    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => {
            writeln!(
                file,
                "This section summarizes warning flags from the population-map CSV tables. \
                 The `Count` column is the number of metadata-group rows with that warning, \
                 not the number of spectra."
            )?;
        }

        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => {
            writeln!(
                file,
                "This section summarizes warning flags from the population-map CSV tables. \
                 The `Count` column is the number of metadata-group rows with that warning, \
                 not the number of molecules."
            )?;
        }
    }

    writeln!(file)?;
    writeln!(file, "Warning meanings:")?;
    writeln!(file)?;
    writeln!(file, "| Warning | Meaning |")?;
    writeln!(file, "|---|---|")?;

    writeln!(
        file,
        "| `LOW_TOTAL_SUPPORT` | The group has fewer than the minimum number of records. |"
    )?;

    writeln!(
        file,
        "| `LOW_TARGET_SUPPORT` | The group has some target-positive records, but too few for confident interpretation. |"
    )?;

    writeln!(
        file,
        "| `NO_TARGET_POSITIVES` | The group has no records whose formulas contain the target element. |"
    )?;

    writeln!(file)?;

    if warnings.is_empty() {
        writeln!(file, "No low-support warnings were found in the population tables.")?;
        return Ok(());
    }

    writeln!(file, "| Warning | Count |")?;
    writeln!(file, "|---|---:|")?;

    for warning in warnings {
        writeln!(file, "| `{}` | {} |", warning.warning, warning.count)?;
    }

    Ok(())
}

pub(super) fn write_interpretation_guide(
    file: &mut File,
    source: &DatasetSource,
    target_element: &str,
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## How to interpret this report")?;
    writeln!(file)?;

    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => {
            writeln!(
                file,
                "This report treats each spectrum as **positive** when its molecular formula contains \
                 the target element `{target_element}`. A spectrum is **negative** when its formula does \
                 not contain `{target_element}`."
            )?;
        }

        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => {
            writeln!(
                file,
                "This report treats each molecule as **positive** when its molecular formula contains \
                 the target element `{target_element}`. A molecule is **negative** when its formula does \
                 not contain `{target_element}`."
            )?;
        }
    }

    writeln!(file)?;
    writeln!(
        file,
        "A **metadata group** means one metadata field and one value inside that field. \
         For example, in the `NPC classes` table, `Carboline alkaloids` is one group. \
         In the `Ion mode` table, `Positive` is one group."
    )?;

    writeln!(file)?;
    writeln!(
        file,
        "The profiler compares the target-positive records against these groups to show \
         where the target element is common, rare, concentrated, or poorly supported."
    )?;

    writeln!(file)?;
    writeln!(file, "Important caveats:")?;

    writeln!(
        file,
        "- These reports are based on formula metadata, not direct spectral proof of the element."
    )?;

    writeln!(
        file,
        "- Some metadata fields can contain multiple pipe-separated values, so assignment counts \
         can be larger than the number of records."
    )?;

    writeln!(
        file,
        "- Highly enriched small groups can be interesting, but they should not be overinterpreted \
         without checking support counts."
    )?;

    Ok(())
}

pub(super) fn write_glossary_and_references(file: &mut File, source: &DatasetSource) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Glossary and external references")?;
    writeln!(file)?;
    writeln!(file, "| Term | Meaning in this report | Reference |")?;
    writeln!(file, "|---|---|---|")?;

    writeln!(
        file,
        "| Molecular formula | Formula metadata used to decide whether a record is target-positive. | [PubChem glossary - Molecular Formula](https://pubchem.ncbi.nlm.nih.gov/docs/glossary#section=Molecular-Formula) |"
    )?;

    writeln!(
        file,
        "| Metadata group | A group formed from one metadata field and one value, such as `NPC classes = Carboline alkaloids`. | Local report definition |"
    )?;

    writeln!(
        file,
        "| Source dataset | The dataset or library source from which the metadata originated. | [GNPS libraries](https://ccms-ucsd.github.io/GNPSDocumentation/gnpslibraries/) / [MassSpecGym](https://github.com/pluskal-lab/MassSpecGym) |"
    )?;

    writeln!(
        file,
        "| Enrichment | A group has high enrichment when a large percentage of records in that group are target-positive. | Local report definition |"
    )?;

    writeln!(
        file,
        "| Low support | A warning that a group has too few total records, too few target-positive records, or no target-positive records. | Local report definition |"
    )?;

    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => {
            writeln!(
                file,
                "| Target-positive spectrum | A spectrum whose molecular formula contains the selected target element. | Local report definition |"
            )?;

            writeln!(
                file,
                "| NPC pathways / superclasses / classes | Natural-product classification fields from NPClassifier-style annotations. | [NPClassifier](https://npclassifier.ucsd.edu/) |"
            )?;

            writeln!(
                file,
                "| ClassyFire taxonomy | Chemical taxonomy fields such as kingdom, superclass, class, subclass, and direct parent. | [ClassyFire paper](https://pmc.ncbi.nlm.nih.gov/articles/PMC5096306/) |"
            )?;
        }

        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => {
            writeln!(
                file,
                "| Target-positive molecule | A molecule whose molecular formula contains the selected target element. | Local report definition |"
            )?;
        }
    }

    Ok(())
}

pub(super) fn write_report_links(file: &mut File, source: &DatasetSource) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Summary")?;
    writeln!(file)?;

    writeln!(file, "- [Summary table](tables/summary.csv)")?;
    writeln!(file, "- Tables are in [`tables/`](tables/)")?;
    writeln!(file, "- Figures are in [`figures/`](figures/)")?;

    writeln!(file)?;
    writeln!(file, "## How to read the figures")?;
    writeln!(file)?;

    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => {
            writeln!(
                file,
                "- **Target count** shows which groups contribute the most target-positive spectra."
            )?;

            writeln!(
                file,
                "- **Percent target** shows which groups are most enriched for the target element across spectra."
            )?;

            writeln!(
                file,
                "- Small groups can look highly enriched, so check the linked CSV tables for support counts."
            )?;
        }

        DatasetSource::PubChemSmiles | DatasetSource::LocalSmilesGz(_) => {
            writeln!(
                file,
                "- **Target count** shows which groups contribute the most target-positive molecules."
            )?;

            writeln!(
                file,
                "- **Percent target** shows which groups are most enriched for the target element across molecules."
            )?;

            writeln!(
                file,
                "- Small groups can look highly enriched, so check the linked CSV tables for support counts (molecule-level support)."
            )?;
        }
    }

    Ok(())
}
