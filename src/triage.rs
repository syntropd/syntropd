//! Autonomous failure triage and root-cause explanation engine.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Categorization of systemd failure root-causes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultCategory {
    /// OOM killed or memory cgroup limit reached.
    OutOfMemory,
    /// Memory pressure stall or hardware throttle.
    ResourceExhaustion,
    /// Missing binary, working directory, or missing dependencies (e.g. 203/EXEC).
    MissingExecutableOrPath,
    /// Permission denied, missing Linux capabilities, or Landlock restriction.
    PermissionDenied,
    /// Segmentation fault, assertion failure, or core dump.
    CrashOrSegmentationFault,
    /// Watchdog expiration or systemd ping timeout.
    WatchdogTimeout,
    /// Service exited with unexpected non-zero code.
    NonZeroExit(i32),
    /// Service is healthy or not in failed state.
    HealthyOrActive,
    /// Failure reason could not be definitively categorized.
    UnknownFailure,
}

/// Comprehensive structured triage report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageReport {
    /// Target systemd unit name.
    pub unit: String,
    /// Current systemd active state (e.g. "failed", "active", "inactive").
    pub active_state: String,
    /// Detected fault category.
    pub fault_category: FaultCategory,
    /// High-level root cause summary.
    pub summary: String,
    /// Excerpt of relevant journal entries.
    pub journal_excerpt: Vec<String>,
    /// Concrete actionable remediation advice.
    pub recommendation: String,
}

/// Perform root cause triage on a systemd unit.
pub fn triage_unit(unit_name: &str) -> TriageReport {
    let clean_unit = if unit_name.contains('.') {
        unit_name.to_string()
    } else {
        format!("{}.service", unit_name)
    };

    let active_state = crate::units::query_unit_active_state(&clean_unit);
    let journal_lines = fetch_recent_journal(&clean_unit, 40);

    let (fault_category, summary, recommendation) = if active_state == "active" {
        (
            FaultCategory::HealthyOrActive,
            format!("Unit {} is currently active and healthy.", clean_unit),
            "No remediation required.".to_string(),
        )
    } else {
        categorize_failure(&clean_unit, &active_state, &journal_lines)
    };

    TriageReport {
        unit: clean_unit,
        active_state,
        fault_category,
        summary,
        journal_excerpt: journal_lines.into_iter().take(10).collect(),
        recommendation,
    }
}

/// Query systemd journal for recent entries of a unit.
fn fetch_recent_journal(unit: &str, limit: usize) -> Vec<String> {
    let output = Command::new("journalctl")
        .arg("-u")
        .arg(unit)
        .arg("-n")
        .arg(limit.to_string())
        .arg("--no-pager")
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        text.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

/// Analyze journal logs and status to categorize failure.
pub fn categorize_failure(
    unit: &str,
    _active_state: &str,
    journal: &[String],
) -> (FaultCategory, String, String) {
    let full_text = journal.join("\n").to_lowercase();

    if full_text.contains("out of memory")
        || full_text.contains("oom-killer")
        || full_text.contains("killed process")
        || full_text.contains("memory.max")
        || full_text.contains("oom_score_adj")
    {
        return (
            FaultCategory::OutOfMemory,
            format!("Unit {} was terminated by OOM Killer or cgroup memory limit.", unit),
            "Inspect cgroup memory.max and system PSI via `syntropctl drift`. Consider increasing MemoryMax= or provisioning swap.".to_string(),
        );
    }

    if full_text.contains("core dumped")
        || full_text.contains("dumped core")
        || full_text.contains("code=dumped")
        || full_text.contains("segv")
        || full_text.contains("sigsegv")
        || full_text.contains("segmentation fault")
        || full_text.contains("sigabrt")
    {
        return (
            FaultCategory::CrashOrSegmentationFault,
            format!("Unit {} crashed with fatal signal or core dump.", unit),
            "Inspect coredump via `coredumpctl info` and review systemd sandbox filters (ProtectSystem, SystemCallFilter).".to_string(),
        );
    }

    if full_text.contains("failed with result 'watchdog'")
        || full_text.contains("watchdog timeout")
    {
        return (
            FaultCategory::WatchdogTimeout,
            format!("Unit {} failed to ping systemd watchdog within WatchdogSec.", unit),
            "Check CPU saturation, deadlocks, or long-running synchronous I/O blocking sd_notify pings.".to_string(),
        );
    }

    if full_text.contains("status=203/exec")
        || full_text.contains("no such file or directory")
    {
        return (
            FaultCategory::MissingExecutableOrPath,
            format!("Unit {} failed to start because the executable or path is missing.", unit),
            "Verify binary path in ExecStart= and ensure all required directories exist.".to_string(),
        );
    }

    if full_text.contains("permission denied")
        || full_text.contains("status=204/perm")
        || full_text.contains("eacces")
    {
        return (
            FaultCategory::PermissionDenied,
            format!("Unit {} encountered permission denial or security capability restriction.", unit),
            "Verify User=/Group= permissions, CapabilityBoundingSet=, and directory access permissions.".to_string(),
        );
    }

    (
        FaultCategory::UnknownFailure,
        format!("Unit {} is in non-active state.", unit),
        format!("Run `journalctl -u {unit} -e` for full journal chronology or `syntropctl explain {unit}`."),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_oom() {
        let logs = vec![
            "kernel: Memory cgroup out of memory: Killed process 12345 (worker)".to_string(),
            "systemd[1]: worker.service: Main process exited, code=killed, status=9/KILL".to_string(),
        ];
        let (cat, summary, rec) = categorize_failure("worker.service", "failed", &logs);
        assert_eq!(cat, FaultCategory::OutOfMemory);
        assert!(summary.contains("OOM Killer"));
        assert!(rec.contains("MemoryMax"));
    }

    #[test]
    fn test_categorize_crash() {
        let logs = vec![
            "systemd[1]: test.service: Main process exited, code=dumped, status=11/SEGV".to_string(),
            "systemd-coredump[445]: Process 123 (test) of user 1000 dumped core.".to_string(),
        ];
        let (cat, summary, _rec) = categorize_failure("test.service", "failed", &logs);
        assert_eq!(cat, FaultCategory::CrashOrSegmentationFault);
        assert!(summary.contains("crashed"));
    }

    #[test]
    fn test_categorize_exec_missing() {
        let logs = vec![
            "systemd[1]: demo.service: Failed at step EXEC spawning /usr/bin/missing: No such file or directory".to_string(),
            "systemd[1]: demo.service: Main process exited, code=exited, status=203/EXEC".to_string(),
        ];
        let (cat, _summary, _rec) = categorize_failure("demo.service", "failed", &logs);
        assert_eq!(cat, FaultCategory::MissingExecutableOrPath);
    }

    #[test]
    fn test_unknown_failure_recommendation_names_unit() {
        let logs = vec!["some unrecognized failure line".to_string()];
        let (cat, _summary, rec) = categorize_failure("demo.service", "failed", &logs);
        assert_eq!(cat, FaultCategory::UnknownFailure);
        assert!(
            rec.contains("demo.service"),
            "recommendation should name the unit: {}",
            rec
        );
        assert!(
            !rec.contains("{}"),
            "recommendation must not contain unfilled placeholder: {}",
            rec
        );
    }
}
