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

See [`USAGE`](USAGE.md) for detailed usage assistance.

## Reports

### Generated reports

After running the profiler, open [`REPORTS.md`](REPORTS.md) for links to generated dataset reports.

#### Report output

The generated `README.md` inside each report directory is the main human-readable report. It links to the CSV tables and embeds the generated SVG figures.

## Contributing

Contributions are welcome.

## License

MIT
