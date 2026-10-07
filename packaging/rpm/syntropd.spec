Name:           syntropd
Version:        0.6.2
Release:        1%{?dist}
Summary:        Native AI Subsystem for systemd — Universal Meta-Package & Supervisor

License:        Apache-2.0
URL:            https://syntropd.github.io
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo >= 1.80
BuildRequires:  rust >= 1.80
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc

Requires:       systemd >= 252
Requires:       glibc >= 2.34
Requires:       acl
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
install -D -p -m 0755 uninstall.sh %{buildroot}%{_bindir}/syntrop-uninstall
ln -sf syntrop-uninstall %{buildroot}%{_bindir}/syn-uninstall

install -D -p -m 0644 units/syntrop-sockets.target %{buildroot}%{_unitdir}/syntrop-sockets.target
install -D -p -m 0644 units/syntrop-triage@.service %{buildroot}%{_unitdir}/syntrop-triage@.service
install -D -p -m 0644 units/syntrop-admin@.service %{buildroot}%{_unitdir}/syntrop-admin@.service
install -D -p -m 0644 units/syntrop-tuning.service %{buildroot}%{_unitdir}/syntrop-tuning.service
install -D -p -m 0644 units/runtimed.service %{buildroot}%{_unitdir}/runtimed.service
install -D -p -m 0644 units/syntrop-companion.service %{buildroot}%{_userunitdir}/syntrop-companion.service

install -D -p -m 0644 tmpfiles.d/syntrop.conf %{buildroot}%{_tmpfilesdir}/syntrop.conf
install -D -p -m 0644 sysusers.d/syntrop.conf %{buildroot}%{_sysusersdir}/syntrop.conf
install -D -p -m 0644 packaging/udev/70-syntrop-uinput.rules %{buildroot}/usr/lib/udev/rules.d/70-syntrop-uinput.rules
install -D -p -m 0755 packaging/shims/systemd-inhibit %{buildroot}%{_prefix}/lib/syntrop/bin/systemd-inhibit
install -D -p -m 0644 packaging/systemd/inferenced.service.d/10-inhibit.conf %{buildroot}%{_unitdir}/inferenced.service.d/10-inhibit.conf
install -D -p -m 0644 packaging/polkit/50-syntrop-inhibit.rules %{buildroot}%{_datadir}/polkit-1/rules.d/50-syntrop-inhibit.rules

install -d -m 0775 %{buildroot}%{_sysconfdir}/syntrop
install -d -m 0775 %{buildroot}%{_sharedstatedir}/syntrop
install -d -m 0775 %{buildroot}%{_sharedstatedir}/models

%post
%systemd_post syntrop-sockets.target
%sysusers_create_package syntropd %{_sysusersdir}/syntrop.conf
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
%{_bindir}/syntrop-uninstall
%{_bindir}/syn-uninstall
%{_unitdir}/syntrop-sockets.target
%{_unitdir}/syntrop-triage@.service
%{_unitdir}/syntrop-admin@.service
%{_unitdir}/syntrop-tuning.service
%{_unitdir}/runtimed.service
%{_userunitdir}/syntrop-companion.service
%{_tmpfilesdir}/syntrop.conf
%{_sysusersdir}/syntrop.conf
/usr/lib/udev/rules.d/70-syntrop-uinput.rules
%{_prefix}/lib/syntrop/bin/systemd-inhibit
%{_unitdir}/inferenced.service.d/10-inhibit.conf
%{_datadir}/polkit-1/rules.d/50-syntrop-inhibit.rules
%dir %attr(0775, root, syntrop) %{_sysconfdir}/syntrop
%dir %attr(0775, syntrop, syntrop) %{_sharedstatedir}/syntrop
%dir %attr(0775, root, syntrop) %{_sharedstatedir}/models

%changelog
* Tue Oct 06 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.6.2-1
- Release v0.6.2: Multi-core CPU Candle/Rayon auto-threading, cgroup CPU quota clamping, declarative sysusers/tmpfiles provisioning, POSIX ACL immediate unprivileged access, BitNet ternary quantization, and multimodal inference suite.
* Thu Oct 01 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.28-1
- Model Family Cooperative Inference: Heterogeneous CPU AVX-512 draft session, Leviathan exact rejection sampler, shared VocabTrie pool, and decode_loop_managed KV layer cache spilling in runtimed (0.5.14); cascaded System 1/System 2 routing and elastic VRAM pressure downgrading in routerd (0.3.12); on-demand binary build target detection in inferenced-qa (0.3.10).
* Thu Oct 01 2026 Syntropd Authors <syntropd@users.noreply.github.com> - 0.3.27-1
- Dynamic Memory & Elasticity production hardening: Production generate routed via decode_loop_managed with zero-budget short-circuit and empty sequence guard in runtimed (0.5.13); GetDrmWatermark and ResizeLease Varlink specs synchronized with device parameter examples and CLI profile path resolution in inferenced (0.3.9); resolved systemd socket activation ordering cycle by removing redundant network.target dependency from syntrop-sockets.target.
