# syntropd

> **Native AI Subsystem for systemd**  
> *Baking local AI acceleration, demand-paged inference, and autonomous fault remediation directly into Linux as native OS primitives — keeping PID 1 inviolable.*

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80.0%2B-orange.svg)](https://www.rust-lang.org)
[![systemd](https://img.shields.io/badge/systemd-v252%2B-red.svg)](https://systemd.io)
[![Documentation](https://img.shields.io/badge/docs-syntropd.github.io-green.svg)](https://syntropd.github.io)
[![crates.io](https://img.shields.io/crates/v/syntropd.svg)](https://crates.io/crates/syntropd)

```bash
# Install the complete native AI subsystem across any Linux distribution
curl -fsSL https://syntropd.github.io/install.sh | sudo bash
```

> **Single-command cargo alternative:** `cargo install syntropd`

---

## 1. System Architecture

`syntropd` integrates autonomous intelligence directly into Linux system management. Rather than introducing complex user-space stacks or external orchestration layers, `syntropd` leverages standard Linux kernel facilities (`cgroups v2`, Pressure Stall Information, Landlock, seccomp, DRM/KMS render nodes) and `systemd` primitives (socket activation, `sd_notify`, file descriptor passing, `OnFailure=` event triggers).

```text
+-------------------------------------------------------------------------+
|                              Linux Kernel                               |
|        (cgroups v2, Memory PSI, DRM/Accel /dev/dri, Landlock/Seccomp)   |
+-------------------------------------------------------------------------+
                                    |
+-------------------------------------------------------------------------+
|                        PID 1: systemd (v252+)                           |
|       (syntrop-sockets.target, OnFailure=syntrop-triage@%n.service)     |
+-------------------------------------------------------------------------+
         |               |              |              |             |              |
         v               v              v              v             v              v
+----------------+ +------------+ +-----------+ +-----------+ +------------+ +---------------+
|   inferenced   | |   modeld   | | contextd  | |   toold   | | runtimed   | |    routerd    |
| (HW Arbiter)   | | (Model CAS)| | (Drift/CG)| | (Sandboxed| | (Tensor/   | | (LLM Proxy/   |
|  Demand Paging | | Zero-Copy  | | Causality | |  Rollback)| |  Inference)| |  Dyn Routing) |
+----------------+ +------------+ +-----------+ +-----------+ +------------+ +---------------+
         ^                                                                          ^
         |                    Varlink IPC / AF_UNIX                                 |
         +--------------------------------------------------------------------------+
                                    |
                 +--------------------------------------+
                 |  sentry / systemd-sentry Supervisor  |
                 |      (Crash Triage & Watchdog)       |
                 +--------------------------------------+
                                    |
                 +--------------------------------------+
                 |       syntropctl / syntropd CLI      |
                 +--------------------------------------+
```

---

## 2. Component Inventory & Varlink IPC

All daemons communicate over standard UNIX domain sockets using [Varlink](https://varlink.org) framing (`\0` delimiters) and standard JSON payloads. All daemons link purely against standard glibc (`libc.so.6`) with zero dynamic C dependencies.

| Component | Crates.io Package | Binaries | Primary Socket / Address | Functional Role |
| :--- | :--- | :--- | :--- | :--- |
| **inferenced** | `syntrop-inferenced` | `inferenced`, `inferenctl` | `/run/syntrop/io.syntrop.Inference1` | Hardware arbiter, demand paging, memory leases, Ollama/OpenAI gateway (`/run/syntrop/gateway.sock`) |
| **modeld** | `syntrop-modeld` | `modeld`, `modelctl` | `/run/syntrop/io.syntrop.Model1` | Content-addressable model cache, zero-copy shm distribution |
| **contextd** | `syntrop-contextd` | `contextd`, `contextctl` | `/run/syntrop/io.syntrop.Context1` | Causal event graphs, configuration diffs, system chronology |
| **toold** | `syntrop-toold` | `toold`, `toolctl` | `/run/syntrop/io.syntrop.Tool1` | Sandboxed diagnostic & remediation execution with automated rollback |
| **runtimed** | `syntrop-runtimed` | `runtimed`, `runtimectl` | `/run/syntrop/io.syntrop.Runtime1` | Headless model execution, tensor generation, GPU/NPU acceleration |
| **sentry** | `syntrop-sentry` | `sentry`, `systemd-sentry` | `/run/systemd-sentry/sentry.sock` | Autonomous supervisor daemon, systemd crash triage watchdog |
| **routerd** | `syntrop-routerd` | `routerd`, `routerctl` | `/run/syntrop/io.syntrop.Router1` | Multi-provider LLM reverse proxy, dynamic router & telemetry offload gateway |
| **syntropctl** | `syntropctl` | `syntropctl` | *(CLI Operator)* | Operator CLI for inspection, drift, models, and failure triage |
| **syntropd** | `syntropd` | `syntropd` | *(Umbrella CLI)* | Unified umbrella CLI, status monitoring, and distribution packaging |

---

## 3. Zero-Idle Socket Activation

Traditional AI subsystems continuously consume gigabytes of host RAM even when dormant. In contrast, `syntropd` daemons are governed by **systemd socket activation**:

1. At boot, systemd creates and listens on all IPC sockets (`/run/syntrop/*.sock`, `/run/systemd-sentry/sentry.sock`).
2. While idle, **0 daemons run and 0 MB of resident RAM is consumed**.
3. When an event or request arrives on a socket, systemd transparently starts the responsible daemon and hands over the listening socket file descriptor via `LISTEN_FDS`.
4. When processing finishes and no active leases remain, daemons exit cleanly, releasing memory back to the host kernel.
5. If memory pressure rises, the kernel PSI monitor signals `inferenced` to page out dormant tensors via `madvise(MADV_DONTNEED)` or freeze lower-priority cgroups.

---

## 4. Installation

### 4.1 Universal Automated Installer
The easiest way to install `syntropd` across any Linux distribution:

```bash
curl -fsSL https://syntropd.github.io/install.sh | sudo bash
```

To run pre-flight compatibility checks without modifying your system:
```bash
curl -fsSL https://syntropd.github.io/install.sh | bash -s -- --dry-run
```

If a binary is missing from both local builds and the release bundle, the
installer compiles it from a matching source checkout instead of failing.
Builds run offline first, as your user, so your cargo cache is reused.

### 4.2 Single-Command Cargo Installation
Install the umbrella CLI directly from crates.io:

```bash
cargo install syntropd
```

To install the individual daemons:
```bash
cargo install syntropctl syntrop-toold syntrop-runtimed syntrop-inferenced syntrop-contextd syntrop-modeld syntrop-sentry syntrop-routerd
```

### 4.3 Distribution Packages

#### Fedora, RHEL & CentOS (RPM)
```bash
sudo dnf copr enable syntropd/syntropd
sudo dnf install syntropd
sudo systemctl enable --now syntrop-sockets.target
```

#### Arch Linux (AUR)
```bash
yay -S syntropd
sudo systemctl enable --now syntrop-sockets.target
```

#### Debian & Ubuntu (APT)
```bash
sudo apt-get install syntropd
sudo systemctl enable --now syntrop-sockets.target
```

### 4.4 Building from source
Release builds use thin link-time optimization with stripped symbols
(portable x86-64, no chip-specific instructions):

```bash
cargo build --release
```

---

## 5. Verification & Operator Usage

Verify subsystem readiness and hardware plane detection:

```bash
# First-time LLM setup (required before use)
sudo routerctl setup

# List connected models (routing aliases hidden)
routerctl models

# Show or pin the default model (pinning needs sudo)
routerctl default
sudo routerctl default MiniMax-M3

# Verify umbrella system status
syntropd status

# List active systemd sockets
systemctl list-sockets "syntrop*"

# Check operator status
syntropctl status

# Autonomous triage of any failed service (e.g. nginx or postgresql)
syntropctl explain nginx.service
# or
syntropd triage nginx.service
```

### Autonomous Triage Hook
To enable autonomous diagnosis for any mission-critical systemd service, append `OnFailure=syntrop-triage@%n.service` to its unit definition:

```ini
[Unit]
Description=My Critical Service
OnFailure=syntrop-triage@%n.service
```

When the service fails, systemd immediately dispatches `syntrop-triage@.service`, running `syntropctl explain` and writing a root-cause explanation directly into the system journal.

---

## 6. License

Dual-licensed under the **Apache License, Version 2.0** ([LICENSE](LICENSE)).
