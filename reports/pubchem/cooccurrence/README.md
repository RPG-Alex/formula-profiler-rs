# Element co-occurrence profile

This report summarizes which chemical elements appear together in molecular formulas across the dataset.

## Summary

| Metric | Value |
|---|---:|
| Profiled molecules | 124589269 |
| Observed elements | 118 |

Heatmap elements shown: `H`, `C`, `N`, `O`, `S`, `F`, `Cl`, `Br`, `I`, `P`, `Si`, `B`, `Na`, `Ir`, `K`, `Pt`, `Se`, `Li`, `Sn`, `Zr`, `Y`, `Al`, `Fe`, `Ti`, `Zn`, `Cu`, `Mg`, `Ni`, `Pd`, `Ge`, `Ca`, `Co`, `W`, `V`, `Ru`, `As`, `Mn`, `Cr`, `Hf`, `U`, `Te`, `Mo`, `Sb`, `Ag`, `Pb`, `Au`, `Ba`, `Bi`, `Hg`, `Rh`, `In`, `Cs`, `Ga`, `Rb`, `Re`, `Ac`, `Os`, `Sr`, `Ce`, `La`, `Cd`, `Gd`, `Nb`, `Tl`, `Ta`, `Eu`, `Nd`, `Rf`, `Ar`, `Pr`, `Tc`, `Sc`, `Sm`, `Yb`, `Be`, `Tb`, `Lu`, `Dy`, `Er`, `Fm`, `Ho`, `Th`, `Po`, `Tm`, `At`, `Lr`, `Cm`, `Xe`, `Pu`, `Np`, `He`, `Pm`, `Am`, `No`, `Es`, `Ra`, `Pa`, `Cf`, `Ne`, `Bk`, `Kr`, `Rn`, `Sg`, `Db`, `Fr`, `Md`, `Mt`, `Bh`, `Hs`, `Ds`, `Rg`, `Fl`, `Nh`, `Cn`, `Lv`, `Mc`, `Ts`, `Og`.

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
