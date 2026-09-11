# Element co-occurrence profile

This report summarizes which chemical elements appear together in molecular formulas across the dataset.

## Summary

| Metric | Value |
|---|---:|
| Profiled molecules | 738816 |
| Observed elements | 87 |

Heatmap elements shown: `H`, `C`, `O`, `N`, `S`, `Cl`, `Br`, `P`, `Na`, `I`, `F`, `Si`, `B`, `Fe`, `As`, `K`, `Se`, `Mg`, `Co`, `Ca`, `Ni`, `Zn`, `Hg`, `Al`, `Cu`, `V`, `Mn`, `Gd`, `Sn`, `Li`, `Bi`, `Ga`, `In`, `Mo`, `Au`, `Cr`, `Sb`, `Ho`, `Pt`, `Sr`, `Tc`, `Te`, `Lu`, `Pb`, `W`, `Ac`, `Ag`, `Cd`, `He`, `Rb`, `Ru`, `Sm`, `Tl`, `Ba`, `Be`, `Ge`, `Ra`, `Ti`, `Y`, `Ar`, `Bk`, `Ce`, `Cs`, `Dy`, `Er`, `Eu`, `Fr`, `Hf`, `Ir`, `La`, `Nb`, `Nd`, `Ne`, `Os`, `Pa`, `Pd`, `Pr`, `Re`, `Sc`, `Ta`, `Tb`, `Th`, `Tm`, `U`, `Xe`, `Yb`, `Zr`.

## Tables

- [Element counts](tables/element_counts.csv)
- [Raw co-occurrence counts](tables/element_cooccurrence_counts.csv)
- [Conditional probabilities](tables/element_cooccurrence_conditional_probability.csv)
- [Normalized PMI](tables/element_cooccurrence_normalized_pmi.csv)


## Heatmaps

### Raw co-occurrence counts

<img src="figures/element_cooccurrence_raw_counts_heatmap.svg" alt="Raw element co-occurrence heatmap" />

### Conditional probability

<img src="figures/element_cooccurrence_conditional_probability_heatmap.svg" alt="Conditional probability element co-occurrence heatmap" />

### Normalized element association (NPMI)

NPMI measures whether two elements co-occur more or less often than expected from their individual frequencies. Values range from -1 to 1: negative values indicate less co-occurrence than expected, 0 indicates independence, and positive values indicate greater co-occurrence than expected.

<img src="figures/element_cooccurrence_normalized_pmi_heatmap.svg" alt="Normalized PMI element co-occurrence heatmap" />
