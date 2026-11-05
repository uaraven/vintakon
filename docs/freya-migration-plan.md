# Plan: migrate Vintakon from iced 0.13 to Freya

Based on an inventory of the current iced code and research into Freya 0.4.3 (as of 2026-11-04). Items marked "spike" are unverified in the Freya docs and must be checked against the Freya repo examples before coding.

## Decisions

- **Freya version:** pin `freya = "=0.4.3"`. The 0.5 line is at rc.13 and changes roughly weekly. Move to 0.5 once it is stable.
- **Async:** keep tokio. Build the runtime in `main()` and hold `rt.enter()`, instead of using `#[tokio::main]`. Run exiftool work on `spawn_blocking`. Bring results back into the UI with Freya's `spawn` plus a `oneshot` or the `JoinHandle`.
- **Dialogs:** keep rfd 0.15. Start with `AsyncFileDialog` inside Freya's `spawn`. If it misbehaves on Linux, fall back to sync `FileDialog` on `spawn_blocking`.
- **Framework-independent code stays as is:** `src/exif/*`, `src/model/lens_profile.rs`, `src/model/file_rec.rs`, and the PNG assets. The tests in `lens_profile.rs` are unaffected.

## Phase 0: groundwork and spikes

1. Install the Linux build dependencies (Fedora: `openssl-devel clang-devel systemd-devel gtk3-devel`). Expect a slow first Skia build.
2. Swap `iced` for `freya` in `Cargo.toml`. Keep `image`, `tokio`, `rfd`, `chrono`, `serde`, `serde_json`, `base64` and `regex`.
3. Run small throwaway spikes for the unverified Freya items:
   - Show an `image()` from `ImageHandle::from_rgba`, and find the `SkAlphaType` import path. Also try `ImageViewer` with the raw preview JPEG bytes as an alternative.
   - Call the rfd async folder dialog from `spawn`.
   - Set a pointer cursor on a plain rect.
   - Read the `KeyboardEventData` fields and check whether Tab focus traversal is built in.
   - Check which components expose tooltips and a disabled state.
   - Check how to size the two content columns in a 1:2 ratio (`Size::percent` / flex).

## Phase 1: split logic from UI

`state.rs` is 1108 lines mixing logic and view code.

1. Move the framework-free pieces out of `state.rs` into plain modules:
   - Validators (`^(\d+(\.\d*)?)?$` and `^\d*$`), `can_apply_profile`, `find_profile_by_name`, `is_any_file_checked`.
   - `load_images`, `select_folder`, `select_exiftool_file`, `apply_profile_to_images`.
2. Replace the iced `Message` enum and `update()` with an `AppState` struct of Freya `State<T>` values (Copy, so they can be passed to child components). Fields match `VintakonApp`: `path`, `images`, `is_loading`, `is_applying`, `selected_file_index`, `config`, `profile_names`, `selected_profile`, `apply_aperture`, `apply_focal_length`, `show_settings`, `temp_config`, `selected_profile_index`.
3. Share `AppState` through context (`use_provide_context` and `use_consume`).
4. Drop `focused_input` (dead code) and the `Event` subscription for Tab handling, if the spike shows Freya handles Tab itself.

## Phase 2: UI components

Each `view_*` function becomes a component, with the same layout, sizes and spacing as today.

| iced | Freya |
|---|---|
| `Column` / `Row` | `rect().vertical()` / `.horizontal()` with `.spacing()` and `.padding()` |
| `FillPortion(1)` / `FillPortion(2)` (file list / preview) | flex sizing or `Size::percent` (per spike) |
| `scrollable` | `ScrollView` (file list could use `VirtualScrollView` later if rows get a fixed height) |
| `text(..)` | `label()` or a plain string child |
| `button(image)` with tooltip | `Button::new().flat()` wrapping `image()`, inside a `TooltipContainer` |
| `checkbox` | `Checkbox` with `Tile::on_select` |
| `text_input` with validation | `Input` over a `State<String>`, using `on_validate` for the two regexes |
| `pick_list` | `Select` with `MenuItem`s |
| `Rule::horizontal` | thin `rect` (height 2-5 px) with a grey background |
| settings screen | keep as a full-window replacement (a `Popup` could be tried later) |

Build order:

1. `main.rs` with the window config: title "Vintakon", size 1200x800, min size 1200x800, the app icon through `LaunchConfig::window_icon`, dark theme, `with_app_id("vintakon")`.
2. Header: path label and open-folder button.
3. Content pane with its four states: applying, loading, empty and populated.
4. File list and rows: checkbox, name, date, and a "bad lens" icon. The selected row gets a primary-colored background, and the other rows are clickable.
5. Preview and metadata. Convert `DynamicImage` to an `ImageHandle` once per selection with `use_memo` (iced rebuilds it on every `view()`).
6. Apply pane: profile select, aperture and focal-length inputs, apply button (enabled only when `can_apply_profile`), settings button.
7. Settings screen: exiftool path with a browse button, profile list with add and delete, profile editor, Cancel and Ok buttons. Keep the same validation and parse-with-fallback behavior.
8. Match colors, spacing and sizes as closely as possible. Pixel-identical output is not realistic (fonts will differ).

## Phase 3: wire up async flows

1. Folder select: dialog, then load, with `is_loading` set around it.
2. Apply: set `is_applying`, run `apply_profile_to_images`, then reload the folder.
3. Exiftool path browse in the settings screen.
4. `load_images` is an async fn that blocks inside `read_files`. Wrap it in `spawn_blocking`.

## Phase 4: cleanup and verification

1. Delete the iced code, unused imports and the old `Message` enum.
2. Run `cargo build`, `cargo test`, `cargo clippy` and `cargo fmt`. Run the app against `test-data`.
3. Compare behavior screen by screen against `main`: the 4 content states, selection, apply, and settings add/delete/save.
4. Check the macOS bundle (the plist exists) and Windows builds.
5. Update CLAUDE.md and the README (UI framework and build dependencies).

## Behaviors to keep or fix on purpose

- **Keep:** the settings screen replaces the whole window, and the file-row selection style stays.
- **Fix:** cancelling the folder dialog currently triggers a load of path `""`.
- **Fix:** `apply_profile_to_images` uses `parse().unwrap()` on the focal length.
- **Check:** typing `5.` in a numeric field reformats because it round-trips through `f32`. The Freya `Input` should keep the raw string while editing.

## Risks

- Freya 0.4 is young. There is no known-working rfd example, and several event and image API details are unverified (see Phase 0).
- Skia makes the first build slow and the binary bigger.
- The 0.4 to 0.5 migration will touch `spawn`, `Platform` and the style traits (`spawn_forever` split, `Platform`/`PlatformWindow` split, `StyleExt` split), so isolate those calls in a few places.
- Exact visual match is not achievable, so "as similar as possible" means the same layout and behavior.

## References

- https://docs.rs/freya/0.4.3/freya/
- https://docs.rs/freya/0.4.3/freya/components/index.html
- https://docs.rs/freya/0.4.3/freya/_docs/tokio_integration/index.html
- https://docs.rs/freya/0.4.3/freya/_docs/development_setup/index.html
- https://github.com/marc2332/freya/tree/main/examples
