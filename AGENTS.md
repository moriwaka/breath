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
- `.github/workflows/ci.yml`: Fedora-container CI for Rust, metadata, and RPM checks.
- `vendor/` plus `.cargo/config.toml`: offline Cargo dependency source used by reproducible package builds.

## Development workflow

- Use Red → Green → Refactor for behavior changes: add a failing test, implement the smallest fix, then refactor only with tests green.
- Keep changes incremental and focused. Run `cargo fmt -- --check` and `cargo test --offline` after code changes.
- Run `cargo clippy --all-targets --offline -- -D warnings` when Clippy is installed; Fedora CI runs the same check.
- For a release build, run `cargo build --release --offline`.
- Validate desktop integration with `desktop-file-validate data/io.github.moriwaka.Breath.desktop` and `appstreamcli validate --no-net data/io.github.moriwaka.Breath.metainfo.xml`.
- The UI uses Japanese by default and shows English when a locale beginning
  with `en` is present in `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, or `LANG`.
- Run `python3 tests/screenshot_regression.py` in a GNOME session with
  `gnome-screenshot` installed for native screen captures; it exits 77 when
  the capture utility is unavailable.
- Do not add web UI dependencies or remote runtime assets.

## Versioning

- Any change to application behavior or user-visible functionality must bump
  the minor version (for example, `0.1.0` to `0.2.0`).
- Keep the version synchronized in `Cargo.toml`, `breath.spec`, the AppStream
  release metadata, and `CHANGELOG.md`.
- Use only the RPM `Release` field for rebuilds that do not change behavior.

## Work files and RPM

- Use `./work/` for all temporary files, source archives, RPM build trees, logs, and generated package artifacts. Do not use `/tmp` or `/var/tmp` for project work.
- `work/` is intentionally ignored by Git. Keep generated files there and never commit them.
- `.gitattributes` preserves CRLF for two vendored fixture files whose recorded
  Cargo checksums depend on those line endings; do not normalize those files.
- Build an offline RPM from the repository root with a vendored source archive. The archive must include `vendor/` and `.cargo/config.toml`:

  ```sh
  mkdir -p work/rpmbuild/SOURCES work/rpmbuild/TMP
  tar --exclude=.git --exclude=target --exclude=work --sort=name \
    --transform='s,^\\./,breath-0.3.0/,' \
    -czf work/rpmbuild/SOURCES/breath-0.3.0.tar.gz .
  rpmbuild -ba breath.spec \
    --define '_topdir %{getenv:PWD}/work/rpmbuild' \
    --define '_sourcedir %{getenv:PWD}/work/rpmbuild/SOURCES' \
    --define '_tmppath %{getenv:PWD}/work/rpmbuild/TMP'
  ```

- If the RPM toolchain still attempts to write outside `work/`, classify it as an environment/sandbox failure, preserve the output, and report the unverified step rather than claiming RPM success.

## Git and safety

- Inspect `git status --short` before editing and commit only intended source, test, metadata, and instruction changes.
- At each coherent, verified milestone, commit the focused changes before
  starting the next slice. Keep generated files under `work/` out of commits.
- Before a milestone commit, run `git diff --check`, inspect the staged diff,
  and verify the command that justifies the milestone actually passed.
- Do not delete or reset user files. Ignore unrelated generated files.
- Before reporting success, verify the final command for that claim actually passed.
- Batch local changes and verify them before pushing. A local commit may remain
  ahead of `origin/main` while investigation is still in progress.

## Verified milestone

- RPM build, RPM metadata/file checks, and the installed-package AT-SPI smoke
  test have been completed once. Re-run them after changing the source or SPEC;
  a same-version RPM must be reinstalled explicitly to test the new binary.

## Tasks that require running outside the sandbox

- Launching the GTK application against the user's real Wayland/X11 display.
  A sandbox may expose `DISPLAY` or `WAYLAND_DISPLAY` but still deny the
  display connection.
- Running AT-SPI tests such as `tests/ui_smoke.py`. They need the user's
  session D-Bus and accessibility bus; run them in the GNOME session, not in a
  filesystem-only sandbox.
- Checking real audio output, keyboard focus, window sizing, dark theme, and
  other compositor-dependent behavior.
- Installing or updating the system package with `sudo rpm -Uvh ...`; this
  writes to `/usr` and requires the user's password. Package inspection with
  `rpm -qpl`, `rpm -qp --requires`, and `rpm -V` does not normally need root.
- Run `rpm -V` for an installed system package outside the filesystem sandbox.
  The sandbox can report synthetic file ownership and permissions, so its
  result is not reliable for validating the real installation.
- Running package post-install checks that update system-wide caches, such as
  GSettings schema or icon caches, when they are not already handled by the
  package manager.

When one of these checks is unavailable in the sandbox, report the exact
environment limitation and do not claim the check passed. Keep project build
trees, logs, and generated packages under `./work/` even when the command is
run outside the sandbox.
