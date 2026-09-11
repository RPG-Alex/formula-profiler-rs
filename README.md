# molecular-profiler-rs

`molecular-profiler-rs` is a Rust tool for profiling molecular formula datasets before using them in machine-learning workflows.

This project is intended to support careful dataset inspection before training models such as MS/MS-to-atom classifiers.

## Dataset sources

Datasets that are supported:
- `annotated_ms2` MS/MS dataset exposed by [`mascot-rs`](https://github.com/earth-metabolome-initiative/mascot-rs).
- `pubchem` dataset available from [PubChem](https://pubchem.ncbi.nlm.nih.gov/docs/downloads)
- `lotus` dataset available from [Natural Products](https://lotus.naturalproducts.net/)
- `coconut` dataset available from [Natural Products](https://coconut.naturalproducts.net/)
- `local-mgf` for a local MGF files
- `smiles-csv` for a local CSV file containing SMILES records (can specify CSV fields)

## Usage

The general command format is:

```bash
cargo run --release -- --target <ELEMENT> <DATASET>
```

Use an element symbol such as `F` or `Cl`, or use `all` to profile every observed element.

### Profile annotated MS/MS data

```bash
cargo run --release -- --target F annotated
```

### Profile PubChem

```bash
cargo run --release -- --target all pubchem
```

### Profile a local MGF file

```bash
cargo run --release -- --target all local-mgf path/to/local_file.mgf
```

### Profile a local SMILES CSV

Provide the CSV path followed by the column labels in their actual file order:

```bash
cargo run --release -- \
  --target all \
  smiles-csv path/to/molecules.csv \
  --fields id smiles class \
  --has-headers
```

The `--fields` values must describe every CSV column in order. Exactly one `id` field and one `smiles` field are required.

Omit `--has-headers` when the first row contains data rather than column headers.

### Limit the number of records

```bash
cargo run --release -- --target F --limit 1000 pubchem
```

### View command help

```bash
cargo run --release -- --help
cargo run --release -- smiles-csv --help
```

# Reports

## Generated reports

After running the profiler, open [`REPORTS.md`](REPORTS.md) for links to generated dataset reports.

## Report output

The generated `README.md` inside each report directory is the main human-readable report. It links to the CSV tables and embeds the generated SVG figures.

## Contributing

Contributions are welcome.

## License

MIT
