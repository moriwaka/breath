# Breath repository instructions

## Project

- `breath` is a native GNOME desktop breathing guide written in Rust.
- UI uses GTK4 and Libadwaita; it must not introduce a browser, Electron, or web runtime.
- The application ID is `io.github.moriwaka.Breath` and the binary is `breath`.
- User-facing text is currently Japanese, while the package metadata and source identifiers remain English.

## Repository map

- `src/lib.rs`: testable breathing domain model, built-in presets, settings types, audio mapping, and session state machine.
- `src/main.rs`: GTK4/Libadwaita application, GSettings preferences, GStreamer audio playback, and session UI.
- `tests/`: Rust integration tests for presets, session lifecycle, and settings/audio mapping.
- `data/`: desktop entry, GSettings schema, AppStream metadata, and the 512px application icon.
- `assets/audio/`: bundled MP3 guidance and completion cues.
- `breath.spec`: Fedora RPM build and file manifest.
- `vendor/` plus `.cargo/config.toml`: offline Cargo dependency source used by reproducible package builds.

## Development workflow

- Use Red → Green → Refactor for behavior changes: add a failing test, implement the smallest fix, then refactor only with tests green.
- Keep changes incremental and focused. Run `cargo fmt -- --check` and `cargo test --offline` after code changes.
- For a release build, run `cargo build --release --offline`.
- Validate desktop integration with `desktop-file-validate data/io.github.moriwaka.Breath.desktop` and `appstreamcli validate --no-net data/io.github.moriwaka.Breath.metainfo.xml`.
- Do not add web UI dependencies or remote runtime assets.

## Work files and RPM

- Use `./work/` for all temporary files, source archives, RPM build trees, logs, and generated package artifacts. Do not use `/tmp` or `/var/tmp` for project work.
- `work/` is intentionally ignored by Git. Keep generated files there and never commit them.
- Build an offline RPM from the repository root with a vendored source archive. The archive must include `vendor/` and `.cargo/config.toml`:

  ```sh
  mkdir -p work/rpmbuild/SOURCES work/rpmbuild/TMP
  tar --exclude=.git --exclude=target --exclude=work --sort=name \
    --transform='s,^\\.,breath-0.1.0,' \
    -czf work/rpmbuild/SOURCES/breath-0.1.0.tar.gz .
  rpmbuild -ba breath.spec \
    --define '_topdir %{getenv:PWD}/work/rpmbuild' \
    --define '_sourcedir %{getenv:PWD}/work/rpmbuild/SOURCES' \
    --define '_tmppath %{getenv:PWD}/work/rpmbuild/TMP'
  ```

- If the RPM toolchain still attempts to write outside `work/`, classify it as an environment/sandbox failure, preserve the output, and report the unverified step rather than claiming RPM success.

## Git and safety

- Inspect `git status --short` before editing and commit only intended source, test, metadata, and instruction changes.
- Do not delete or reset user files. Ignore unrelated generated files.
- Before reporting success, verify the final command for that claim actually passed.
