//! Agent screen detection, derived from herdr's manifest engine.
//!
//! Source: https://github.com/herdrdev/herdr (Apache-2.0), commit `ee5bfb50`.
//! The evaluator is trimmed to what the Spaces navigator needs: bundled
//! manifests, region extraction, gate matching, priority selection, and the
//! visible/skip flags. Remote manifest loading, overrides, diagnostics, and
//! filesystem/update machinery are intentionally omitted.
//!
//! Manifests are stored as TOML under `manifests/herdr/` and converted to
//! `manifests.json` by `scripts/sync-screen-manifests.py` so the WASM build
//! does not need a TOML parser.

use std::collections::HashMap;
use std::sync::OnceLock;

use regex::Regex;
use serde::Deserialize;

use super::{Detected, ScreenState};

const DEFAULT_REGION: &str = "whole_recent";
const TOP_NON_EMPTY_LINES_ENGINE_VERSION: u32 = 3;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestFile {
    #[serde(default)]
    source: String,
    #[serde(default)]
    engine_version: u32,
    manifests: Vec<AgentManifest>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // schema metadata retained from the vendored manifests
struct AgentManifest {
    id: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    min_engine_version: Option<u32>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    rules: Vec<ManifestRule>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ManifestRule {
    id: String,
    #[serde(default)]
    state: Option<ManifestState>,
    #[serde(default)]
    priority: i32,
    #[serde(default = "default_region")]
    region: String,
    #[serde(default)]
    visible_idle: bool,
    #[serde(default)]
    visible_blocker: bool,
    #[serde(default)]
    visible_working: bool,
    #[serde(default)]
    skip_state_update: bool,
    #[serde(default)]
    all: Vec<ManifestGate>,
    #[serde(default)]
    any: Vec<ManifestGate>,
    #[serde(default, rename = "not")]
    not_gate: Vec<ManifestGate>,
    #[serde(default)]
    contains: Vec<String>,
    #[serde(default)]
    regex: Vec<String>,
    #[serde(default)]
    line_regex: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ManifestGate {
    #[serde(default)]
    all: Vec<ManifestGate>,
    #[serde(default)]
    any: Vec<ManifestGate>,
    #[serde(default, rename = "not")]
    not_gate: Vec<ManifestGate>,
    #[serde(default)]
    contains: Vec<String>,
    #[serde(default)]
    regex: Vec<String>,
    #[serde(default)]
    line_regex: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ManifestState {
    Idle,
    Working,
    Blocked,
    Unknown,
}

impl From<ManifestState> for ScreenState {
    fn from(value: ManifestState) -> Self {
        match value {
            ManifestState::Idle => ScreenState::Idle,
            ManifestState::Working => ScreenState::Working,
            ManifestState::Blocked => ScreenState::Blocked,
            ManifestState::Unknown => ScreenState::Unknown,
        }
    }
}

fn default_region() -> String {
    DEFAULT_REGION.to_string()
}

#[derive(Debug)]
struct CompiledManifest {
    id: String,
    rules: Vec<CompiledRule>,
}

#[derive(Debug)]
struct CompiledRule {
    rule: ManifestRule,
    gate: CompiledGate,
}

#[derive(Debug)]
struct CompiledGate {
    all: Vec<CompiledGate>,
    any: Vec<CompiledGate>,
    not_gate: Vec<CompiledGate>,
    contains: Vec<String>,
    regex: Vec<Regex>,
    line_regex: Vec<Regex>,
}

#[derive(Debug)]
pub(crate) struct Engine {
    manifests: Vec<CompiledManifest>,
    lookup: HashMap<String, usize>,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();

const BUNDLED_MANIFESTS: &str = include_str!("manifests.json");

/// Return the compiled engine, loading the bundled manifests once.
pub(crate) fn engine() -> &'static Engine {
    ENGINE.get_or_init(|| {
        Engine::load(BUNDLED_MANIFESTS).unwrap_or_else(|error| {
            // A broken bundled bundle is a build/test failure, not a runtime
            // crash. Keep the navigator working in base/OSC mode.
            eprintln!("spaces: screen manifests unavailable: {error}");
            Engine {
                manifests: Vec::new(),
                lookup: HashMap::new(),
            }
        })
    })
}

/// Canonical manifest id for an app label, if a manifest covers it.
pub(crate) fn canonical_id(app: &str) -> Option<&'static str> {
    engine().manifest_id(app)
}

/// Whether a manifest covers this app label.
#[allow(dead_code)]
pub(crate) fn manifest_exists(app: &str) -> bool {
    engine().manifest(app).is_some()
}

/// Evaluate the screen for a supported app. Unsupported apps and screens with
/// no matching rule return a low-confidence `Unknown` result.
pub(crate) fn detect(app: &str, screen: &str, osc_title: &str, osc_progress: &str) -> Detected {
    engine().detect(app, screen, osc_title, osc_progress)
}

impl Engine {
    fn load(json: &str) -> Result<Self, String> {
        let file: ManifestFile =
            serde_json::from_str(json).map_err(|error| format!("parse manifests: {error}"))?;
        if file.manifests.is_empty() {
            return Err("manifest bundle is empty".to_string());
        }
        let _ = (&file.source, file.engine_version);
        let mut manifests = Vec::with_capacity(file.manifests.len());
        let mut lookup = HashMap::new();
        for manifest in file.manifests {
            if manifest.rules.is_empty() {
                return Err(format!("manifest {} has no rules", manifest.id));
            }
            let id = manifest.id.to_lowercase();
            let compiled = CompiledManifest {
                id: id.clone(),
                rules: compile_manifest(&manifest)?,
            };
            let index = manifests.len();
            manifests.push(compiled);
            if lookup.insert(id.clone(), index).is_some() {
                return Err(format!("duplicate manifest id {id}"));
            }
            for alias in manifest.aliases {
                lookup.entry(alias.to_lowercase()).or_insert(index);
            }
        }
        Ok(Engine { manifests, lookup })
    }

    fn manifest(&self, app: &str) -> Option<&CompiledManifest> {
        let key = normalize_app(app);
        self.lookup
            .get(&key)
            .and_then(|index| self.manifests.get(*index))
    }

    fn manifest_id(&self, app: &str) -> Option<&str> {
        self.manifest(app).map(|manifest| manifest.id.as_str())
    }

    fn detect(&self, app: &str, screen: &str, osc_title: &str, osc_progress: &str) -> Detected {
        let Some(manifest) = self.manifest(app) else {
            return Detected::unknown(None);
        };
        let input = DetectionInput {
            screen,
            osc_title,
            osc_progress,
        };
        let mut matched: Option<(&CompiledRule, ScreenState)> = None;
        for rule in &manifest.rules {
            let region_text = region(input, &rule.rule.region);
            if !compiled_gate_matches(&rule.gate, region_text) {
                continue;
            }
            let state = rule
                .rule
                .state
                .map(ScreenState::from)
                .unwrap_or(ScreenState::Unknown);
            match matched {
                Some((previous, _)) if previous.rule.priority >= rule.rule.priority => {}
                _ => matched = Some((rule, state)),
            }
        }
        let Some((rule, state)) = matched else {
            // Herdr defaults a known agent to Idle when no rule matches. For a
            // fallback we keep that result low-confidence: the caller only
            // applies Idle when `visible_idle` is set, so this behaves as
            // "no fresh evidence" rather than asserting a state.
            return Detected {
                state: ScreenState::Unknown,
                visible_idle: false,
                visible_blocker: false,
                visible_working: false,
                skip_state_update: false,
                matched_rule: None,
            };
        };
        Detected {
            state,
            visible_idle: rule.rule.visible_idle && state == ScreenState::Idle,
            visible_blocker: rule.rule.visible_blocker && state == ScreenState::Blocked,
            visible_working: rule.rule.visible_working && state == ScreenState::Working,
            skip_state_update: rule.rule.skip_state_update,
            matched_rule: Some(rule.rule.id.clone()),
        }
    }
}

fn compile_manifest(manifest: &AgentManifest) -> Result<Vec<CompiledRule>, String> {
    manifest
        .rules
        .iter()
        .map(|rule| {
            if !valid_region(&rule.region) {
                return Err(format!("rule {} uses invalid region {}", rule.id, rule.region));
            }
            if rule.region.trim().starts_with("top_non_empty_lines(")
                && manifest
                    .min_engine_version
                    .is_some_and(|version| version < TOP_NON_EMPTY_LINES_ENGINE_VERSION)
            {
                return Err(format!(
                    "rule {} uses top_non_empty_lines below engine {TOP_NON_EMPTY_LINES_ENGINE_VERSION}",
                    rule.id
                ));
            }
            let gate = compile_gate(&manifest_gate_from_rule(rule))
                .map_err(|error| format!("rule {}: {error}", rule.id))?;
            Ok(CompiledRule {
                rule: rule.clone(),
                gate,
            })
        })
        .collect()
}

fn manifest_gate_from_rule(rule: &ManifestRule) -> ManifestGate {
    ManifestGate {
        all: rule.all.clone(),
        any: rule.any.clone(),
        not_gate: rule.not_gate.clone(),
        contains: rule.contains.clone(),
        regex: rule.regex.clone(),
        line_regex: rule.line_regex.clone(),
    }
}

fn compile_gate(gate: &ManifestGate) -> Result<CompiledGate, String> {
    Ok(CompiledGate {
        all: gate
            .all
            .iter()
            .map(compile_gate)
            .collect::<Result<_, _>>()?,
        any: gate
            .any
            .iter()
            .map(compile_gate)
            .collect::<Result<_, _>>()?,
        not_gate: gate
            .not_gate
            .iter()
            .map(compile_gate)
            .collect::<Result<_, _>>()?,
        contains: gate
            .contains
            .iter()
            .map(|needle| needle.to_lowercase())
            .collect(),
        regex: gate
            .regex
            .iter()
            .map(|pattern| Regex::new(pattern).map_err(|error| error.to_string()))
            .collect::<Result<_, _>>()?,
        line_regex: gate
            .line_regex
            .iter()
            .map(|pattern| Regex::new(pattern).map_err(|error| error.to_string()))
            .collect::<Result<_, _>>()?,
    })
}

fn compiled_gate_matches(gate: &CompiledGate, text: &str) -> bool {
    let lower_text = text.to_lowercase();
    compiled_gate_matches_lower(gate, text, &lower_text)
}

fn compiled_gate_matches_lower(gate: &CompiledGate, text: &str, lower_text: &str) -> bool {
    if !gate
        .contains
        .iter()
        .all(|needle| lower_text.contains(needle))
    {
        return false;
    }
    if !gate.regex.iter().all(|regex| regex.is_match(text)) {
        return false;
    }
    if !gate
        .line_regex
        .iter()
        .all(|regex| text.lines().any(|line| regex.is_match(line)))
    {
        return false;
    }
    if !gate
        .all
        .iter()
        .all(|nested| compiled_gate_matches_lower(nested, text, lower_text))
    {
        return false;
    }
    if !gate.any.is_empty()
        && !gate
            .any
            .iter()
            .any(|nested| compiled_gate_matches_lower(nested, text, lower_text))
    {
        return false;
    }
    if gate
        .not_gate
        .iter()
        .any(|nested| compiled_gate_matches_lower(nested, text, lower_text))
    {
        return false;
    }
    true
}

#[derive(Clone, Copy)]
struct DetectionInput<'a> {
    screen: &'a str,
    osc_title: &'a str,
    osc_progress: &'a str,
}

fn valid_region(spec: &str) -> bool {
    let trimmed = spec.trim();
    matches!(
        trimmed,
        "whole_recent"
            | "after_last_prompt_marker"
            | "before_current_prompt_marker"
            | "whole_recent_without_current_prompt_marker"
            | "current_prompt_block_marker"
            | "after_current_prompt_block_marker"
            | "prompt_box_body"
            | "above_prompt_box"
            | "last_non_empty_above_prompt_box"
            | "after_last_horizontal_rule"
            | "osc_title"
            | "osc_progress"
    ) || region_count(trimmed, "bottom_lines").is_some()
        || region_count(trimmed, "bottom_non_empty_lines").is_some()
        || top_region_count(trimmed).is_some()
}

fn region<'a>(input: DetectionInput<'a>, spec: &str) -> &'a str {
    let trimmed = spec.trim();
    match trimmed {
        "osc_title" => return input.osc_title,
        "osc_progress" => return input.osc_progress,
        _ => {}
    }
    let content = input.screen;
    match trimmed {
        "whole_recent" => content,
        "after_last_prompt_marker" => after_last_prompt_marker(content),
        "before_current_prompt_marker" => before_current_prompt_marker(content),
        "whole_recent_without_current_prompt_marker" => {
            whole_recent_without_current_prompt_marker(content)
        }
        "current_prompt_block_marker" => current_prompt_block_marker(content).unwrap_or(""),
        "after_current_prompt_block_marker" => {
            after_current_prompt_block_marker(content).unwrap_or("")
        }
        "prompt_box_body" => prompt_box_body(content).unwrap_or(""),
        "above_prompt_box" => above_prompt_box(content),
        "last_non_empty_above_prompt_box" => last_non_empty_line(above_prompt_box(content)),
        "after_last_horizontal_rule" => after_last_horizontal_rule(content),
        _ => {
            if let Some(count) = region_count(trimmed, "bottom_lines") {
                return bottom_lines(content, count);
            }
            if let Some(count) = region_count(trimmed, "bottom_non_empty_lines") {
                return bottom_non_empty_lines(content, count);
            }
            if let Some(count) = top_region_count(trimmed) {
                return top_non_empty_lines(content, count);
            }
            ""
        }
    }
}

fn region_count(spec: &str, name: &str) -> Option<usize> {
    spec.strip_prefix(name)
        .and_then(|rest| rest.strip_prefix('('))
        .and_then(|rest| rest.strip_suffix(')'))
        .and_then(|count| count.parse::<usize>().ok())
}

fn top_region_count(spec: &str) -> Option<usize> {
    let count = spec
        .strip_prefix("top_non_empty_lines")?
        .strip_prefix('(')?
        .strip_suffix(')')?;
    if count.starts_with('0') || !count.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    count
        .parse::<usize>()
        .ok()
        .filter(|count| *count <= u16::MAX as usize)
}

fn bottom_lines(content: &str, count: usize) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(count);
    slice_from_line_index(content, &lines, start)
}

fn bottom_non_empty_lines(content: &str, count: usize) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    let Some(start_index) = lines
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, line)| !line.trim().is_empty())
        .take(count)
        .last()
        .map(|(index, _)| index)
    else {
        return "";
    };
    slice_from_line_index(content, &lines, start_index)
}

fn top_non_empty_lines(content: &str, count: usize) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    let Some(end_index) = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .take(count)
        .last()
        .map(|(index, _)| index)
    else {
        return "";
    };
    let byte_offset = line_start_offset(content, &lines, end_index + 1);
    &content[..byte_offset]
}

fn after_last_prompt_marker(content: &str) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    let Some(index) = lines.iter().rposition(|line| codex_prompt_line(line)) else {
        return content;
    };
    slice_from_line_index(content, &lines, index + 1)
}

fn before_current_prompt_marker(content: &str) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    let Some(index) = current_codex_prompt_index(&lines) else {
        return content;
    };
    let byte_offset = line_start_offset(content, &lines, index);
    &content[..byte_offset.min(content.len())]
}

fn whole_recent_without_current_prompt_marker(content: &str) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    if current_codex_prompt_index(&lines).is_some() {
        ""
    } else {
        content
    }
}

fn current_prompt_block_marker(content: &str) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();
    let prompt_index = current_codex_prompt_index(&lines)?;
    lines[..prompt_index]
        .iter()
        .rev()
        .find(|line| codex_block_marker_line(line))
        .copied()
}

fn after_current_prompt_block_marker(content: &str) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();
    let prompt_index = current_codex_prompt_index(&lines)?;
    let block_index = lines[..prompt_index]
        .iter()
        .rposition(|line| codex_block_marker_line(line))?;
    Some(slice_from_line_index(content, &lines, block_index))
}

fn current_codex_prompt_index(lines: &[&str]) -> Option<usize> {
    let prompt_index = lines.iter().rposition(|line| codex_prompt_line(line))?;
    if lines[prompt_index + 1..]
        .iter()
        .any(|line| codex_block_marker_line(line))
    {
        return None;
    }
    Some(prompt_index)
}

fn codex_prompt_line(line: &str) -> bool {
    line == "›" || line.starts_with("› ")
}

fn codex_block_marker_line(line: &str) -> bool {
    line.starts_with('•') || line.starts_with('■') || line.starts_with('✗') || line.starts_with('✓')
}

fn prompt_box_body(content: &str) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();
    let top = prompt_box_top_border_index(&lines)?;
    let start = line_start_offset(content, &lines, top + 1);
    let end_index = lines[top + 1..]
        .iter()
        .position(|line| is_horizontal_rule(line))
        .map(|relative| top + 1 + relative)
        .unwrap_or(lines.len());
    let end = line_start_offset(content, &lines, end_index);
    Some(&content[start.min(content.len())..end.min(content.len())])
}

fn above_prompt_box(content: &str) -> &str {
    let lines: Vec<&str> = content.lines().collect();
    let Some(top) = prompt_box_top_border_index(&lines) else {
        return content;
    };
    let end = line_start_offset(content, &lines, top);
    &content[..end.min(content.len())]
}

fn after_last_horizontal_rule(content: &str) -> &str {
    let mut last_rule_end = 0usize;
    let mut offset = 0usize;
    for line in content.lines() {
        let next_offset = offset + line.len() + 1;
        if is_horizontal_rule(line) {
            last_rule_end = next_offset.min(content.len());
        }
        offset = next_offset;
    }
    &content[last_rule_end..]
}

fn last_non_empty_line(content: &str) -> &str {
    content
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
}

fn prompt_box_top_border_index(lines: &[&str]) -> Option<usize> {
    let mut border_count = 0;
    for index in (0..lines.len()).rev() {
        if is_horizontal_rule(lines[index]) {
            border_count += 1;
            if border_count == 2 {
                return Some(index);
            }
        }
    }
    None
}

fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let rule_chars = trimmed.chars().take_while(|&ch| ch == '─').count();
    if rule_chars == 0 {
        return false;
    }
    let rule_bytes = trimmed
        .char_indices()
        .nth(rule_chars)
        .map(|(index, _)| index)
        .unwrap_or(trimmed.len());
    let suffix = trimmed[rule_bytes..].trim_start();
    suffix.is_empty() || rule_chars >= 3
}

fn slice_from_line_index<'a>(content: &'a str, lines: &[&str], index: usize) -> &'a str {
    let byte_offset = line_start_offset(content, lines, index);
    &content[byte_offset.min(content.len())..]
}

fn line_start_offset(content: &str, lines: &[&str], index: usize) -> usize {
    lines[..index.min(lines.len())]
        .iter()
        .map(|line| line.len() + 1)
        .sum::<usize>()
        .min(content.len())
}

/// Lowercase, basename, and strip common executable suffixes.
pub fn normalize_app(app: &str) -> String {
    let trimmed = app.trim();
    let basename = trimmed
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(trimmed);
    let lowered = basename.to_lowercase();
    for suffix in [".exe", ".cmd", ".bat", ".ps1", ".js", ".mjs", ".cjs"] {
        if let Some(stripped) = lowered.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    lowered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{agent_is_known, canonical_app, detect};

    #[test]
    fn all_bundled_manifests_parse_and_compile() {
        let engine = Engine::load(BUNDLED_MANIFESTS).expect("bundled manifests must compile");
        assert_eq!(engine.manifests.len(), 22);
        assert!(engine.manifest("claude-code").is_some());
        assert!(engine.manifest("cursor-agent").is_some());
        assert!(engine.manifest("antigravity").is_some());
    }

    #[test]
    fn aliases_resolve_to_canonical_ids() {
        assert_eq!(canonical_app("claude-code"), "claude");
        assert_eq!(canonical_app("cursor-agent"), "cursor");
        assert_eq!(canonical_app("antigravity"), "agy");
        assert_eq!(canonical_app("github-copilot"), "copilot");
        assert!(agent_is_known("claude-code"));
        assert!(!agent_is_known("deepseek"));
    }

    #[test]
    fn priority_gates_and_not_gates_are_preserved() {
        let json = r#"{
          "source":"test","engine_version":3,
          "manifests":[{"id":"claude","rules":[
            {"id":"low","state":"idle","priority":1,"contains":["match"]},
            {"id":"high","state":"working","priority":10,"contains":["match"],
             "all":[{"any":[{"regex":["w[io]n"]},{"contains":["fallback"]}]}],
             "not":[{"contains":["blocked"]}]},
            {"id":"line","state":"blocked","priority":20,"visible_blocker":true,"line_regex":["^exact line$"]}
          ]}]
        }"#;
        let engine = Engine::load(json).unwrap();
        let high = engine.detect("claude", "match win", "", "");
        assert_eq!(high.state, ScreenState::Working);
        assert_eq!(high.matched_rule.as_deref(), Some("high"));
        let blocked = engine.detect("claude", "match win blocked", "", "");
        assert_eq!(blocked.state, ScreenState::Idle);
        let line = engine.detect("claude", "before\nexact line\nafter", "", "");
        assert_eq!(line.state, ScreenState::Blocked);
        assert!(line.visible_blocker);
    }

    #[test]
    fn regions_extract_the_sections_manifests_use() {
        let screen = "top\n───\n│ a │\n───\nlast\n";
        assert_eq!(region(input(screen), "bottom_lines(2)"), "───\nlast\n");
        assert_eq!(region(input(screen), "bottom_non_empty_lines(1)"), "last\n");
        assert_eq!(
            region(input("one\n\n\ntwo\n"), "bottom_non_empty_lines(2)"),
            "one\n\n\ntwo\n"
        );
    }

    #[test]
    fn osc_title_regions_use_the_title_not_the_screen() {
        let screen = "screen text\n";
        let title = "✳ working";
        assert_eq!(region(input_with_title(screen, title), "osc_title"), title);
        assert_eq!(region(input_with_title(screen, title), "osc_progress"), "");
    }

    #[test]
    fn skip_rules_are_reported_without_state_update() {
        let json = r#"{
          "source":"test","engine_version":3,
          "manifests":[{"id":"claude","rules":[
            {"id":"viewer","state":"unknown","skip_state_update":true,"contains":["showing detailed transcript"]}
          ]}]
        }"#;
        let engine = Engine::load(json).unwrap();
        let detected = engine.detect("claude", "showing detailed transcript", "", "");
        assert!(detected.skip_state_update);
        assert_eq!(detected.state, ScreenState::Unknown);
    }

    #[test]
    fn known_agent_without_a_match_is_low_confidence() {
        let detected = detect("gemini", "an unrecognized shell prompt", "");
        assert_eq!(detected.state, ScreenState::Unknown);
        assert!(!detected.visible_idle);
    }

    fn input(screen: &str) -> DetectionInput<'_> {
        DetectionInput {
            screen,
            osc_title: "",
            osc_progress: "",
        }
    }

    fn input_with_title<'a>(screen: &'a str, title: &'a str) -> DetectionInput<'a> {
        DetectionInput {
            screen,
            osc_title: title,
            osc_progress: "",
        }
    }
}
