# ICE — a Rust RPG engine

ICE is a role-playing game engine written in Rust. It aims for a classic
Fallout-like feel: a character moving cell by cell across a tile map, where the
world is a *place that remembers what you did* rather than a backdrop that
revolves around you.

The project is in early **alpha**. Each milestone lives in its own version
directory (`v0_01`, `v0_02`, …) with a document explaining what changed, how
the code works, and where the project goes next.

## Current state

| Version | Contents |
|---|---|
| `v0_01` | Design notes: the technology plan ([v0_01/v0_01.md](v0_01/v0_01.md)). |
| `v0_02` | First runnable engine: a map, an entity, and cell-by-cell movement, drawn in a window ([v0_02/v0_02.md](v0_02/v0_02.md)). |

## The stack

- **[Bevy](https://bevy.org)** — the rendering and ECS skeleton (the window,
  sprites, input).
- **[gametools](https://docs.rs/gametools)** — the spatial/math backbone: grid
  storage, coordinates, and (later) pathfinding, field-of-view, dice, bounded
  resources, and initiative.
- **dramaturge** (planned, v0.03) — the narrative/dialogue brain: characters,
  facts, memories, and conditional dialogue.

See [v0_01/v0_01.md](v0_01/v0_01.md) for the full design rationale.

## Layout

```
ice-dev_rust-rpg/
├─ crates/
│  ├─ engine/   # library crate — all game rules, no rendering. Unit-testable.
│  └─ game/     # binary crate — the Bevy shell that draws `engine`'s world.
├─ v0_01/       # design doc
└─ v0_02/       # v0.02 doc (what changed + how it works + roadmap)
```

The rule of thumb is **logic in `engine`, presentation in `game`**. The `game`
crate owns no game rules — it asks `engine` where things are and forwards input
back to `engine`. That split is what makes the rules testable with plain
`cargo test` and readable without a game engine in the way.

## Prerequisites

- Rust `1.97.1` or newer (`rustup` is the usual way to install it).
- On Windows, nothing else — Bevy's `wgpu` picks DX12/Vulkan automatically.

## Run it

```sh
# Build and open the game window (arrow keys or WASD move the green square)
cargo run -p game

# Run the engine's unit tests
cargo test

# Lint
cargo clippy --all-targets -- -D warnings

# Generate local API docs
cargo doc --no-deps
```

## Learning to read the code

If you know some programming fundamentals and want to learn by reading, start
with `crates/engine/src/map.rs`, then `position.rs`, `entity.rs`, and
`world.rs` — in that order. Each file has module-level and item-level comments
written to explain not just *what* but *why*. `v0_02/v0_02.md` walks through the
same material as a guided tour.
