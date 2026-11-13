# Orbit Club

A Rust + raylib space arcade built on the original flying solar system and ship. The **ship and planets remain 3D**; the characters use the **original 2D chibi illustrations**, with an updated astronaut, **Fer**. All five minigames are played against a CPU.

## Launch

Double-click **Launch Game.command**, or open **dist/Orbit Club.app**. Build the application bundle with `./scripts/bundle-macos.sh` first. The resulting application is for Apple Silicon macOS and plays offline without Rust installed. The launcher opens a fresh instance of the latest build.

1. The hub starts in third-person free flight. Fly with **W/S**, turn with **A/D or arrows**, and hold **right mouse** to steer. Click a planet, radar dot, host card, or press **1–5** to mark a destination. **G** or **Warp** starts optional autopilot.
2. Fly near a planet, then press **Enter** or **Play** to launch its game. Manual controls release any planet-relative orbit immediately.
3. Press **H** inside a minigame to return to your ship.
4. Beat each CPU game to earn its victory star. Visited worlds and five victory stars persist between launches; earlier progress is preserved.

| Key | Planet | Illustrated host | CPU game |
| --- | --- | --- | --- |
| 1 | Cinder | Ember, orange axolotl pilot | Blast Orbit |
| 2 | Relic | Bleep, space robot | Orbit Beat |
| 3 | Aurelia | Captain Truffle, space pig | Quoridor |
| 4 | Prism | Lumen, faceless purple star mage | Chess |
| 5 | Bloom | Mochi, green alien cat | Connect Four |

## Illustrated cast

The pig, cat, axolotl, and purple star retain their original illustrations. Bleep is an illustrated chibi space robot replacing the yellow star, with a matching six-expression sprite sheet, circular portraits, idle bob, celebration bounce, and nervous tremble. Fer uses updated astronaut artwork. Fer has warm light-medium skin, chestnut waves with curtain bangs and lighter ends, translucent brown glasses, a white-and-teal spacesuit, a jellyfish emblem, and a **small mole on her chin**.

Fer has a separate 2-column × 3-row sheet so the original character atlas stays intact. All character sheets are embedded in the executable. The original host sheet is `assets/characters/orbit-cast-atlas-v2.png`; Fer's updated sheet is `assets/characters/fer-expressions-v1.png`; Bleep uses `assets/characters/bleep-expressions-v1.png`. Bleep's exact imagegen prompt is in `assets/characters/BLEEP_ART_DIRECTION.json`. The source photos are not redistributed.

The six expressions are **idle, thinking, nervous, happy, sad, and surprised**. Circular face portraits and full-body illustrations react to the actual game state: Truffle worries when your finish path is short, Mochi reacts to four-in-a-row threats, Lumen reacts to check and mating threats; Bleep gets nervous during long combos, and Ember reacts to hostile bomb paths. Moves can trigger surprise; winning and losing trigger celebration and sadness. Nervous poses tremble slightly and idle poses gently bob.

Press **F2** in the hub for the **Character Gallery**. Use **1–6** or Tab to choose a character, Space to cycle expressions, or click the buttons. **H** or Escape returns to the hub.

The 3D-character experiment is preserved locally in `legacy-characters-3d/`, excluded from the current release. Its models and viewer are not used by this version.

## Hub controls

| Input | Action |
| --- | --- |
| 1–5 / click a planet, radar dot, or host card | Mark a destination |
| G / Warp button | Optional warp autopilot |
| Enter | Play after docking |
| Tab | Switch map / third-person ship view |
| W / S | Forward / reverse thrust |
| A / D or Left / Right | Turn the ship |
| Up / Down | Aim the nose |
| Q / E | Descend / ascend |
| Z / C | Strafe |
| R | Face the selected destination |
| Shift / X | Boost / brake |
| O | Toggle local orbit |
| Right-drag | Steer in flight; rotate the map |
| Mouse wheel | Adjust chase-camera distance / zoom map |
| F1 / F2 | Flight guide / Character Gallery |
| F | Fullscreen |

Manual movement cancels autopilot without teleporting. The ship banks when turning, collides with the sun and planets, and can follow a planet in local orbit. Planets retain their different orbital speeds.

## Minigames

**Quoridor:** the previous 9×9 3D planet board, with the original ship pawns. Reach any planet on the opposite edge first. Each turn, move or place one of ten barriers. Barriers block two connections, cannot cross or overlap, and must leave both ships a path to their finish. Straight and blocked-jump side-step rules are supported. Click glowing destinations; **B** places barriers, **R/Space** rotates, **M** moves, **Z** undoes the round, **V** resets the camera, **F1** opens rules, and **F12** saves a screenshot. CPU difficulty: Casual, Club, or Pro.

**Connect Four:** on Bloom, play ringed planets against Mochi's ocean planets. Click a column or press **1–7**. Four matching discs horizontally, vertically, or diagonally wins; a filled board without a line draws. Discs fall with an animation, and the CPU checks wins/blocks and searches six plies. **Z** undoes the round, **R** rematches, **F1** pauses for rules.

**Chess:** play White against the host's blue side. Click a piece and a legal destination. Check, castling, en passant, and all four promotions are supported. The CPU searches three plies. Checkmate wins; stalemate, recognized insufficient material, repetition, and the 50-move rule draw automatically for casual play. **Z** undoes the round, **R** rematches, **F1** opens rules and pauses the CPU.

**Blast Orbit:** play Fer against Ember in a bomb-placement arena. **WASD/arrows** move, **Space** places a bomb, **P/Escape** pauses, **R** rematches. Each pilot can have two active bombs with 2.2-second fuses. Cross-shaped blasts break crates, stop at stone, and trigger chain reactions. Green pickups extend range. Survive while the CPU is caught in a blast to win; simultaneous knockouts or the two-minute limit draw.

All games provide a **Return to hub** button and **H** shortcut. Leaving an unfinished match abandons it. Matches are not saved between visits; progress stars are saved when you return after a win.

## Build and verify

Requires Rust 1.88+, CMake, and raylib's native development tools; on macOS install Xcode Command Line Tools. The first source build may download dependencies. Raylib is pinned to 6.0.0. The internal Cargo package/binary name remains `crownfall` for compatibility.

```sh
cargo run --release --locked
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
./scripts/bundle-macos.sh
```

Development modes:

```sh
cargo run --release --locked -- --render-characters /absolute/output/folder
cargo run --release --locked -- --render /absolute/output/folder
cargo run --release --locked -- --gallery
cargo run --release --locked -- --simulate
cargo run --release --locked -- --quoridor
cargo run --release --locked -- --chess
cargo run --release --locked -- --connect4
cargo run --release --locked -- --blast
cargo run --release --locked -- --beat
```

The art export uses the actual in-game sprite sheets and circular portrait crops without opening a window. Native screen capture mode, `--capture-suite /absolute/output/folder`, requires an interactive desktop. Gameplay/rules, CPU legality, move reactions, flight/docking, progress migration, sprite layout, transparent assets, and portrait masks are checked by the automated tests. The original host atlas is also compared byte-for-byte with the previous release. Native input automation was not repeated because the desktop tool stalled during an earlier release.

Progress is stored at `~/Library/Application Support/Orbit Club/progress.txt` on macOS and `~/.local/share/orbit-club/progress.txt` elsewhere. Quoridor screenshots go to `dist/snapshots/` from the app bundle or `snapshots/` from a source build.

## Source and credits

`src/characters.rs` renders the illustrated cast; `src/studio.rs` is the expression gallery; `src/reactions.rs` connects portraits to the rules state. The flight controller, 3D math/rasterizer, original ship OBJ, five planet families, Quoridor board, chess, Connect Four, and bomb arena remain in the existing Rust modules. The old prototypes are preserved in `legacy-*` folders and excluded from the compact release.

Fer's new illustration was made with the built-in image_gen tool. Exact prompts and asset information are recorded in `assets/characters/FER_ART_DIRECTION.json`. The original cast illustrations and their prompts remain in `assets/characters/`.

Quoridor was designed by Mirko Marchesi and published by Gigamic: [publisher's rules](https://export.gigamic.com/wp-content/uploads/2021/01/INS-RULES-QUORIDOR_10-2016.pdf). Chess follows the [FIDE move rules](https://handbook.fide.com/chapter/e012023), with the casual automatic-draw behavior described above. Blast Orbit is an original game inspired by the bomb-placement genre. No commercial game artwork is redistributed. Raylib/raylib-rs notices remain in `third-party/`; the supplied ship belongs to its respective creator.

## Space arcade update
Relic now hosts Bleep, a space robot, and Orbit Beat: a 96-note paddle rhythm duel against a speed-limited CPU, with synthesized original percussion and hit sounds at 120 BPM. Move the mouse vertically; R restarts and H returns to the hub. Inspired by the paddle/rhythm gameplay described by Nintendo: https://www.nintendo.com/en-gb/Games/WiiWare/BIT-TRIP-BEAT-284567.html . No original game's artwork or audio is used.

Connect Four uses ringed and ocean planet tokens. Prism retains chess with lit, projected 3D space miniatures: scout orbs, rockets, ringed planets, satellites, solar crowns and command beacons. The chess backdrop is a starfield. Blast Orbit uses astronaut board markers and classic dark bombs with lit fuses. In free flight, hold the right mouse button over the viewport and drag to turn the ship; in overview drag rotates the system view.

Hub: Left/Right arrows turn the ship; Up/Down tilt its nose. W/S thrust follows the heading. Tab opens the map and returns to flight at the same position and heading. The chase camera keeps a level horizon, avoids entering planets, and widens slightly at speed. Free flight has no invisible atlas boundary.

Autopilot travel includes a warp-drive effect with animated radial star trails and a smooth entry/exit envelope. It stops when travel is cancelled or docking completes.
