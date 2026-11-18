# Bombsweeper

A tactile, GPU-rendered modern reimagining of classic Windows Minesweeper built with **Rust** and **[Rune](https://github.com/anomalyco/rune)**.

## Highlights

- **Three Curated Themes** with smooth OKLab spring transitions:
  - **Obsidian (Dark)** — Anodized dark slate chassis, tactile beveled keycap tiles, luminous jewel-toned numbers, and recessed OLED odometer counters.
  - **Alabaster (Light)** — Crisp warm porcelain chassis, sculpted pearl-stone keycaps, and high-contrast ink numbers.
  - **Classic '95** — Nostalgic Windows 95 teal backdrop, crisp 3D beveled pewter tiles, and authentic vector 7-segment red LED displays.
- **Custom GPU Vector Artwork**: Every bomb, blast starburst, pennant flag, misflag cross, expressive mascot face (Happy, Tense `:O`, Cool 😎, Singed `X_X`), and HUD icon is drawn analytically with Rune's vector `Shape`s—razor-sharp at any DPI.
- **Tactile Motion & Micro-interactions**:
  - Cascading BFS ripple reveal waves (`enter_delay` + spring scale).
  - Bouncy flag pops, press-scale keycap feel, and spring-animated rolling odometer digits.
  - Visual chord preview (holding down on a revealed number depresses its unopened neighbors and turns the mascot `:O`).
- **Thoughtful Gameplay**:
  - **First-click 3×3 safe zone** guarantee so every game opens cleanly.
  - **Three classic presets** (Beginner 9×9, Intermediate 16×16, Expert 30×16) plus a **Custom Board** configurator (6–30 cols, 6–20 rows, live bomb density).
  - **Smart Logical Hint (`H`)** powered by single-cell and pairwise subset deduction.
  - **Persistent Best Times & Statistics** (`rune::prefs::Prefs`) with 3BV/s and click efficiency metrics.
  - **Dig / Flag Mode Toggle (`M`)** and optional `?` marks for effortless trackpad play.

## Controls

| Action | Mouse | Keyboard |
|---|---|---|
| Reveal / Chord | Left-click (click revealed number to chord) | `Space` / `Enter` |
| Flag / Unflag | Right-click (or `Shift` + click) | `F` |
| Middle-Click Chord | Middle-click revealed number | — |
| Move Tile Cursor | — | `Arrow Keys` or `WASD` |
| Smart Hint | Sparkle button | `H` |
| New Game | Mascot face button | `R` / `N` |
| Switch Difficulty | Beginner / Intermediate / Expert | `1` / `2` / `3` |
| Cycle Theme | Dark / Light / 95 | `T` |
| Toggle Dig / Flag Mode | Dig / Flag button | `M` |
| Close Dialog / Clear Hint | Click outside | `Esc` |

## Build & Run

```sh
cargo run --release
```

### Tests & Headless Snapshots

```sh
cargo test
cargo run -- --scene midgame --scale 2 --snap preview.png
cargo run -- --list-scenes
```
