<div align="center">

# syntropd — Native AI Subsystem for systemd

*Baking local AI acceleration, demand-paged inference, and autonomous fault remediation directly into Linux as native OS primitives — keeping PID 1 inviolable.*

[syntropd.github.io](https://syntropd.github.io) • [Documentation](https://syntropd.github.io/manual.html) • [Quickstart](https://syntropd.github.io/quickstart.html)

```sh
curl -fsSL https://syntropd.github.io/install.sh | sudo bash
```

</div>

---

## The Subsystem Architecture

`syntropd` integrates autonomous intelligence directly into Linux system management via native `cgroups v2`, Pressure Stall Information, Landlock, seccomp, DRM/KMS render nodes, and `systemd` primitives (socket activation, `sd_notify`, file descriptor passing, `OnFailure=` event triggers).

| Daemon | Role | Interface & Protocol |
|:---|:---|:---|
| **[inferenced](https://github.com/syntropd/inferenced)** | Unprivileged Heterogeneous Hardware Arbiter & Model Lifecycle Broker | `io.syntrop.Inference1` (Varlink) |
| **[modeld](https://github.com/syntropd/modeld)** | Content-Addressable Model Store & Zero-Copy Asset Broker | `io.syntrop.Model1` (Varlink) |
| **[contextd](https://github.com/syntropd/contextd)** | System Chronology, Configuration Drift, and Causality Graph | `io.syntrop.Context1` (Varlink) |
| **[toold](https://github.com/syntropd/toold)** | Sandboxed Agentic Action & Diagnostic Execution Daemon | `io.syntrop.Tool1` (Varlink) |
| **[runtimed](https://github.com/syntropd/runtimed)** | Headless Model Execution & Tensor Generation Daemon | `io.syntrop.Runtime1` (Varlink) |
| **[routerd](https://github.com/syntropd/routerd)** | LLM Dynamic Semantic Routing & Task Dispatcher | `io.syntrop.Router1` (Varlink) |
| **[sentry](https://github.com/syntropd/sentry)** | Autonomous Zero-Trust System Supervisor & Anomaly Sentry | Journald / PSI triggers |
| **[syntropctl](https://github.com/syntropd/syntropctl)** | Unified Administration and Diagnostic CLI | Terminal / Socket |
| **[syntropd](https://github.com/syntropd/syntropd)** | Subsystem Meta-Package, Units & Distribution Packaging | System Units & Target definitions |

---

License: Apache-2.0. Built with Rust and systemd.
