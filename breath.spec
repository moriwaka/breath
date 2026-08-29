Name:           breath
Version:        0.5.0
Release:        1%{?dist}
Summary:        Guided breathing exercises for GNOME
License:        MIT and MPL-2.0
URL:            https://github.com/moriwaka/breath
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  desktop-file-utils
BuildRequires:  gtk4-devel
BuildRequires:  libadwaita-devel
BuildRequires:  gstreamer1-devel
BuildRequires:  appstream

%description
Breath is a native GNOME application for guided breathing exercises.

%prep
%autosetup

%build
CARGO_TARGET_DIR=%{_builddir}/breath-cargo-target cargo build --release --offline

%install
install -Dpm0755 %{_builddir}/breath-cargo-target/release/breath %{buildroot}%{_bindir}/breath
install -Dpm0644 data/io.github.moriwaka.Breath.desktop %{buildroot}%{_datadir}/applications/io.github.moriwaka.Breath.desktop
install -Dpm0644 data/io.github.moriwaka.Breath.metainfo.xml %{buildroot}%{_metainfodir}/io.github.moriwaka.Breath.metainfo.xml
install -Dpm0644 data/io.github.moriwaka.Breath.gschema.xml %{buildroot}%{_datadir}/glib-2.0/schemas/io.github.moriwaka.Breath.gschema.xml
install -Dpm0644 data/icons/hicolor/512x512/apps/io.github.moriwaka.Breath.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/io.github.moriwaka.Breath.png
install -Dpm0644 data/breath.1 %{buildroot}%{_mandir}/man1/breath.1
install -d %{buildroot}%{_datadir}/breath/audio
install -pm0644 assets/audio/*.mp3 %{buildroot}%{_datadir}/breath/audio/
install -Dpm0644 THIRD_PARTY_LICENSES/Breathly-MPL-2.0.txt %{buildroot}%{_datadir}/licenses/%{name}/Breathly-MPL-2.0.txt
install -Dpm0644 LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE

%check
CARGO_TARGET_DIR=%{_builddir}/breath-cargo-target cargo test --offline
desktop-file-validate data/io.github.moriwaka.Breath.desktop
appstreamcli validate --no-net data/io.github.moriwaka.Breath.metainfo.xml

%files
%defattr(-,root,root,-)
%license LICENSE
%license THIRD_PARTY_LICENSES/Breathly-MPL-2.0.txt
%doc README.md CHANGELOG.md
%{_bindir}/breath
%{_datadir}/applications/io.github.moriwaka.Breath.desktop
%{_metainfodir}/io.github.moriwaka.Breath.metainfo.xml
%{_datadir}/glib-2.0/schemas/io.github.moriwaka.Breath.gschema.xml
%{_datadir}/icons/hicolor/512x512/apps/io.github.moriwaka.Breath.png
%{_mandir}/man1/breath.1*
%{_datadir}/breath/audio/

%changelog
* Sat Aug 29 2026 Breath contributors - 0.5.0-1
- Move breathing pattern selection into Preferences and remember the choice
- Distinguish the two hold phases with different guide circle sizes
- Add English UI translations and screenshot regression harness
