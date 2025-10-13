# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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
  - **`exif.rs`**: Core EXIF data structures including `ExifInfo` and `LensInfo` enums (Prime vs Zoom lenses)
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
