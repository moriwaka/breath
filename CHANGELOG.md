# Changelog

## 0.6.0

- Show full descriptions and timing details for breathing pattern choices.
- Remove the redundant duration prefix from the Deep Calm pattern name.

## 0.5.0

- Move breathing pattern selection into Preferences.
- Remember the selected pattern and provide a home-screen Start button for it.

## 0.4.0

- Distinguish the hold after inhaling from the hold after exhaling with
  different guide circle sizes.

## 0.3.0

- Added English UI translations, selected automatically for English locales.
- Added a Wayland screenshot regression test harness.

## 0.2.0

- Show a user-visible warning when guidance audio cannot be played.
- Keep the completion cue silent when audio mode is set to Off.

## 0.1.0

- Added a native GTK4 and Libadwaita breathing guide for GNOME.
- Added seven built-in breathing patterns based on the Breathly-inspired
  session model.
- Added a three-second preparation countdown and animated inhale/exhale guide.
- Added pause, resume, stop, back, and Escape navigation controls.
- Added optional Paul, Laura, bell, and silent guidance modes.
- Added Fedora RPM packaging with vendored offline Cargo dependencies.
- Added Rust domain tests and an AT-SPI UI smoke test.
