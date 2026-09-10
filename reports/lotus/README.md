# `lotus` profiling reports

This directory contains generated exploratory profiling reports for `lotus`.

The reports summarize element presence from molecular formulas and should be interpreted as dataset profiling, not direct molecules evidence.

Only records successfully converted into a molecular formula profile are included below; malformed or unparseable inputs are reported during dataset processing.

## Dataset facts

| Metric | Value |
|---|---:|
| Profiled molecules | 276518 |
| Observed elements | 14 |

## Dataset-level reports

- [Element co-occurrence profile](cooccurrence/README.md): Contains raw and normalized element co-occurrence heatmaps.

## Observed elements

The following valid chemical elements were observed, ordered by descending frequency.

`C`, `H`, `O`, `N`, `S`, `Cl`, `Br`, `P`, `I`, `F`, `Si`, `As`, `Se`, `B`

## Top observed elements

| Element | Record count | % of profiled molecules |
|---|---:|---:|
| `C` | 276515 | 100.00% |
| `H` | 276511 | 100.00% |
| `O` | 271406 | 98.15% |
| `N` | 63410 | 22.93% |
| `S` | 7777 | 2.81% |
| `Cl` | 6106 | 2.21% |
| `Br` | 4218 | 1.53% |
| `P` | 1104 | 0.40% |
| `I` | 141 | 0.05% |
| `F` | 122 | 0.04% |
| `Si` | 65 | 0.02% |
| `As` | 28 | 0.01% |
| `Se` | 23 | 0.01% |
| `B` | 3 | 0.00% |

## Element reports generated in this run

Each element report summarizes metadata groups for profiled molecules whose formulas contain that element.

| Element | Record count | % of profiled molecules | Report |
|---|---:|---:|---|
| `C` | 276515 | 100.00% | [Open](./c/README.md) |
| `H` | 276511 | 100.00% | [Open](./h/README.md) |
| `O` | 271406 | 98.15% | [Open](./o/README.md) |
| `N` | 63410 | 22.93% | [Open](./n/README.md) |
| `S` | 7777 | 2.81% | [Open](./s/README.md) |
| `Cl` | 6106 | 2.21% | [Open](./cl/README.md) |
| `Br` | 4218 | 1.53% | [Open](./br/README.md) |
| `P` | 1104 | 0.40% | [Open](./p/README.md) |
| `I` | 141 | 0.05% | [Open](./i/README.md) |
| `F` | 122 | 0.04% | [Open](./f/README.md) |
| `Si` | 65 | 0.02% | [Open](./si/README.md) |
| `As` | 28 | 0.01% | [Open](./as/README.md) |
| `Se` | 23 | 0.01% | [Open](./se/README.md) |
| `B` | 3 | 0.00% | [Open](./b/README.md) |
