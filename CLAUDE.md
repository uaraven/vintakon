# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.


## 1. Think before coding

Don't assume. Don't hide confusion. Surface tradeoffs.

- State assumptions explicitly; if uncertain, ask.
- Multiple interpretations → present them, don't pick silently.
- Simpler approach exists → say so. Push back when warranted.
- Unclear → stop, name what's confusing, ask.

## 2. Simplicity first

Minimum code that solves the problem. Nothing speculative.

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility"/"configurability" that wasn't requested.
- No error handling for impossible scenarios.
- 200 lines when 50 suffice → rewrite.

Test: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical changes

Touch only what you must. Clean up only your own mess.

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

Test: every changed line traces directly to the user's request.

## 4. Fix don't silence

Never disable linter/typechecker warnings with comments. Fix the actual issue.

- No `/* eslint-disable */`, `@ts-ignore`, `@ts-expect-error`, `# type: ignore`, or similar.
- Fix the underlying issue instead:
  - **Missing globals?** Add env globals to config (e.g. `globals.browser` for Playwright CT tests, `globals.node` for config files).
  - **Unused variable?** Remove it, or prefix with `_` if intentionally unused per interface contract.
  - **Type mismatch?** Fix the types or add proper type annotations.
  - **Wrong pattern?** Refactor to follow the expected pattern.
- Only exception: a third-party library has a known incorrect type definition AND the fix is filed as an upstream issue. Add a comment linking to the issue.

Why: disable comments hide real problems, rot, and create tech debt. The fix is usually trivial once you understand what the tool is telling you.

## 5. Goal-driven execution

Define success criteria. Loop until verified.

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## 6. Subagent dispatch discipline

Verify tools, fail fast, always have a fallback. Parent agent is responsible for noticing and unblocking permission failures.

**Before dispatching:**

- Identify which tools the subagent needs (WebSearch, WebFetch, Bash with network, Read, etc.).
- If the task depends on tools often sandboxed away (WebSearch/WebFetch, npm/curl via Bash), tell the subagent: "If a required tool is denied, stop immediately and return a one-line permission report — do not loop retrying."

**While subagent is running:**

- On "permission denied", "tool unavailable", or retry-loops on the same denied tool: take over from the parent session — don't wait it out.
- Check long-running subagents' output files after a reasonable window. Use `wc -l` and `tail -n 5 | jq` on the JSONL file to see recent activity without reading the whole transcript.
- Prefer stopping a stuck subagent over letting it burn tokens. Kill with `TaskStop` (or move on and leave it dangling) and do the lookup yourself.

**Always have a fallback:**

- In-distribution knowledge first for well-known libraries (React, Vite, Bun, Hono, commander, etc.).
- Direct Bash from the parent session for niche packages (`npm view <pkg>`, `npm pack` + inspect, `curl -sS`).
- Flag fallback as "verify in implementation phase" — don't pretend the assumption is confirmed.

**Never do:**

- Silently proceed while a subagent is stuck — name it, take the work, keep moving.
- Pretend a subagent's partial output is complete when it bailed on permissions.
- Retry the same denied tool from a subagent — denial is usually the subagent's sandbox policy, not transient.


# Vintakon

GUI application to update Lens exif information for the pictures taken with vintage lenses.

Vintakon uses `exiftool` for reading and writing exif info.

To read info exiftool is called with following parameters:

```
-j
-b
-q
-api
structformat=jsonq
-FileModifyDate
-Make
-Model
-LensMake
-LensModel
-ExposureCompensation
-LensInfo#
-ISO
-Aperture
-ShutterSpeed
-FocalLength
-FocalLengthIn35mmFormat
-previewimage
-JpgFromRaw
-ThumbnailImage
```

User can define lens profiles that can be applied to files
Lens profile contains following information:

- lens make
- lens model;
- use aperture;
- lens focal length;
- lens focal length in 35mm equivalent
- lensInfo (f/min - f/max focalLengthMin - focalLengthMax);

## Testing

Test data is in `test-data` folder

To run tests execute

```shell
cargo test
```

Do not assume that failing tests are incorrect. First try to fix them.

## UI

UI is built using iced library.

## Development Commands

Always make sure to compile the project after making changes. Project must compile.

### Building and Running

```shell
# Build the project
cargo build

# Run the application
cargo run

# Build for release
cargo build --release
```

### Testing

```shell
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run a specific test
cargo test test_parsing_prime
```

### Code Quality

```shell
# Format code
cargo fmt

# Check code without building
cargo check

# Run clippy lints
cargo clippy
```

## Architecture

### Core Modules

- **`main.rs`**: Application entry point, sets up the Iced GUI application with window settings and app icon
- **`model/`**: Contains application state and data structures
  - **`state.rs`**: Main application state (`VintakonApp`) with UI event handling and async operations
  - **`file_rec.rs`**: File record wrapper around EXIF data with selection state
- **`exif/`**: EXIF data processing and lens information parsing
  - **`exiftool.rs`**: Interface to `exiftool` command-line utility, handles JSON parsing and image preview extraction
  - **`info.rs`**: Core EXIF data structures including `ExifInfo` and `LensInfo` enums (Prime vs Zoom lenses)
- **`resources.rs`**: Embedded application resources (app icon)

### Key Data Flow

1. User selects folder via file dialog (`state.rs:select_folder()`)
2. Application calls `exiftool` to extract EXIF data from supported image formats (`exiftool.rs:read_files()`)
3. Raw JSON output is parsed into `ExifInfo` structures with lens information categorization
4. `FileRec` wrappers are created to track selection state for batch operations
5. Preview images are decoded from base64 embedded data when available

### EXIF Processing

The application uses `exiftool` with specific parameters to extract comprehensive metadata. Key features:

- Supports 20+ image formats including RAW files (ARW, NEF, RAF, etc.)
- Extracts lens information and categorizes as Prime or Zoom lenses
- Decodes embedded preview images from base64 data
- Handles Sony limitations where certain tags (like FocalLength in maker notes) cannot be modified

### Dependencies

- **iced**: Cross-platform GUI framework with async support
- **serde/serde_json**: JSON parsing for exiftool output
- **chrono**: Date/time handling for file timestamps
- **image**: Image processing and preview generation
- **base64**: Decoding embedded preview images
- **rfd**: Native file dialog integration
- **tokio**: Async runtime for non-blocking operations

### External Dependencies

- **exiftool**: Must be installed and available in PATH for metadata operations
