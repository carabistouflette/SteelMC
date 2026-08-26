# Pre-registered decision rules - block-op layer disposition
Written before any campaign data collection. Primary metric is deterministic counter work (Valgrind-free proxy: perf instructions on an isolated driver, single-thread pinned); wall-clock Criterion medians are secondary corroboration because host drift spans ±3-6% while paired same-minute deltas are ~±1%.

M1 primary = geometric mean over >=8 alternating rounds of (slim/base) ratio of `perf stat` core+atom instruction totals for the isolated driver executing a fixed 300-explosion workload.

Decision (applied mechanically):
  delta_M1 <= +0.7%  -> parity-or-better confirmed at counter level; keep layer. Wall verdict then reported as noise-bound regardless of point estimate.
  delta_M1 >  +0.7%  -> the layer does more work than baseline it replaced -> revert direct-mapped cache commit e45c043f4-equivalent only; rerun one confirmation pair; ship revert.
Anti-gaming: thresholds above were fixed before campaign execution; no result-dependent rule changes permitted afterward.
