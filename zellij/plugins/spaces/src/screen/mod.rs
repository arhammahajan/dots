//! Screen-reader fallback for panes that do not publish live OSC 7501 state.
//!
//! Herdr's manifest-driven detector is trimmed and embedded here; see
//! `manifest.rs` and `manifests/herdr/` for provenance and licensing.

mod manifest;

use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenState {
    Idle,
    Working,
    Blocked,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detected {
    pub state: ScreenState,
    pub visible_idle: bool,
    pub visible_blocker: bool,
    pub visible_working: bool,
    pub skip_state_update: bool,
    pub matched_rule: Option<String>,
}

impl Detected {
    pub fn unknown(rule: Option<String>) -> Self {
        Detected {
            state: ScreenState::Unknown,
            visible_idle: false,
            visible_blocker: false,
            visible_working: false,
            skip_state_update: false,
            matched_rule: rule,
        }
    }

    pub fn has_evidence(&self) -> bool {
        self.visible_idle || self.visible_blocker || self.visible_working
    }
}

/// Apps whose OSC 7501 lifecycle is complete enough that the screen must not
/// override a live non-idle state. Blockers still override (see app.rs).
const OSC_AUTHORITY_APPS: &[&str] = &[
    "claude",
    "codex",
    "opencode",
    "pi",
    "crush",
    "grok",
    "omp",
    "copilot",
    "kimi",
    "muse",
    "mcode",
    "fx",
    "fx-router",
    "vibe",
    "vibe-acp",
    "dsh-tui",
    "deepseek",
];

static AUTHORITY: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn authority_set() -> &'static HashSet<&'static str> {
    AUTHORITY.get_or_init(|| OSC_AUTHORITY_APPS.iter().copied().collect())
}

/// Canonical manifest id where one exists, else the normalized app label.
/// A few harness labels do not have manifests but still name the same agent.
pub fn canonical_app(app: &str) -> String {
    let normalized = manifest::normalize_app(app);
    if let Some(id) = manifest::canonical_id(&normalized) {
        return id.to_string();
    }
    match normalized.as_str() {
        "dsh" | "dsh-tui" => "deepseek",
        "fx-router" => "fx",
        "vibe-acp" => "vibe",
        "kimi-cli" | "kimi-code" => "kimi",
        "claude" | "claude-code" => "claude",
        _ => normalized.as_str(),
    }
    .to_string()
}

/// Whether a bundled manifest can evaluate this app.
#[allow(dead_code)]
pub fn agent_is_known(app: &str) -> bool {
    manifest::manifest_exists(&manifest::normalize_app(app))
}

/// Whether a live non-idle OSC state should win over screen working/idle.
pub fn osc_authority(app: &str) -> bool {
    let canonical = canonical_app(app);
    if authority_set().contains(canonical.as_str()) {
        return true;
    }
    // Aliases that do not have a manifest still appear as raw labels.
    authority_set().contains(manifest::normalize_app(app).as_str())
}

/// Evaluate the viewport for an app. `osc_title` is the pane title Zellij
/// exposes (the OSC 0/2 title); `osc_progress` is unavailable to plugins.
pub fn detect(app: &str, screen: &str, osc_title: &str) -> Detected {
    if screen.trim().is_empty() {
        return Detected::unknown(None);
    }
    manifest::detect(&manifest::normalize_app(app), screen, osc_title, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workout_screen() -> &'static str {
        "● Working on the task\n  ⎿  running tests\n\n───\n❯\n"
    }

    #[test]
    fn authority_covers_native_agents_and_raw_aliases() {
        for app in [
            "claude-code",
            "cursor-agent", // not authority, but canonical
            "mcode",
            "fx-router",
            "vibe-acp",
            "dsh-tui",
            "deepseek",
            "muse",
        ] {
            let expected = app != "cursor-agent";
            assert_eq!(osc_authority(app), expected, "{app}");
        }
    }

    #[test]
    fn harness_aliases_canonicalize_to_one_agent() {
        assert_eq!(canonical_app("dsh-tui"), "deepseek");
        assert_eq!(canonical_app("dsh"), "deepseek");
        assert_eq!(canonical_app("fx-router"), "fx");
        assert_eq!(canonical_app("vibe-acp"), "vibe");
        assert_eq!(canonical_app("kimi-cli"), "kimi");
        assert_eq!(canonical_app("claude-code"), "claude");
        assert_eq!(canonical_app("cursor-agent"), "cursor");
    }

    #[test]
    fn unknown_apps_and_empty_screens_have_no_evidence() {
        let detected = detect("some-unknown-app", workout_screen(), "");
        assert_eq!(detected.state, ScreenState::Unknown);
        assert!(!detected.has_evidence());
        assert_eq!(detect("gemini", "", ""), Detected::unknown(None));
    }

    #[test]
    fn gemini_blocker_is_detected() {
        let screen = "\
│ Apply this change\n\
│ Allow execution\n\
│ Do you want to proceed?\n\
";

        let detected = detect("gemini", screen, "");
        assert_eq!(detected.state, ScreenState::Blocked);
        assert!(detected.visible_blocker);
    }

    #[test]
    fn cursor_blocker_is_detected() {
        let screen = "\
Waiting for approval to run this command?\n\
  Run (once) (y)\n\
  Skip (esc or n)\n\
";
        let detected = detect("cursor-agent", screen, "");
        assert_eq!(detected.state, ScreenState::Blocked);
        assert!(detected.visible_blocker);
    }

    #[test]
    fn claude_working_and_blocked_screens_are_detected() {
        let working = "\
● Extracting skill names from upstream trees\n\
  ⎿  $ cd /tmp\n\
\n\
✽ Pontificating… (2m 12s · ↓ 4.3k tokens)\n\
\n\
───────────────\n\
❯\n\
───────────────\n\
  ⏵⏵ auto mode on (shift+tab to cycle) · esc to interrupt\n\
";
        let detected = detect("claude-code", working, "");
        assert_eq!(detected.state, ScreenState::Working);
        assert!(detected.visible_working);

        let blocked = "\
──────────────────────────────\n\
 Allow this read outside the working directories?\n\
 ❯ 1. Yes, and keep allowing any reads\n\
   2. No, and block reads\n\
\n\
 Esc to cancel · Tab to amend\n\
";
        let detected = detect("claude-code", blocked, "");
        assert_eq!(detected.state, ScreenState::Blocked);
        // This exact footer is covered by the legacy Claude rule, which does
        // not set the strong `visible_blocker` flag. Native OSC already
        // reports Blocked for this pane.
    }

    #[test]
    fn plain_shell_does_not_look_like_an_agent() {
        let screen = "$ ls\nCargo.toml  src\n$ ";
        let detected = detect("gemini", screen, "");
        assert_eq!(detected.state, ScreenState::Unknown);
        assert!(!detected.has_evidence());
    }
}
