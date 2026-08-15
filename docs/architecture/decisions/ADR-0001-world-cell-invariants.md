# ADR-0001 — World Cell Invariants

- **Status:** Accepted
- **Date:** 2026-08-15

## Context

Powdergame could model each Cell as a rich mixture containing multiple Matter amounts, mass, composition and many future states. That direction resembles some simulation games but conflicts with the core identity the user wants from DAN-BALL Powder Game: direct spatial interaction where one pixel/cell is one thing.

The user explicitly treated this simplicity as part of the game identity, comparable to Minecraft's strong one-position/one-block rule.

## Decision

### One Cell = Max One Matter

A Cell contains at most one Matter identity at a time.

### Unit Cell Quantity

Matter presence is one unit. Per-cell `0.3 Water`, `50% Oil`, mixed composition and similar amount models are not part of the baseline Cell model.

### EMPTY is not Matter

`EMPTY` means no Matter occupies the Cell.

It is not hidden Air, Vacuum Material, heat medium or pressure medium.

### Fields are not second Matter

Temperature, Pressure and small state/flags may be associated with a Cell without violating single Matter occupancy.

### Finite editable boundary

The world is finite. Outer BLOCK may be erased. Matter leaving the domain through an opened boundary is lost to Void.

## Consequences

### Positive

- very small and predictable Cell model
- easier GPU memory layout
- strong spatial identity
- interactions occur between neighboring Matter rather than hidden mixtures
- content complexity grows through relations, not per-cell composition width

### Tradeoffs

- true physical mixtures are not represented directly
- gas/liquid combinations may need local displacement, phase or reaction rules instead of mixture fractions
- some realistic phenomena will be approximated through game-consistent rules

These tradeoffs are intentional.

## Rejected alternatives

### Mixed material amounts per Cell

Rejected because it makes each Cell more expensive and weakens the Powder Game identity.

### Air fills every empty Cell

Rejected because every movement would implicitly become Matter↔Air exchange and EMPTY would no longer be truly empty.

### Vacuum as a normal registered Matter

Rejected because it gives absence of Matter unnecessary properties and reactions.

## User provenance

The decision was repeatedly confirmed during the Foundation Design Session. The user emphasized that the game should not inherit ONI-style same-space multi-material/mass composition and that one-space-one-thing is an important gameplay identity, not merely a technical limitation.

See `docs/design-history/2026-08-15-foundation-design-session.md`.
