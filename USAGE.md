# Usage

Use an element symbol such as `F` or `Cl`, or use `all` to profile every observed element.

## Profile annotated MS/MS data

```bash
cargo run --release -- --target F annotated
```

## Profile PubChem

```bash
cargo run --release -- --target all pubchem
```

## Profile a local MGF file

```bash
cargo run --release -- --target all local-mgf path/to/local_file.mgf
```

## Profile a local SMILES CSV

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

## Limit the number of records

```bash
cargo run --release -- --target F --limit 1000 pubchem
```

## View command help

```bash
cargo run --release -- --help
cargo run --release -- smiles-csv --help
```