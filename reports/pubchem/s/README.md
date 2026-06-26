# `S` profile for `pubchem`

This report summarizes how often the target element `S` appears across metadata groups in `pubchem`.

## How to interpret this report

This report treats each spectrum as **positive** when its molecular formula contains the target element `S`. A spectrum is **negative** when its formula does not contain `S`.

A **metadata group** means one metadata field and one value inside that field. For example, in the `NPC classes` table, `Carboline alkaloids` is one group. In the `Ion mode` table, `Positive` is one group.

The profiler compares the target-positive spectra against these groups to show where the target element is common, rare, concentrated, or poorly supported.

Important caveats:
- These reports are based on formula metadata, not direct spectral proof of the element.
- Some metadata fields can contain multiple pipe-separated values, so assignment counts can be larger than the number of spectra.
- Highly enriched small groups can be interesting, but they should not be overinterpreted without checking support counts.

## Glossary and external references

| Term | Meaning in this report | Reference |
|---|---|---|
| Molecular formula | Formula metadata used to decide whether a spectrum is target-positive. | [PubChem glossary - Molecular Formula](https://pubchem.ncbi.nlm.nih.gov/docs/glossary#section=Molecular-Formula) |
| Target-positive spectrum | A spectrum whose molecular formula contains the selected target element. | Local report definition |
| Metadata group | A group formed from one metadata field and one value, such as `NPC classes = Carboline alkaloids`. | Local report definition |
| NPC pathways / superclasses / classes | Natural-product classification fields from NPClassifier-style annotations. | [NPClassifier](https://npclassifier.ucsd.edu/) |
| ClassyFire taxonomy | Chemical taxonomy fields such as kingdom, superclass, class, subclass, and direct parent. | [ClassyFire paper](https://pmc.ncbi.nlm.nih.gov/articles/PMC5096306/) |
| Source dataset | The dataset or library source from which the spectrum metadata originated. | [GNPS libraries](https://ccms-ucsd.github.io/GNPSDocumentation/gnpslibraries/) / [MassSpecGym](https://github.com/pluskal-lab/MassSpecGym) |
| Enrichment | A group has high enrichment when a large percentage of spectra in that group are target-positive. | Local report definition |
| Low support | A warning that a group has too few total spectra, too few target-positive spectra, or no target-positive spectra. | Local report definition |

## Numeric summary

| Metric | Value |
|---|---:|
| Total spectra | 36179085 |
| Positive count | 36179085 |
| Negative count | 0 |
| Positive percentage | 100.0000% |

## Atom-count distribution

This section shows how many formula-bearing spectra have exactly `k` atoms of `S`.
The `0` row represents formulas that do not contain `S`.

[CSV table](tables/target_atom_count_distribution.csv)

<img src="figures/target_atom_count_distribution.svg" alt="S atom-count distribution" />

## Top enriched groups

This table compares **metadata groups** across all population-map tables. A metadata group is one field/value pair, such as `NPC classes = Carboline alkaloids` or `Ion mode = Positive`.

The table is sorted by **Positive %**, meaning the percentage of spectra inside that group whose formulas contain the target element. Only groups with at least `30` total spectra are included.

This table answers: **where is the target element unusually common?** It does not necessarily show the groups with the largest absolute number of positives.

| Metadata group | Value | Total | Positive | Positive % | % of positives |
|---|---|---:|---:|---:|---:|
| Source dataset | PubChem | 36179085 | 36179085 | 100.00% | 100.00% |
| Source dataset | TOTAL_RECORDS | 36179085 | 36179085 | 100.00% | 100.00% |
| Source dataset | TOTAL_ASSIGNMENTS | 36179085 | 36179085 | 100.00% | 100.00% |

## Low-support warning summary

This section summarizes warning flags from the population-map CSV tables. The `Count` column is the number of metadata-group rows with that warning, not the number of spectra.

Warning meanings:

| Warning | Meaning |
|---|---|
| `LOW_TOTAL_SUPPORT` | The group has fewer than the minimum number of total spectra. |
| `LOW_TARGET_SUPPORT` | The group has some target-positive spectra, but too few for confident interpretation. |
| `NO_TARGET_POSITIVES` | The group has no spectra whose formulas contain the target element. |

No low-support warnings were found in the population tables.

## Summary

- [Summary table](tables/summary.csv)
- Tables are in [`tables/`](tables/)
- Figures are in [`figures/`](figures/)

## How to read the figures

- **Target count** shows which groups contribute the most target-positive spectra.
- **Percent target** shows which groups are most enriched for the target element.
- Small groups can look highly enriched, so check the linked CSV tables for support counts.

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
