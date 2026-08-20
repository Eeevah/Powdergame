# G9 Product Brief — First Playable World

Date: 2026-08-19  
Status: **USER APPROVED / AUTHORIZED / IMPLEMENTATION NOT STARTED**  
Precondition: G8 **CLOSED / FROZEN**, official Matrix recommendation `PROCEED_TO_G9`

This document is the canonical product scope for the first G9 implementation line. It translates `docs/vision/FIRST_PLAYABLE_WORLD.md`, `docs/vision/UI_DIRECTION.md` and the G9 Gate in `MILESTONES.md` into the choices explicitly approved by the user.

## 1. Product goal

> **Give the player an editable living world, let a small hypothesis create a visible consequence, and make the next experiment feel irresistible.**

G9 is not another fixed Gallery fixture and not another performance-instrument round. It is the first actual Powdergame vertical slice.

## 2. Approved start experience

### Default: Starter Lab

The first playable opens into a small, editable Starter Lab that helps the player act immediately without forcing one correct solution.

The user-testable candidate uses this product default for the canonical no-argument BAT/EXE launch. The frozen G8 Gallery remains available only through its explicit `gallery` / `--benchmark-gallery` route.

The Starter Lab must:

- use the ordinary production simulation and common M0 rules;
- remain fully editable;
- leave meaningful empty space for the player's own structures;
- make Sand, Water, Heat, phase change, combustion and pressure experiments possible without scripting their outcomes;
- avoid an automatic sequence that plays the game for the user;
- function as a starting point, not a tutorial puzzle with one answer.

Exact geometry is an implementation proposal and remains subject to user review.

### Immediate alternative: New Blank World

`New Blank World` is visible and immediately available from the same product surface.

The blank world uses the normal world/boundary contract and contains no hidden scenario-specific result. The player does not have to finish or dismiss the Starter Lab before choosing it.

## 3. Initial Matter palette

All current M0 Matter is visible and usable from the beginning:

- Boundary Block
- Stone
- Sand
- Ice
- Water
- Steam
- Smoke
- Wood
- Oil

Discovery does **not** unlock or hide these Matter entries in the first slice.

The palette should communicate canonical names and the current selection without requiring the player to memorize colors. Exact button layout and shortcuts are implementation details, but the world remains visually primary.

## 4. G9-A editor MVP

The approved first interaction set is:

- Matter selection
- Draw
- Erase
- brush-size change
- Heat
- Cool
- Pause / Play
- Single Step
- speed control: x1 / x4 / x16
- Reset
- Pan
- Zoom
- preset load
- existing compact Hover Inspector
- existing `I` detailed Cell Inspector

Required product behavior:

- edits affect the GPU-authoritative production world through a safe edit-command path;
- UI code does not create a second CPU simulation truth;
- One Cell = Max One Matter remains intact;
- erase produces EMPTY with valid field/flag hygiene;
- Matter identity and relevant fields are written consistently for the next production tick;
- edited chunks and the required safety halo wake correctly;
- clipping at world boundaries is safe and deterministic;
- edit operations remain responsive without per-pointer full-world readback;
- reset restores the selected preset's pristine state;
- Pan/Zoom and cursor-to-cell picking use one authoritative viewport transform;
- Inspector values remain honest about sample freshness after edits, reset, preset changes and camera movement.

Exact key bindings, pointer buttons and discrete brush sizes are implementation-level choices. They must preserve existing play controls, avoid conflicts, appear in the minimal HUD and be reported for user review.

## 5. G9-B open emergence

After the editor is usable, the user must be able to create ordinary sandbox setups that combine the existing rules, including:

```text
Sand / Water / Oil movement and layering
Ice ↔ Water ↔ Steam
Wood / Oil combustion
sealed chamber → Pressure → rupture → vent
Heat / Smoke / Pressure causing follow-up change
```

These outcomes must come from the existing common Rule chain. G9 does not add a `boiler_explosion()`-style answer script or scenario-specific production physics.

A failed experiment is valid information when the world stays safe and the result is understandable from the same rules.

## 6. G9-C Discovery MVP

Discovery begins **after the editor core works**, but remains part of the same G9 product milestone.

The first Research Note records phenomenon-level observations from actual simulation state or semantic events:

- Phase Change
- Combustion Started
- Combustion Extinguished
- Pressure Generated
- Rupture / Vent
- Matter Transformation
- meaningful No Reaction / Resistance observation

Discovery policy:

- all M0 Matter remains usable from the start;
- no material-unlock economy in the first slice;
- no recipe tree or exact completion percentage;
- exact thresholds, coefficients and remaining discovery count stay hidden;
- a discovery is recorded only when the runtime truth supports it;
- reset, preset change or replayed UI state must not fabricate a discovery;
- the note is the player's research record, not a complete answer key.

## 7. Save / Load and Rewind

Both are **deferred from the first playable acceptance slice**.

This is a scope decision, not a rejection of their long-term value.

- Save/Load is postponed until the basic sandbox is worth preserving.
- Rewind remains an important future experiment tool and a core part of the longer product direction, but it does not block the first G9 validation.
- The first implementation must not create a throwaway architecture that makes later Rewind impossible, but it does not build Rewind infrastructure speculatively.

## 8. Honest presentation

The first slice does not require the final art stack. It does require enough feedback that the player is not judging only raw diagnostic colors.

Minimum direction:

- world remains the visual focus;
- selected Matter/tool, brush size, pause state and speed are readable;
- Material identity is available through the palette and Inspector;
- Heat, combustion, Smoke and rupture/vent receive honest, minimal feedback derived from real state/events;
- presentation never invents movement, reaction or pressure that did not occur;
- benchmark provenance and full diagnostic counters stay outside the normal product HUD.

Detailed overlays and final FX remain later polish unless user validation shows they are required to understand causality.

## 9. Manual acceptance session

The first product-validation session is approximately **10–15 minutes**.

Conditions:

- no prescribed objective;
- no expected result or correct recipe is revealed;
- the user may start in the Starter Lab or choose New Blank World;
- the evaluator answers questions but does not direct the experiment sequence;
- technical instrumentation may observe correctness, but it does not replace user judgment.

Primary strong success signals:

1. the user voluntarily begins a second experiment without being told;
2. the user can explain at least one result's cause without knowing exact numeric thresholds.

Additional positive signals:

- the same Matter is reused in a different way;
- an unexpected but understandable result is intentionally recreated;
- the user changes one condition to test a hypothesis;
- the user asks what will happen with another combination;
- the interface feels like a game rather than a benchmark viewer.

Only the user can give the final G9/M0 product disposition.

## 10. Implementation order

The approved order is:

```text
G9-A editor and world interaction candidate
→ user checks basic control and comprehension
→ G9-B free-form emergence validation
→ G9-C Research Note MVP
→ G9-D minimum honest presentation
→ G9-E 10–15 minute user product validation
```

The first Codex task is **G9-A only**. It should stop with a user-testable implementation candidate before expanding into Discovery, Save/Load, Rewind or broad presentation work.

## 11. Explicit non-goals for the first G9-A task

- new Matter
- material unlock progression
- Recipe/technology tree
- Save/Load
- Rewind
- final audio/FX stack
- Interaction Lab
- G7-C compaction or indirect dispatch
- packing, f16 or speculative optimization
- G8 evidence recapture
- `main` promotion
- M0 `ACHIEVED` declaration

## 12. Validation boundary

Automated validation may establish editor correctness, edit hygiene, input mapping, reset/preset behavior, regression safety and bounded performance.

It cannot establish whether the sandbox is fun, understandable or motivating. G9 advances to final product validation only through direct user play.

Existing exact-source G8 evidence is not rerun because this product brief or other docs/memory files changed.

## 13. First candidate user review disposition

The first G9-A candidate was **USER REVIEWED / REVISION REQUIRED**. Source `b363c078fdc1d7e8b54fa6be328b7a0c5b908f06` implemented the five authorized revisions. Direct re-review confirmed those changes were present but still classified G9-A **USER RE-REVIEWED / REVISION REQUIRED** because Inspector continuity flickered during rapid Cell movement. Source `a00e39b2e00bfbd9ac28214c44cd22cc97542bb4` implements continuity v2. The latest direct disposition records Inspector continuity v2 **USER ACCEPTED** and G9-A overall **USER ACCEPTED WITH KNOWN FOLLOW-UP**. Thermal Environment/phase work remains separately gated; TE-2 requires re-review after candidate-only remediation and TE-3 is design-required/not-started. None of this authorizes G9-B/C/D/E or any non-goal above.
