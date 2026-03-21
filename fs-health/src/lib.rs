// fs-health — Generic health check framework for FreeSynergy.
//
// Design Patterns:
//   Validator — check_* functions validate cross-resource consistency
//   Strategy  — each resource type has its own rule set
//   OOP       — HealthLevel carries its own indicators and i18n keys
//
// Health levels:
//   Ok      — all required conditions satisfied, deployment is possible.
//   Warning — optional components missing; degraded but functional.
//   Error   — required components missing; deployment will fail.

pub mod checker;
pub mod reporter;

pub use checker::HealthCheck;
pub use reporter::{HealthIssue, HealthLevel, HealthRules, HealthStatus};

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_status_is_ok() {
        let s = HealthStatus::ok();
        assert!(s.is_ok());
        assert_eq!(s.overall, HealthLevel::Ok);
    }

    #[test]
    fn error_raises_overall() {
        let mut s = HealthStatus::ok();
        s.warning("w");
        assert_eq!(s.overall, HealthLevel::Warning);
        s.error("e");
        assert_eq!(s.overall, HealthLevel::Error);
    }

    #[test]
    fn health_rules_require_and_warn() {
        let status = HealthRules::new()
            .require(true, "should.not.appear")
            .require(false, "health.missing.host")
            .warn(false, "health.no.monitoring")
            .build();

        assert_eq!(status.overall, HealthLevel::Error);
        assert_eq!(status.issues.len(), 2);
    }

    #[test]
    fn merge_two_statuses() {
        let mut a = HealthStatus::ok();
        a.warning("w1");
        let mut b = HealthStatus::ok();
        b.error("e1");
        a.merge(b);
        assert_eq!(a.overall, HealthLevel::Error);
        assert_eq!(a.issues.len(), 2);
    }

    #[test]
    fn level_ordering() {
        assert!(HealthLevel::Error > HealthLevel::Warning);
        assert!(HealthLevel::Warning > HealthLevel::Ok);
    }

    #[test]
    fn health_level_indicators() {
        assert_eq!(HealthLevel::Ok.indicator(),      "✓");
        assert_eq!(HealthLevel::Warning.indicator(), "⚠");
        assert_eq!(HealthLevel::Error.indicator(),   "✗");
    }

    #[test]
    fn health_level_indicator_text() {
        assert_eq!(HealthLevel::Ok.indicator_text(),      "ok");
        assert_eq!(HealthLevel::Warning.indicator_text(), "warning");
        assert_eq!(HealthLevel::Error.indicator_text(),   "error");
    }

    #[test]
    fn health_level_indicator_with_text() {
        assert_eq!(HealthLevel::Ok.indicator_with_text(),      "✓ (ok)");
        assert_eq!(HealthLevel::Warning.indicator_with_text(), "⚠ (warning)");
        assert_eq!(HealthLevel::Error.indicator_with_text(),   "✗ (error)");
    }

    #[test]
    fn health_level_i18n_keys() {
        assert_eq!(HealthLevel::Ok.i18n_key(),      "health.ok");
        assert_eq!(HealthLevel::Warning.i18n_key(), "health.warning");
        assert_eq!(HealthLevel::Error.i18n_key(),   "health.error");
    }

    #[test]
    fn health_level_is_ok() {
        assert!(HealthLevel::Ok.is_ok());
        assert!(!HealthLevel::Warning.is_ok());
        assert!(!HealthLevel::Error.is_ok());
    }

    #[test]
    fn health_issue_constructors() {
        let e = HealthIssue::error("err.key");
        assert_eq!(e.level, HealthLevel::Error);
        assert_eq!(e.msg_key, "err.key");

        let w = HealthIssue::warning("warn.key");
        assert_eq!(w.level, HealthLevel::Warning);

        let i = HealthIssue::info("info.key");
        assert_eq!(i.level, HealthLevel::Ok);
    }

    #[test]
    fn issues_at_level_filters_correctly() {
        let mut s = HealthStatus::ok();
        s.warning("w1");
        s.error("e1");
        s.warning("w2");

        let errors: Vec<_> = s.issues_at_level(HealthLevel::Error).collect();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].msg_key, "e1");

        let at_warn: Vec<_> = s.issues_at_level(HealthLevel::Warning).collect();
        assert_eq!(at_warn.len(), 3);
    }

    #[test]
    fn rules_all_true_yields_ok() {
        let status = HealthRules::new()
            .require(true, "no.err")
            .warn(true, "no.warn")
            .build();
        assert!(status.is_ok());
        assert_eq!(status.overall, HealthLevel::Ok);
    }

    #[test]
    fn health_check_trait_works() {
        struct DummyResource { ok: bool }

        impl HealthCheck for DummyResource {
            fn health(&self) -> HealthStatus {
                HealthRules::new()
                    .require(self.ok, "health.dummy.not_ok")
                    .build()
            }
        }

        assert!(DummyResource { ok: true }.health().is_ok());
        assert!(!DummyResource { ok: false }.health().is_ok());
    }
}
