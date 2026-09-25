Name:           syntropd
Version:        0.3.4
Release:        1%{?dist}
Summary:        Native AI Subsystem for systemd

License:        Apache-2.0
URL:            https://syntropd.github.io
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo >= 1.80
BuildRequires:  rust >= 1.80
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc

Requires:       systemd >= 252
Requires:       glibc >= 2.34
Recommends:     syntropctl

%description
syntropd integrates local AI acceleration and autonomous fault remediation
directly into systemd as native OS primitives, keeping PID 1 inviolable.
It provides zero-idle socket activation, cgroups v2 memory throttling,
and autonomous failure triage.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -D -p -m 0755 target/release/syntropd %{buildroot}%{_bindir}/syntropd
install -D -p -m 0644 units/syntrop-sockets.target %{buildroot}%{_unitdir}/syntrop-sockets.target
install -D -p -m 0644 units/syntrop-triage@.service %{buildroot}%{_unitdir}/syntrop-triage@.service
install -D -p -m 0644 tmpfiles.d/syntrop.conf %{buildroot}%{_tmpfilesdir}/syntrop.conf

install -d -m 0755 %{buildroot}%{_sysconfdir}/syntrop
install -d -m 0750 %{buildroot}%{_sharedstatedir}/syntrop
install -d -m 0775 %{buildroot}%{_sharedstatedir}/models

%post
%systemd_post syntrop-sockets.target
%tmpfiles_create_package syntropd %{_tmpfilesdir}/syntrop.conf

%preun
%systemd_preun syntrop-sockets.target

%postun
%systemd_postun_with_restart syntrop-sockets.target

%files
%license LICENSE
%doc README.md
%{_bindir}/syntropd
%{_unitdir}/syntrop-sockets.target
%{_unitdir}/syntrop-triage@.service
%{_tmpfilesdir}/syntrop.conf
%dir %{_sysconfdir}/syntrop
%dir %{_sharedstatedir}/syntrop
%dir %{_sharedstatedir}/models

%changelog
* Fri Sep 25 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.4-1
- Installer ships per-daemon ctl tools alongside the daemons.
* Fri Sep 25 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.3-1
- Installer finishes non-interactively and points at routerctl setup as the required next step.
* Fri Sep 25 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.2-1
- Release builds use thin link-time optimization with stripped symbols.
* Fri Sep 25 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.1-1
- Installer compiles missing binaries from local source checkouts.
- Triage names the failed unit in unknown-failure recommendations.
* Fri Sep 25 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.0-1
- Update umbrella release for routerd integration and tmpfiles.d runtime configuration.
* Wed Sep 24 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.1.0-1
- Initial umbrella release for Fedora and RHEL.
