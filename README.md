# spectra-profiler-rs

`spectra-profiler-rs` is a Rust tool for profiling annotated MS/MS spectral datasets before using them in machine-learning workflows.

The current focus is **element-level population analysis**. Given a target element such as `F`, `Cl`, `Br`, or `I`, the profiler reports how often that element appears across metadata groups such as NPC pathways, NPC superclasses, NPC classes, source datasets, organisms, ion modes, instruments, and library-quality annotations.

The tool can also profile **all observed elements** in a dataset.

This project is intended to support careful dataset inspection before training models such as MS/MS-to-atom classifiers.

## Purpose

Before training a model, it is useful to know what the dataset actually contains.

For example, if we want to train a classifier that predicts whether a spectrum belongs to a fluorinated molecule, we need to know:

* how many fluorinated spectra exist,
* which chemical classes they belong to,
* whether they are concentrated in a few source datasets,
* whether certain organisms or instruments dominate the positive examples,
* whether other elements have enough support to be useful modeling targets.

`spectra-profiler-rs` generates reports to help answer those questions.

## Functionality

For a selected target element, the profiler:

1. Loads an MS/MS dataset.
2. Reads molecular formulas from spectrum metadata.
3. Checks whether each formula contains the target element.
4. Aggregates target-element presence across metadata fields.
5. Writes CSV population-map tables.
6. Generates SVG visualizations.
7. Creates a Markdown report that links the generated tables and figures.

For `all` mode, the profiler:

1. Loads the dataset once.
2. Detects all element symbols observed in molecular formulas.
3. Generates one report folder per observed element.

## Dataset sources

By default, this tool uses the `annotated_ms2` dataset exposed by [`mascot-rs`](https://github.com/earth-metabolome-initiative/mascot-rs).

Relevant upstream resources:

* [`mascot-rs`](https://github.com/earth-metabolome-initiative/mascot-rs)
* [`mascot-rs` annotated MS2 documentation](https://github.com/earth-metabolome-initiative/mascot-rs#annotated-ms2)
* [GNPS](https://gnps.ucsd.edu/)
* [MassSpecGym](https://github.com/pluskal-lab/MassSpecGym)

Local MGF files are also supported.

Element profiling depends on molecular formula metadata being available. If a local MGF file does not include parseable formulas, target-element counts will not be meaningful.

## Installation

Clone the repository:

```bash
git clone https://github.com/<OWNER>/spectra-profiler-rs.git
cd spectra-profiler-rs
```

Build the project:

```bash
cargo build --release
```

Run the test suite:

```bash
cargo test
```

## Usage

### Profile one element

Profile the default annotated MS2 dataset for fluorine:

```bash
cargo run --release -- F
```

Profile chlorine:

```bash
cargo run --release -- Cl
```

Profile bromine:

```bash
cargo run --release -- Br
```

Profile iodine:

```bash
cargo run --release -- I
```

The element symbol is normalized automatically, so these are equivalent:

```bash
cargo run --release -- f
cargo run --release -- F
```

### Profile all observed elements

Profile every element observed in the default annotated MS2 dataset:

```bash
cargo run --release -- all
```

This generates one report directory per observed element.

For example:

```text
reports/annotated_ms2/c/
reports/annotated_ms2/h/
reports/annotated_ms2/n/
reports/annotated_ms2/o/
reports/annotated_ms2/f/
reports/annotated_ms2/cl/
reports/annotated_ms2/br/
reports/annotated_ms2/i/
```

The exact set of folders depends on which elements are present in the loaded dataset.

### Profile a local MGF file

Profile a local MGF file for fluorine:

```bash
cargo run --release -- F path/to/local_file.mgf
```

Profile all observed elements in a local MGF file:

```bash
cargo run --release -- all path/to/local_file.mgf
```

When a local MGF file is provided, reports are written under a report directory named after the file stem.

For example:

```bash
cargo run --release -- F data/example.mgf
```

writes to:

```text
reports/example/f/
```

## Report output

Reports are written to:

```text
reports/<dataset_name>/<element>/
```

For the default annotated MS2 dataset and fluorine, the output path is:

```text
reports/annotated_ms2/f/
```

Each report folder contains:

```text
reports/annotated_ms2/f/
├── README.md
├── tables/
│   ├── summary.csv
│   ├── contains_by_npc_pathways.csv
│   ├── contains_by_npc_superclasses.csv
│   ├── contains_by_npc_classes.csv
│   ├── contains_by_source_dataset.csv
│   ├── contains_by_organism.csv
│   ├── contains_by_ion_mode.csv
│   ├── contains_by_source_instrument.csv
│   └── contains_by_library_quality.csv
└── figures/
    ├── top_npc_pathways_by_target_count.svg
    ├── top_npc_pathways_by_percent_target.svg
    ├── top_npc_superclasses_by_target_count.svg
    ├── top_npc_superclasses_by_percent_target.svg
    └── ...
```

The generated `README.md` inside each report directory is the main human-readable report. It links to the CSV tables and embeds the generated SVG figures.

## How to read the reports

Each report contains two main kinds of visualizations.

### Target-count plots

Target-count plots show which groups contribute the largest number of target-positive spectra.

These answer questions like:

* Which NPC classes contain the most fluorinated spectra?
* Which source datasets contribute most of the chlorine-positive examples?
* Which instruments dominate the bromine-positive population?

These plots are useful for identifying population dominance.

### Percent-target plots

Percent-target plots show which groups are most enriched for the target element.

These answer questions like:

* Which groups have the highest fraction of iodine-containing spectra?
* Are some small classes unusually enriched for fluorine?
* Are certain metadata groups highly target-specific?

These plots should be interpreted carefully. A tiny group can have a high percentage while still having weak support.

Always check the corresponding CSV table before drawing conclusions from percent-target plots.

## Table columns

Population-map CSV files include the following columns:

| Column                        | Meaning                                                           |
| ----------------------------- | ----------------------------------------------------------------- |
| `value`                       | Metadata group value, such as an NPC class or source dataset      |
| `total_count`                 | Number of spectra in that group                                   |
| `target_count`                | Number of spectra in that group containing the target element     |
| `non_target_count`            | Number of spectra in that group not containing the target element |
| `percent_target_within_group` | Percent of the group that contains the target element             |
| `percent_of_all_records`      | Percent of the full dataset represented by the group              |
| `percent_of_all_target`       | Percent of all target-positive spectra represented by the group   |
| `support_warning`             | Warning flags for small or unsupported groups                     |

## Summary table

Each report contains:

```text
tables/summary.csv
```

The summary table includes:

| Metric                        | Meaning                                                     |
| ----------------------------- | ----------------------------------------------------------- |
| `target_element`              | The profiled element                                        |
| `total_records`               | Total number of spectra loaded                              |
| `records_with_formula`        | Number of spectra with formula metadata                     |
| `records_with_target_element` | Number of spectra whose formula contains the target element |

This is the fastest way to check whether an element has enough support to be worth modeling.

## Support warnings

The profiler adds support warnings to help avoid overinterpreting tiny groups.

Current warning types include:

| Warning               | Meaning                                                      |
| --------------------- | ------------------------------------------------------------ |
| `LOW_TOTAL_SUPPORT`   | The group has few total spectra                              |
| `NO_TARGET_POSITIVES` | The group has no target-positive spectra                     |
| `LOW_TARGET_SUPPORT`  | The group has only a small number of target-positive spectra |

These warnings are important for percent-based plots, where small groups can appear highly enriched despite weak support.

## Cache and generated files

Downloaded datasets are cached under:

```text
cache/
```

The cache should not be committed to Git.

Generated reports are written under:

```text
reports/
```

Reports may be committed when they are intended to be published or reviewed.

## Development

Run formatting:

```bash
cargo fmt --all
```

Run Clippy:

```bash
cargo clippy --all-targets -- -D warnings
```

Run tests:

```bash
cargo test
```

Build an optimized release binary:

```bash
cargo build --release
```

Run a release-mode profile:

```bash
cargo run --release -- F
```

Release mode is recommended for normal profiling because the annotated MS2 dataset is large.

## Contributing

Contributions are welcome.

Useful contribution areas include:

* Adding support for additional dataset sources.
* Improving local MGF handling.
* Adding new population-map report types.
* Improving visualizations.
* Adding summary statistics.
* Improving CLI options.
* Adding tests for formula parsing.
* Adding tests for population-map aggregation.
* Adding tests for Markdown report generation.
* Improving CI and repository hygiene checks.

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

MIT
