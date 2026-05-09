# idle-tui — vision: UFO / X-COM–flavored idle (no strategy layer)

Date: 2026-05-09

## Goal

Define the product direction for **idle-tui**: a terminal idle game with a **UFO / X-COM–style containment fantasy** (incidents, squads, pressure) that deliberately **omits the classic strategic command layer**. The player does not run a geoscape turn-by-turn or micromanage tactical combat. Instead, **missions appear on the map**, and **squads autonomously pick suitable mission types, complete them, return to base, and repeat**.

This document is **vision + first gameplay slice**. It does not replace existing MVP-0/MVP-1 designs for TUI shell and tick/pause behavior; those remain the technical baseline.

## Player fantasy

- You run an organization that responds to a world filling with incidents.
- Teams are competent: **they dispatch themselves** when a mission fits their capabilities.
- You observe progress **over time**: resources accrue at the base, squads cycle through missions while simulated time advances.

## Non-goals (vision-level)

- Faithful reproduction of UFO / X-COM tactical combat under player control each turn.
- A full strategic metagame identical to classic geoscape management (manual assignments per mission from a dense map UI is not the target paradigm).
- Rich narrative, factions diplomacy, tech trees, inventory tetris — all deferred unless explicitly added later.

## Core loop (canonical)

Repeat for each squad that is available:

1. **Mission availability** appears on the world map (alerts / mission queue).
2. The squad **autonomously selects** a mission whose **mission type** the squad **can execute** (“suitable type”). Selection rules stay explicit in domain code and may later depend on traits, readiness, etc.
3. The squad **executes** the mission (abstracted while idle time advances — no player-per-action tactical grid in this vision).
4. The squad **returns to base**, bringing any payloads (resources, casualties as data, etc. — payloads are feature-specific).
5. Base state updates (e.g. resource stockpile).
6. Return to step 2.

## MVP gameplay slice (first extension after tick MVP)

Objective: smallest loop that proves **autonomous missions → base inventory**.

Rules for this MVP:

- **Exactly one mission type:** **resource gathering**.
- **Exactly one resource on the base:** **Silver** (working name «серебро»).
- Successful completion yields a **Silver** payout applied to **base warehouse stock**. The **numeric amount per mission** (fixed constant vs scaling rules) is left to the **implementation plan**, not this vision doc.
- Squad behavior: **choose → gather → return → increment base Silver → choose again**. With only one mission type, “suitable type” is trivially satisfied; the structure exists so multiple types can be added later.

### MVP non-goals

- Multiple mission types or multiple resource kinds.
- Manual mission assignment by the player.
- Mission failure states, RNG combat, logistics limits — unless separately specified.
- Saves/persistence beyond what existing MVPs specify.

## UI anchors (map / base / units)

Align with the existing panel placeholders:

| Panel   | Role in vision / MVP                                                          |
|---------|--------------------------------------------------------------------------------|
| **Map** | Surface **available missions** / alerts and high-level squad activity cues.    |
| **Base**| Show **resource stockpile** — for MVP show **Silver** count.                   |
| **Units** | Rosters / squads executing or idle; MVP can start with minimal copy.       |

Concrete layout and fidelity follow ratatui constraints and prior MVP layouts.

## Relationship to MVP-0 and MVP-1

- **MVP-0** — TUI shell, panels, clean exit.
- **MVP-1** — 1-second tick, pause, step; domain `tick(ms)` increments a counter only.
- **This vision doc** introduces the **semantic game direction** and the **first resource loop** specification. Implementing gathering, Squads, and base Silver requires a **new implementation plan** (separate from MVP-1) that builds on MVP-1’s time model.

## Future (out of MVP)

- Multiple mission types; non-trivial **suitability** rules between squad capabilities and mission type.
- Additional resources, crafts, research, failure outcomes, and richer map presentation.

## Success criteria (for the documentation deliverable)

- README and this spec give a **single consistent story**: idle UFO-like tone, no strategy layer, autonomous squads, map missions, base stockpile.
- MVP slice is **unambiguous**: one mission type (gather), one resource (Silver), loop squad → gather → base increment.
