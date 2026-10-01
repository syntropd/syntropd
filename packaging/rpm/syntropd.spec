Name:           syntropd
Version:        0.3.27
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
install -D -p -m 0755 target/release/syntrop %{buildroot}%{_bindir}/syntrop
ln -sf syntrop %{buildroot}%{_bindir}/syn
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
%{_bindir}/syntrop
%{_bindir}/syn
%{_unitdir}/syntrop-sockets.target
%{_unitdir}/syntrop-triage@.service
%{_tmpfilesdir}/syntrop.conf
%dir %{_sysconfdir}/syntrop
%dir %{_sharedstatedir}/syntrop
%dir %{_sharedstatedir}/models

%changelog
* Thu Oct 01 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.27-1
- Dynamic Memory & Elasticity production hardening: Production generate routed via decode_loop_managed with zero-budget short-circuit and empty sequence guard in runtimed (0.5.13); GetDrmWatermark and ResizeLease Varlink specs synchronized with device parameter examples and CLI profile path resolution in inferenced (0.3.9); resolved systemd socket activation ordering cycle by removing redundant network.target dependency from syntrop-sockets.target.
* Thu Oct 01 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.26-1
- Dynamic Memory & Elasticity wiring: AmbientCapabilities (CAP_SYS_PTRACE) and caller lease ownership in inferenced (0.3.7), JIT layer prefetch pipeline integration and sample VRAM watermark decode loop in runtimed (0.5.11).
* Thu Oct 01 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.25-1
- Dynamic Memory & Host OS Elasticity: DRM sysfs telemetry and cgroup v2 slice priority preemption in inferenced (0.3.5), dual-watermark hysteresis controller and JIT layer prefetcher in runtimed (0.5.9).
* Wed Sep 30 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.24-1
- Multimedia enhancements in runtimed (0.5.7) including async non-blocking audio streaming, atomic visual file output, soft token image cache wiring, and neural weights integration.
* Wed Sep 30 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.23-1
- Multimedia Memory & Pipeline enhancements in runtimed (0.5.6) including vectorized patch pooling, ephemeral vision lifecycle, visual prefix cache, Kokoro TTS, and SD-Turbo generative output.
* Wed Sep 30 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.22-1
- End-to-end multimedia support with vision base64 pass-through in routerd and updated suite component synchronization.
* Wed Sep 30 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.21-1
- Unified front door syntrop and syn CLI, unprivileged daemon units, idle unload.
* Fri Sep 25 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.5-1
- Setup verifies every provider live before enabling; dead entries switch off.
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
