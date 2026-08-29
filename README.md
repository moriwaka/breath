# Breath

Breath is a native GNOME breathing exercise application written in Rust with
GTK4 and Libadwaita. GStreamer provides optional guidance audio.

## Development

```sh
cargo fmt -- --check
cargo test --offline
cargo build --release --offline
```

To run from the checkout, compile the GSettings schema into `work/` first:

```sh
mkdir -p work/gsettings
glib-compile-schemas --targetdir work/gsettings data
GSETTINGS_SCHEMA_DIR="$PWD/work/gsettings" cargo run --offline
```

The application includes seven breathing patterns, configurable session length,
optional Paul/Laura/bell guidance audio, a three-second countdown,
pause/resume/stop controls, and Escape/back navigation.

## Fedora RPM

The SPEC file builds offline using the vendored Cargo dependencies. Keep all
temporary files and generated packages under `work/`:

```sh
mkdir -p work/rpmbuild/SOURCES work/rpmbuild/TMP
tar --exclude=.git --exclude=target --exclude=work --sort=name \
  --transform='s,^\\.,breath-0.1.0,' \
  -czf work/rpmbuild/SOURCES/breath-0.1.0.tar.gz .
rpmbuild -ba breath.spec \
  --define "_topdir $PWD/work/rpmbuild" \
  --define "_sourcedir $PWD/work/rpmbuild/SOURCES" \
  --define "_tmppath $PWD/work/rpmbuild/TMP"
```

Install the package with:

```sh
sudo rpm -Uvh --replacepkgs work/rpmbuild/RPMS/x86_64/breath-0.1.0-1*.rpm
```

## UI smoke test

`tests/ui_smoke.py` uses AT-SPI and does not require a browser. Run it inside
an active GNOME session with the accessibility bus available:

```sh
python3 tests/ui_smoke.py
# Or test the current checkout instead of the installed package:
python3 tests/ui_smoke.py target/debug/breath
```

It checks the home screen, a preset start action, the `3`, `2`, `1` countdown,
and session controls. It exits with code 77 when AT-SPI is unavailable.

## License and attribution

The Breath application source code is licensed under the MIT License; see
[`LICENSE`](LICENSE).

The bundled voice and bell audio resources are adapted from the
[Breathly app](https://github.com/mmazzarolo/breathly-app). They are included
under the Mozilla Public License 2.0. The original license text is preserved
in [`THIRD_PARTY_LICENSES/Breathly-MPL-2.0.txt`](THIRD_PARTY_LICENSES/Breathly-MPL-2.0.txt).
The Breathly project and its contributors retain the copyright to those
resources.
