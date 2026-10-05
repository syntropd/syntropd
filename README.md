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
| **inferenced** | `syntrop-inferenced` | `inferenced`, `inferenctl` | `/run/syntrop/io.syntrop.Inference1` | Hardware arbiter, demand paging, memory leases, OpenAI-compatible gateway (`/run/syntrop/gateway.sock`) |
| **modeld** | `syntrop-modeld` | `modeld`, `modelctl` | `/run/syntrop/io.syntrop.Model1` | Content-addressable model cache, zero-copy shm distribution |
| **contextd** | `syntrop-contextd` | `contextd`, `contextctl` | `/run/syntrop/io.syntrop.Context1` | Causal event graphs, configuration diffs, system chronology |
| **toold** | `syntrop-toold` | `toold`, `toolctl` | `/run/syntrop/io.syntrop.Tool1` | Sandboxed diagnostic & remediation execution with automated rollback |
| **runtimed** | `syntrop-runtimed` | `runtimed`, `runtimectl` | `/run/syntrop/io.syntrop.Runtime1` | Headless model execution, tensor generation, GPU/NPU acceleration |
| **sentry** | `syntrop-sentry` | `sentry`, `systemd-sentry` | `/run/systemd-sentry/sentry.sock` | Autonomous supervisor daemon, systemd crash triage watchdog |
| **routerd** | `syntrop-routerd` | `routerd`, `routerctl` | `/run/syntrop/io.syntrop.Router1` | Multi-provider LLM reverse proxy, dynamic router & telemetry offload gateway |
| **syntropctl** | `syntropctl` | `syntropctl` | *(CLI Operator)* | Operator CLI for inspection, drift, models, and failure triage |
| **syntropd** | `syntropd` | `syntropd` | *(Umbrella CLI)* | Unified umbrella CLI, status monitoring, and distribution packaging |

---

## 3. Zero-Idle Socket Activation & Memory Governance

Traditional AI subsystems continuously consume gigabytes of host RAM even when dormant. In contrast, `syntropd` daemons are governed by **systemd socket activation and kernel-native memory controls**:

1. **Zero Idle Overhead**: At boot, systemd creates and listens on all IPC sockets (`/run/syntrop/*.sock`, `/run/systemd-sentry/sentry.sock`). While idle, **0 daemons run and 0 MB of resident RAM is consumed**.
2. **Demand Activation**: When an event or request arrives on a socket, systemd transparently starts the responsible daemon and hands over the listening socket file descriptor via `LISTEN_FDS`.
3. **Zero-Copy Sealed `memfd` CAS Pipeline**: `modeld` provisions immutable, sealed memory descriptors (`memfd_create` + `F_SEAL_SEAL | F_SEAL_WRITE`) transmitted over Unix sockets via `SCM_RIGHTS` into `runtimed`. Model weights are mapped directly without duplicate disk reads or memory copying.
4. **Autonomous PSI Memory Load-Shedding**: `runtimed` continuously samples Linux Pressure Stall Information (`/proc/pressure/memory`). When stalls spike (`some > 25.0%` or `full > 5.0%`), proactive load-shedding evicts non-busy resident weights within one sample window without dropping active client sessions.
5. **Configurable Idle Unload**: When quiet, `runtimed` can shed resident weights automatically after a quiet duration (`RUNTIMED_IDLE_UNLOAD_SECS=300`), releasing hardware leases and freeing RAM/VRAM.
6. **Strict Security Sandboxing**: 100% of daemons drop root privileges to run under dedicated system accounts with Landlock ABI v3 filesystem sandboxing, seccomp filters (`@system-service`), and strict cgroups v2 resource ceilings.

---

## 4. Advanced AI Subsystem Frontiers

`syntropd` incorporates seven architectural frontiers for production-grade, reliable system intelligence:

1. **Constrained / Grammar-Guided Decoding (`runtimed`)**:
   Deterministic FSM token filtering supporting JSON schema, regex, and Varlink nul-terminated protocols with SIMD-aligned bitset logit masking and vocab trie pruning, guaranteeing syntactically valid machine outputs.

2. **Heterogeneous Speculative Decoding (`inferenced` & `runtimed`)**:
   Gang-scheduling drafts onto CPU-host matrix extensions (`PlaneRole::Draft`) while serving target models on discrete GPU/NPU accelerators (`PlaneRole::Target`), backed by O(1) KV-cache rollback (`truncate`).

3. **Autonomous Agentic Triage & Environment Feedback (`toold` & `sentry`)**:
   Unprivileged bubblewrap and Landlock sandboxing with isolated namespaces, executing guarded diagnostic iterations under strict circuit breaking (5 iterations / 30s timeout) and error reflection.

4. **Test-Time Compute Scaling & Reasoning Budgets (`routerd` & `runtimed`)**:
   Granular reasoning token allocation (`reasoning_budget`, `max_thinking_tokens`) with forced `</think>` token emission/logit masking, and streaming state machine (`ThinkFilter`) separating internal thoughts into `reasoning_content`.

5. **Dynamic State Compression & Streaming Infinite Context (`runtimed`)**:
   StreamingLLM attention sinks (`SinkWindowCache`, `sink_causal_mask`) with cache-relative RoPE and bounded streaming event journals (`StreamJournal`) providing continuous unbounded inference without memory leakage.

6. **End-to-End Multimedia Pipeline (`routerd` & `runtimed`)**:
   Full bidirectional multimodal engine: vectorized spatial patch pooling (2x2, 3x3, 4x4) with symmetric edge replication, ephemeral vision tower lifecycle (VRAM unpinning post-prefill saving 20%–30% device memory), L2 host visual KV prefix caching for multi-turn chats, real-time 24kHz S16LE streaming audio out (Kokoro-82M TTS) directly to PipeWire (`pw-cat`), and 1-step SD-Turbo generative visuals rendering to atomic PNG output via isolated compute leases. Exposed natively over Varlink RPCs (`StreamAudioOut` and `GenerateVisual`).

7. **Dynamic Memory & Host OS Elasticity (`inferenced` & `runtimed`)**:
   Zero-idle kernel telemetry via non-blocking fixed-stack sysfs DRM sampling (`/sys/class/drm/renderD*`), dual-watermark hysteresis memory governor (spill from 85% high down to 70%, prefetch below 65% with 5.0s dwell cooldown), recursive cgroups v2 slice priority binding (`user.slice` interactive preemption with 250ms cooperative SIGUSR1 deadline), and circular double-buffered JIT attention layer prefetching from host pinned RAM.

8. **Linux Cognitive Desktop Companion (`syntropctl companion` / `syn companion`)**:
   Zero-disk multimodal streaming of desktop screen frames directly to `routerd` (`/run/syntrop/router.sock`) via HTTP `POST /v1/chat/completions`, grounded visual question answering (`syn companion ask`), planned virtual HID UI actions via `io.syntrop.Actuator1` (`syn companion execute`), intermediate visual state validation, automatic physical user input preemption, and session-aware systemd user unit daemon (`syntrop-companion.service`).

---

## 5. Installation

### 5.1 Universal Automated Installer
The easiest way to install `syntropd` across any Linux distribution:

```bash
curl -fsSL https://syntropd.github.io/install.sh | sudo bash
```

To supply a Hugging Face token directly for unthrottled downloads of gated models (e.g. Gemma):
```bash
curl -fsSL https://syntropd.github.io/install.sh | sudo bash -s -- --hf-token <TOKEN>
```

To run pre-flight compatibility checks without modifying your system:
```bash
curl -fsSL https://syntropd.github.io/install.sh | bash -s -- --dry-run
```

If a binary is missing from both local builds and the release bundle, the
installer compiles it from a matching source checkout instead of failing.
Builds run offline first, as your user, so your cargo cache is reused.

### 5.2 Single-Command Cargo Installation
Install the umbrella CLI directly from crates.io:

```bash
cargo install syntropd
```

To install the individual daemons:
```bash
cargo install syntropctl syntrop-toold syntrop-runtimed syntrop-inferenced syntrop-contextd syntrop-modeld syntrop-sentry syntrop-routerd
```

### 5.3 Distribution Packages

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

### 5.4 Building from source
Release builds use thin link-time optimization with stripped symbols
(portable x86-64, no chip-specific instructions):

```bash
cargo build --release
```

---

## 6. Verification & Operator Usage

One front door reaches every tool: `syn <namespace> <command>`, e.g.
`syn router models`, `syn fleet status`, `syn system units`. The full
name `syntrop` works identically; per-repo CLIs (`routerctl`, …) are
unchanged underneath. Namespaces: `router`, `runtime`, `store`,
`hardware`, `context`, `tools`, `fleet`, `system`.

### Model Family Autoloader (`syn setup`)
`syn setup` inspects your machine's hardware envelope (host RAM, GPU VRAM, CPU cores), downloads and pins a complete curated model family (Qwen, Granite, or Gemma) with optimal CPU draft / GPU primary pairings via `modelctl bootstrap`, configures `/etc/syntrop/routerd.toml` for heterogeneous speculative decoding, and reloads `routerd.service`:

```bash
# Bootstrap curated model family (default: qwen)
sudo syn setup --family qwen

# Preview hardware envelope sizing without downloading
syn setup --family granite --dry-run
```

Verify subsystem readiness and hardware plane detection:

```bash
# First-time LLM setup (alternative manual wiring)
sudo syn router setup

# List connected models (routing aliases hidden)
syn router models

# Show or pin the default model (pinning needs sudo)
syn router default
sudo syn router default qwen2.5-coder:7b

# Verify umbrella system status
syn system status

# List active systemd sockets
systemctl list-sockets "syntrop*"

# Check operator status
syn fleet status

# Autonomous triage of any failed service (e.g. nginx or postgresql)
syn fleet explain nginx.service
# or
syn system triage nginx.service
```

### Autonomous Triage & Self-Healing Hooks
To enable autonomous diagnosis and self-healing for any mission-critical systemd service, append `OnFailure=` hooks to its unit definition:

```ini
[Unit]
Description=My Critical Service
OnFailure=syntrop-admin@%n.service
```

When the service fails, systemd dispatches `syntrop-admin@.service`, which triggers `syn admin remediate %I` under an unprivileged sandboxed user, querying `sentry` for circuit-breaker safety, inspecting causal drift in `contextd`, verifying syntax and executing remediations with pre-mutation snapshots in `toold`, and logging structured audit fields directly to `systemd-journald`.

---

## 7. Autonomous Administration (`syn admin`)

`syn admin` provides unified management of autonomous self-healing, declarative remediation recipes, and circuit-breaker lockout safety:

```bash
# Check autonomous healing readiness and circuit-breaker states
syn admin status

# Execute or simulate a guided remediation recipe
syn admin remediate nginx.service --recipe restart --dry-run
syn admin remediate nginx.service

# Roll back configuration modifications from remediation snapshots
syn admin rollback nginx.service

# Inspect structured journald forensic audit records
syn admin audit --limit 50 --unit nginx.service --json

# Manually clear a flapping or tripped circuit breaker lockout
syn admin lockout reset nginx.service
```

---

## 8. Linux Cognitive Desktop Companion (`syn companion`)

`syn companion` provides multimodal visual grounding, UI action planning, and desktop actuation:

```bash
# Ask a grounded visual question about the active screen
syn companion ask "Explain what is causing the compilation error in the active terminal"

# Plan and execute desktop UI actions via virtual HID actuator
syn companion execute "focus terminal and run cargo check" --dry-run
syn companion execute "click the submit button"

# Run daemonized ambient listening session for voice or hotkey triggers
syn companion listen --voice --hotkey "Super+Space"

# Manage systemd user unit
systemctl --user status syntrop-companion.service
systemctl --user enable --now syntrop-companion.service
```

---

## 9. Dynamic Kernel Telemetry & Closed-Loop Tuning (`syn telemetry`, `syn tune`)

`syntropd` provides zero-allocation kernel Pressure Stall Information (PSI) monitoring and eBPF runqueue latency tracking with dynamic closed-loop tuning:

```bash
# Inspect instantaneous kernel PSI pressure and eBPF runqueue latency
syn telemetry status
syn telemetry status --json

# Query or adjust closed-loop dynamic tuning governor policy
syn tune --policy balanced
syn tune --policy aggressive
syn tune --policy conservative

# Manage the systemd governor unit
systemctl status syntrop-tuning.service
```

When memory pressure spikes (`some > 25.0%` or `full > 10.0%` under balanced policy), `routerd` automatically clamps token budgets, shortens speculative draft horizons ($K \to 1$), and triggers proactive KV cache compaction (`io.syntrop.Runtime1.CompactKvCache`). When CPU contention spikes, speculative threads yield cooperatively (`tokio::task::yield_now()`) within a 250ms deadline.

---

## 10. License

Dual-licensed under the **Apache License, Version 2.0** ([LICENSE](LICENSE)).

