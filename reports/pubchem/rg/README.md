# `Rg` profile for `pubchem`

This report summarizes how often the target element `Rg` appears across metadata groups in `pubchem`.

## How to interpret this report

This report treats each molecule as **positive** when its molecular formula contains the target element `Rg`. A molecule is **negative** when its formula does not contain `Rg`.

A **metadata group** means one metadata field and one value inside that field. For example, in the `NPC classes` table, `Carboline alkaloids` is one group. In the `Ion mode` table, `Positive` is one group.

The profiler compares the target-positive records against these groups to show where the target element is common, rare, concentrated, or poorly supported.

Important caveats:
- These reports are based on formula metadata, not direct spectral proof of the element.
- Some metadata fields can contain multiple pipe-separated values, so assignment counts can be larger than the number of records.
- Highly enriched small groups can be interesting, but they should not be overinterpreted without checking support counts.

## Glossary and external references

| Term | Meaning in this report | Reference |
|---|---|---|
| Molecular formula | Formula metadata used to decide whether a record is target-positive. | [PubChem glossary - Molecular Formula](https://pubchem.ncbi.nlm.nih.gov/docs/glossary#section=Molecular-Formula) |
| Metadata group | A group formed from one metadata field and one value, such as `NPC classes = Carboline alkaloids`. | Local report definition |
| Source dataset | The dataset or library source from which the metadata originated. | [GNPS libraries](https://ccms-ucsd.github.io/GNPSDocumentation/gnpslibraries/) / [MassSpecGym](https://github.com/pluskal-lab/MassSpecGym) |
| Enrichment | A group has high enrichment when a large percentage of records in that group are target-positive. | Local report definition |
| Low support | A warning that a group has too few total records, too few target-positive records, or no target-positive records. | Local report definition |
| Target-positive molecule | A molecule whose molecular formula contains the selected target element. | Local report definition |

## Numeric summary

| Metric | Value |
|---|---:|
| Total molecules | 123927373 |
| Positive count | 9 |
| Negative count | 123927364 |
| Positive percentage | 0.0000% |

## Atom-count distribution

This section shows how many formula-bearing molecules have exactly `k` atoms of `Rg`.
The `0` row represents formulas that do not contain `Rg`.

[CSV table](tables/target_atom_count_distribution.csv)

<img src="figures/target_atom_count_distribution.svg" alt="Rg atom-count distribution" />

## Top enriched groups

This table compares **metadata groups** across all population-map tables. A metadata group is one field/value pair, such as `NPC classes = Carboline alkaloids` or `Ion mode = Positive`.

The table is sorted by **Positive %**, meaning the percentage of molecules inside that group whose formulas contain the target element. Only groups with at least `30` total molecules are included.

This table answers: **where is the target element unusually common?** It does not necessarily show the groups with the largest absolute number of positives.

| Metadata group | Value | Total | Positive | Positive % | % of positives |
|---|---|---:|---:|---:|---:|
| Source dataset | PubChem | 123927373 | 9 | 0.00% | 100.00% |
| Source dataset | TOTAL_RECORDS | 123927373 | 9 | 0.00% | 100.00% |
| Source dataset | TOTAL_ASSIGNMENTS | 123927373 | 9 | 0.00% | 100.00% |

## Low-support warning summary

This section summarizes warning flags from the population-map CSV tables. The `Count` column is the number of metadata-group rows with that warning, not the number of molecules.

Warning meanings:

| Warning | Meaning |
|---|---|
| `LOW_TOTAL_SUPPORT` | The group has fewer than the minimum number of records. |
| `LOW_TARGET_SUPPORT` | The group has some target-positive records, but too few for confident interpretation. |
| `NO_TARGET_POSITIVES` | The group has no records whose formulas contain the target element. |

| Warning | Count |
|---|---:|
| `LOW_TARGET_SUPPORT` | 1 |

## Summary

- [Summary table](tables/summary.csv)
- Tables are in [`tables/`](tables/)
- Figures are in [`figures/`](figures/)

## How to read the figures

- **Target count** shows which groups contribute the most target-positive molecules.
- **Percent target** shows which groups are most enriched for the target element across molecules.
- Small groups can look highly enriched, so check the linked CSV tables for support counts (molecule-level support).

## Source dataset

Groups spectra by the dataset or spectral-library source recorded in metadata.

[CSV table](tables/contains_by_source_dataset.csv)

<table>
<tr>
<th>Top groups by target count</th>
<th>Top groups by percent target</th>
</tr>
<tr>
<td width="50%"><img src="figures/top_source_dataset_by_target_count.svg" alt="Source dataset by target count" /></td>
<td width="50%"><img src="figures/top_source_dataset_by_percent_target.svg" alt="Source dataset by percent target" /></td>
</tr>
</table>
