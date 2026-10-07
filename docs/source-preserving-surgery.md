# Source-preserving clip surgery

Modified for concat-song on **2026-10-07**; see [FORK-NOTICE.md](../FORK-NOTICE.md).

Split, trim and the moving portions of freeze now restrict the existing source
time map rather than replacing a varying speed with its mean. For an original
map `M` and retained interval `[a, b]`, the intended mapping is
`M_new(t) = M_old(a + t)`. The implementation restricts the piecewise-linear
speed curve, retains interior knots and discontinuities, and updates the cached
mean. Eased property/effect keys are subdivided along their existing curves.
Floating-point computation and export clock precision still bound equality.

The same interval operation handles transition pre-roll and audio gain slices.
Where an exact derived boundary key cannot be carried by the existing property's
permitted range, the operation refuses it specifically rather than silently
clamping the result. The affected commands stage both project state and minted
IDs before publication. Preview/export planning exposes timing-representation
errors; an invalid preview plan does not quietly become a valid empty scene.

## Boundary joins checked during review and execution

- Export rationalizes **absolute start and end**, then derives their difference.
  Independently rounding start and duration created a hole containing an output
  frame in a reproduced 30 fps sub-microsecond boundary case. Adjacent pieces now
  share a half-open boundary. Half-tick arithmetic residue is canonicalized;
  genuinely separate nearby times remain separate within the declared clock.
- Merging split eased keys retains the left piece's incoming easing at a shared
  boundary. The inherited absorption helper used to overwrite it with the right
  piece's initial linear anchor. Dense split/merge samples reproduce that failure
  and protect the repair. Nearby distinct keys are not collapsed using the
  interactive key-selection tolerance; conflicting boundary values are refused
  atomically.
- The broader rendered suite exposed a one-frame tail disappearing after
  microsecond rounding. Genuinely frame-aligned timeline endpoints now retain
  their exact frame rational, with only floating-arithmetic residue tolerated;
  genuine subframe boundaries remain independent of the output grid.
- Paced decoding needs the source frame whose presentation interval covers the
  requested time. Its seek and software-recovery paths now retain the preceding
  frame for that selection. Unpaced reads remain at-or-after. EOF uses declared
  source-frame duration, then stream end; if both are absent, the last observed
  source cadence is an explicit fallback, not an output-rate-derived hold.

## Evidence and remaining limits

Focused checks cover constant/nonlinear maps, interior sampling, discontinuities,
endpoint extension, eased/overshooting keys, trim/split/freeze, transition
pre-roll, source offsets, rational-boundary ownership, merge, undo and refusals.
CPU decoder controls also cover differing source/output rates and recovery
before the first frame and after progress. The unchanged end-to-end export edge
test passes. These checks are not proof that every rendered image or audio
sample matches a previous render, and no hardware fault was physically induced.

- Genuinely frame-aligned timeline endpoints remain exact frame rationals;
  other endpoints and source positions retain **microsecond rationalization**,
  not arbitrary `f64` timestamp equality. Source in-points are never snapped to
  output FPS. Source/key tolerances are case-specific in the tests.
- Audio tempo remains a stepped approximation; gain shape and each piece's
  source start are checked separately from sample-for-sample sound identity.
- The caller must supply a freeze still depicting the intended source moment.
  This repair preserves moving pieces; it does not verify that supplied image.
- Final output duration keeps its existing nearest-frame policy. Transition
  effect ramps remain frame-shaped.
- Merge still rejects varying-speed clips; no general curve-merge feature is
  claimed. Multistream reattachment is a separate, unrepaired limitation.
- No sync-group, document schema, model inference or creator-preference system
  is introduced by this repair.
- Missing source-duration metadata retains the documented cadence fallback;
  this is not a general variable-frame-rate fidelity guarantee.
