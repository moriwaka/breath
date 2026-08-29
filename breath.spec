Name:           breath
Version:        0.1.0
Release:        1%{?dist}
Summary:        Guided breathing exercises for GNOME
License:        MPL-2.0
URL:            https://github.com/mmazzarolo/breathly-app
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo-rpm-macros
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
%cargo_build

%install
install -Dpm0755 target/release/breath %{buildroot}%{_bindir}/breath
install -Dpm0644 data/io.github.moriwaka.Breath.desktop %{buildroot}%{_datadir}/applications/io.github.moriwaka.Breath.desktop
install -Dpm0644 data/io.github.moriwaka.Breath.metainfo.xml %{buildroot}%{_metainfodir}/io.github.moriwaka.Breath.metainfo.xml
install -Dpm0644 data/icons/hicolor/512x512/apps/io.github.moriwaka.Breath.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/io.github.moriwaka.Breath.png
install -d %{buildroot}%{_datadir}/breath/audio
install -pm0644 assets/audio/*.mp3 %{buildroot}%{_datadir}/breath/audio/
install -Dpm0644 THIRD_PARTY_LICENSES/Breathly-MPL-2.0.txt %{buildroot}%{_datadir}/licenses/%{name}/Breathly-MPL-2.0.txt

%check
%cargo_test
desktop-file-validate data/io.github.moriwaka.Breath.desktop
appstreamcli validate data/io.github.moriwaka.Breath.metainfo.xml

%files
%license THIRD_PARTY_LICENSES/Breathly-MPL-2.0.txt
%{_bindir}/breath
%{_datadir}/applications/io.github.moriwaka.Breath.desktop
%{_metainfodir}/io.github.moriwaka.Breath.metainfo.xml
%{_datadir}/icons/hicolor/512x512/apps/io.github.moriwaka.Breath.png
%{_datadir}/breath/audio/

%changelog
* Sat Aug 29 2026 Breath contributors - 0.1.0-1
- Initial package
