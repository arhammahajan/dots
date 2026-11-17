use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::layout;
use crate::model::{
    apply_global_attention, build_exited, build_spaces, pane_score, sort_spaces, space_score,
    AttentionFilter, Counts, ExitedSpace, HookDetail, HookSummary, Space, Status,
};
use crate::screen::{self, Detected, ScreenState};
use crate::util;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Space(String),
    Pane {
        space: String,
        tab: usize,
        pane: u32,
    },
    Exited(String),
    Create(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Esc,
    Backspace,
    Delete,
    Tab,
    F(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseInput {
    ScrollUp(usize),
    ScrollDown(usize),
    Click(usize, usize),
    DoubleClick(usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Close,
    OpenSpace {
        name: String,
        focus: Option<(usize, u32)>,
    },
    FocusPane {
        tab: usize,
        pane: u32,
    },
    CreateSpace {
        name: String,
        cwd: Option<String>,
    },
    RenameCurrent(String),
    KillSpace(String),
    DeleteExited(String),
    Refresh,
    LoadHookDetail {
        session: String,
        pane: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowKind {
    Space,
    Pane,
    ExitedHeader,
    Exited { name: String },
    Create { name: String },
}

#[derive(Clone, Debug)]
pub struct Row {
    pub kind: RowKind,
    pub target: Option<Target>,
    pub space_idx: Option<usize>,
    pub pane_idx: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct TextField {
    chars: Vec<char>,
    cursor: usize,
}

impl TextField {
    pub fn new(value: &str) -> Self {
        let chars: Vec<char> = value.chars().collect();
        let cursor = chars.len();
        TextField { chars, cursor }
    }

    pub fn value(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    pub fn insert(&mut self, ch: char) {
        self.chars.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.chars.len());
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.chars.len();
    }

    pub fn delete_word(&mut self) {
        while self.cursor > 0 && self.chars[self.cursor - 1].is_whitespace() {
            self.backspace();
        }
        while self.cursor > 0 && !self.chars[self.cursor - 1].is_whitespace() {
            self.backspace();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormField {
    Name,
    Cwd,
}

#[derive(Clone, Debug)]
pub struct NewSpaceForm {
    pub name: TextField,
    pub cwd: TextField,
    pub field: FormField,
    pub error: Option<String>,
    pub cwd_base: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RenameForm {
    pub name: TextField,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ConfirmAction {
    KillSpace {
        name: String,
        tabs: usize,
        panes: usize,
    },
    DeleteExited {
        name: String,
    },
}

#[derive(Clone, Debug)]
pub enum Mode {
    Navigate,
    Search,
    Help,
    NewSpace(NewSpaceForm),
    Rename(RenameForm),
    Confirm(ConfirmAction),
    HookDetail { scroll: usize, loading: bool },
}

/// The `zj __space-snapshot` payload. `ok` is false when `list-sessions`
/// failed; per-session `ok` is false when `list-panes` failed. The navigator
/// retains previous readings for failed sessions instead of pruning them.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub sessions: BTreeMap<String, SnapshotSession>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct SnapshotSession {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub panes: BTreeMap<String, SnapshotPane>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct SnapshotPane {
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub screen: String,
    #[serde(default)]
    pub dump_ok: bool,
}

/// `zj __space-hooks` summaries. Pane keys are strings on the wire.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct HookSnapshot {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub panes: BTreeMap<String, BTreeMap<String, HookSummary>>,
}

/// `zj __space-hooks --detail SESSION PANE` for the overlay.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct HookDetailSnapshot {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub record: Option<HookDetail>,
}

#[derive(Clone, Debug)]
struct ScreenReading {
    raw_app: String,
    canonical: String,
    detected: Detected,
    hash: u64,
}

#[derive(Default)]
struct ScreenFallback {
    readings: BTreeMap<(String, u32), ScreenReading>,
    previous: BTreeMap<(String, u32), ScreenState>,
    idle_confirmations: BTreeMap<(String, u32), u8>,
    done: BTreeMap<(String, u32), u64>,
    last_ok: BTreeMap<(String, u32), u64>,
    /// Session creation time (seconds), used to drop stale pane-id state when
    /// a session is restored under the same name.
    created: BTreeMap<String, u64>,
}

impl ScreenFallback {
    fn remove(&mut self, key: &(String, u32)) {
        self.readings.remove(key);
        self.previous.remove(key);
        self.idle_confirmations.remove(key);
        self.done.remove(key);
        self.last_ok.remove(key);
    }

    fn retain_session_panes(&mut self, session: &str, panes: &BTreeSet<u32>) {
        let keep = |key: &(String, u32)| key.0 != session || panes.contains(&key.1);
        self.readings.retain(|key, _| keep(key));
        self.previous.retain(|key, _| keep(key));
        self.idle_confirmations.retain(|key, _| keep(key));
        self.done.retain(|key, _| keep(key));
        self.last_ok.retain(|key, _| keep(key));
    }

    fn retain_sessions(&mut self, sessions: &BTreeSet<String>) {
        let keep = |key: &(String, u32)| sessions.contains(&key.0);
        self.readings.retain(|key, _| keep(key));
        self.previous.retain(|key, _| keep(key));
        self.idle_confirmations.retain(|key, _| keep(key));
        self.done.retain(|key, _| keep(key));
        self.last_ok.retain(|key, _| keep(key));
        self.created.retain(|name, _| sessions.contains(name));
    }

    fn sync_session_epochs(&mut self, spaces: &[Space]) {
        for space in spaces {
            let created = space.created.as_secs();
            match self.created.get(&space.name) {
                Some(previous) if *previous == created => {}
                Some(_) => {
                    self.clear_session(&space.name);
                    self.created.insert(space.name.clone(), created);
                }
                None => {
                    self.created.insert(space.name.clone(), created);
                }
            }
        }
    }

    fn clear_session(&mut self, session: &str) {
        let keep = |key: &(String, u32)| key.0 != session;
        self.readings.retain(|key, _| keep(key));
        self.previous.retain(|key, _| keep(key));
        self.idle_confirmations.retain(|key, _| keep(key));
        self.done.retain(|key, _| keep(key));
        self.last_ok.retain(|key, _| keep(key));
    }
}

const DONE_TTL_MS: u64 = 10 * 60 * 1000;
const STALE_SCREEN_MS: u64 = 60 * 1000;

fn reading_hash(app: &str, title: &str, screen: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    app.hash(&mut hasher);
    title.hash(&mut hasher);
    screen.hash(&mut hasher);
    hasher.finish()
}

fn idle_like(status: Status) -> bool {
    matches!(status, Status::Idle | Status::Quiet | Status::NewOutput)
}

pub struct App {
    pub spaces: Vec<Space>,
    pub exited: Vec<ExitedSpace>,
    pub current: Option<String>,
    pub mode: Mode,
    pub search: TextField,
    pub filter: AttentionFilter,
    pub selected: Option<Target>,
    pub collapsed: BTreeSet<String>,
    pub rows: Vec<Row>,
    pub toast: Option<String>,
    pub home: Option<String>,
    pub default_cwd: Option<String>,
    global: Option<zellij_tile::prelude::GlobalAttention>,
    agent_apps: BTreeMap<String, BTreeMap<u32, String>>,
    hooks: BTreeMap<String, BTreeMap<u32, HookSummary>>,
    hook_detail: Option<((String, u32), u64, HookDetail)>,
    hook_epochs: BTreeMap<String, u64>,
    screen_fallback: ScreenFallback,
    open_new_on_first_update: bool,
}

impl App {
    pub fn new(home: Option<String>) -> Self {
        let mut app = App {
            spaces: Vec::new(),
            exited: Vec::new(),
            current: None,
            mode: Mode::Navigate,
            search: TextField::new(""),
            filter: AttentionFilter::All,
            selected: None,
            collapsed: BTreeSet::new(),
            rows: Vec::new(),
            toast: None,
            home,
            default_cwd: None,
            global: None,
            agent_apps: BTreeMap::new(),
            hooks: BTreeMap::new(),
            hook_detail: None,
            hook_epochs: BTreeMap::new(),
            screen_fallback: ScreenFallback::default(),
            open_new_on_first_update: false,
        };
        app.rebuild_rows();
        app
    }

    pub fn configure_search(&mut self, query: &str) {
        self.search = TextField::new(query);
        self.mode = Mode::Search;
        self.selected = None;
        self.rebuild_rows();
    }

    pub fn request_new(&mut self) {
        self.open_new_on_first_update = true;
    }

    pub fn set_sessions(
        &mut self,
        sessions: &[zellij_tile::prelude::SessionInfo],
        resurrectable: &[(String, Duration)],
        now_ms: u64,
    ) {
        self.spaces = build_spaces(sessions, now_ms);
        if let Some(global) = self.global.as_ref() {
            apply_global_attention(&mut self.spaces, global, now_ms);
        }
        self.reapply(now_ms);
        self.exited = build_exited(resurrectable);
        self.current = self
            .spaces
            .iter()
            .find(|space| space.is_current)
            .map(|space| space.name.clone());
        if self.default_cwd.is_none() {
            self.default_cwd = self.current_space().and_then(|space| space.cwd.clone());
        }
        self.rebuild_rows();
        if self.open_new_on_first_update {
            self.open_new_on_first_update = false;
            self.open_new(None);
        }
    }

    pub fn set_global_attention(
        &mut self,
        global: zellij_tile::prelude::GlobalAttention,
        now_ms: u64,
    ) {
        apply_global_attention(&mut self.spaces, &global, now_ms);
        self.global = Some(global);
        self.reapply(now_ms);
    }

    /// Merge the live agent identity map produced by `zj __space-agents`.
    /// Zellij's peer session manifests omit program status and terminal
    /// commands, so an idle agent in another space would otherwise be labelled
    /// as a plain terminal.
    pub fn set_agent_apps(&mut self, apps: BTreeMap<String, BTreeMap<u32, String>>, now_ms: u64) {
        self.agent_apps = apps;
        self.reapply(now_ms);
    }

    /// Re-attach hook summaries to the current panes, then run the normal
    /// agent-label and screen passes. Calling this after every update keeps
    /// hook state alive across `set_sessions` rebuilding the space tree.
    fn reapply(&mut self, now_ms: u64) {
        self.sync_hook_epochs();
        self.apply_agent_apps();
        self.apply_hooks();
        self.apply_screen_fallback(now_ms);
    }

    fn sync_hook_epochs(&mut self) {
        let spaces: Vec<(String, u64)> = self
            .spaces
            .iter()
            .map(|space| (space.name.clone(), space.created.as_secs()))
            .collect();
        for (name, created) in spaces {
            match self.hook_epochs.get(&name) {
                Some(previous) if *previous == created => {}
                Some(_) => {
                    self.hooks.remove(&name);
                    self.hook_detail = None;
                    self.hook_epochs.insert(name, created);
                }
                None => {
                    self.hook_epochs.insert(name, created);
                }
            }
        }
    }

    fn apply_hooks(&mut self) {
        for index in 0..self.spaces.len() {
            let name = self.spaces[index].name.clone();
            let session_hooks = self.hooks.get(&name);
            for pane in &mut self.spaces[index].panes {
                pane.hook = session_hooks.and_then(|panes| panes.get(&pane.id)).cloned();
                if let Some(hook) = pane.hook.as_ref() {
                    let placeholder =
                        pane.app.is_empty() || pane.app == "terminal" || pane.app == "shell";
                    if placeholder && !hook.app.is_empty() {
                        pane.app = hook.app.clone();
                    }
                }
            }
        }
    }

    /// Merge `zj __space-hooks` summaries. A failed helper keeps the previous
    /// state, matching the screen-snapshot path.
    pub fn set_hook_snapshot(&mut self, snapshot: HookSnapshot, now_ms: u64) {
        if !snapshot.ok || (snapshot.version != 0 && snapshot.version != 1) {
            return;
        }
        let mut hooks = BTreeMap::new();
        for (session, panes) in snapshot.panes {
            let mut parsed = BTreeMap::new();
            for (id, summary) in panes {
                if let Ok(id) = id.parse::<u32>() {
                    parsed.insert(id, summary);
                }
            }
            hooks.insert(session, parsed);
        }
        self.hooks = hooks;
        self.reapply(now_ms);
    }

    /// Cache a full detail record only when it belongs to the summary the
    /// overlay opened on.
    pub fn set_hook_detail_loading(&mut self, loading: bool) {
        if let Mode::HookDetail { scroll, .. } = self.mode {
            self.mode = Mode::HookDetail { scroll, loading };
        }
    }

    pub fn set_hook_detail(
        &mut self,
        session: String,
        pane: u32,
        detail: HookDetail,
        _now_ms: u64,
    ) {
        let expected = self
            .hooks
            .get(&session)
            .and_then(|panes| panes.get(&pane))
            .map(|summary| summary.ts_ms);
        let matches = expected == Some(detail.summary.ts_ms);
        if matches {
            self.hook_detail = Some(((session, pane), detail.summary.ts_ms, detail));
        }
        if let Mode::HookDetail { scroll, .. } = self.mode {
            self.mode = Mode::HookDetail {
                scroll,
                loading: false,
            };
        }
    }

    pub fn hook_detail_for(&self, key: &(String, u32)) -> Option<&HookDetail> {
        self.hook_detail
            .as_ref()
            .filter(|(stored, _, _)| stored == key)
            .map(|(_, _, detail)| detail)
    }

    pub fn hook_summary_for(&self, key: &(String, u32)) -> Option<&HookSummary> {
        self.hooks.get(&key.0).and_then(|panes| panes.get(&key.1))
    }

    pub fn hook_detail_target(&self) -> Option<(String, u32)> {
        match self.selected.as_ref()? {
            Target::Pane { space, pane, .. } => Some((space.clone(), *pane)),
            _ => None,
        }
    }

    /// Merge the richer snapshot produced by `zj __space-snapshot`: live app
    /// labels plus viewport text for every agent pane in every running space.
    /// Text is re-evaluated only when it changes; failures retain the previous
    /// reading until it goes stale.
    pub fn set_screen_snapshot(&mut self, snapshot: Snapshot, now_ms: u64) {
        if !snapshot.ok {
            return;
        }
        for (session_name, session) in &snapshot.sessions {
            if !session.ok {
                continue;
            }
            let mut apps = BTreeMap::new();
            let mut pane_ids = BTreeSet::new();
            for (id_text, pane) in &session.panes {
                let Ok(id) = id_text.parse::<u32>() else {
                    continue;
                };
                pane_ids.insert(id);
                if !pane.app.trim().is_empty() {
                    apps.insert(id, pane.app.clone());
                }
            }
            if apps.is_empty() {
                self.agent_apps.remove(session_name);
            } else {
                self.agent_apps.insert(session_name.clone(), apps);
            }

            for (id_text, pane) in &session.panes {
                let Ok(id) = id_text.parse::<u32>() else {
                    continue;
                };
                let key = (session_name.clone(), id);
                if pane.app.trim().is_empty() {
                    self.screen_fallback.remove(&key);
                    continue;
                }
                let canonical = screen::canonical_app(&pane.app);
                let has_screen = pane.dump_ok && !pane.screen.trim().is_empty();
                if !has_screen {
                    let remove = match self.screen_fallback.readings.get(&key) {
                        Some(reading) if reading.canonical == canonical => {
                            let last = self
                                .screen_fallback
                                .last_ok
                                .get(&key)
                                .copied()
                                .unwrap_or(now_ms);
                            now_ms.saturating_sub(last) > STALE_SCREEN_MS
                        }
                        Some(_) => true,
                        None => false,
                    };
                    if remove {
                        self.screen_fallback.remove(&key);
                    }
                    continue;
                }

                let hash = reading_hash(&pane.app, &pane.title, &pane.screen);
                let (replace, app_changed) = match self.screen_fallback.readings.get(&key) {
                    Some(reading) => (
                        reading.hash != hash
                            || reading.canonical != canonical
                            || reading.raw_app != pane.app,
                        reading.canonical != canonical || reading.raw_app != pane.app,
                    ),
                    None => (true, true),
                };
                if replace {
                    if app_changed {
                        self.screen_fallback.previous.remove(&key);
                        self.screen_fallback.idle_confirmations.remove(&key);
                        self.screen_fallback.done.remove(&key);
                    }
                    let detected = screen::detect(&pane.app, &pane.screen, &pane.title);
                    self.screen_fallback.readings.insert(
                        key.clone(),
                        ScreenReading {
                            raw_app: pane.app.clone(),
                            canonical,
                            detected,
                            hash,
                        },
                    );
                }
                self.screen_fallback.last_ok.insert(key, now_ms);
            }
            self.screen_fallback
                .retain_session_panes(session_name, &pane_ids);
        }

        self.apply_agent_apps();
        self.apply_screen_fallback(now_ms);
    }

    /// Reapply the screen fallback over the current base statuses. Safe to call
    /// after every SessionUpdate/attention merge; it never consumes evidence.
    pub fn apply_screen_fallback(&mut self, now_ms: u64) {
        apply_fallback(&mut self.spaces, &mut self.screen_fallback, now_ms);
        sort_spaces(&mut self.spaces);
        self.rebuild_rows();
    }

    fn apply_agent_apps(&mut self) {
        for space in &mut self.spaces {
            let Some(apps) = self.agent_apps.get(&space.name) else {
                continue;
            };
            for pane in &mut space.panes {
                let Some(app) = apps.get(&pane.id) else {
                    continue;
                };
                if pane.app == "terminal" || pane.app == "shell" {
                    pane.app = app.clone();
                }
            }
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        self.rows
            .iter()
            .position(|row| row.target.as_ref() == Some(selected))
    }

    pub fn selected_row(&self) -> Option<&Row> {
        self.selected_index().and_then(|index| self.rows.get(index))
    }

    pub fn selected_space_name(&self) -> Option<String> {
        match self.selected.as_ref()? {
            Target::Space(name) => Some(name.clone()),
            Target::Pane { space, .. } => Some(space.clone()),
            _ => None,
        }
    }

    pub fn selected_space(&self) -> Option<&Space> {
        let name = self.selected_space_name()?;
        self.spaces.iter().find(|space| space.name == name)
    }

    pub fn current_space(&self) -> Option<&Space> {
        let name = self.current.as_deref()?;
        self.spaces.iter().find(|space| space.name == name)
    }

    pub fn active_filter(&self) -> AttentionFilter {
        self.filter
    }

    fn name_exists(&self, name: &str) -> bool {
        self.spaces
            .iter()
            .any(|space| space.name.eq_ignore_ascii_case(name))
            || self
                .exited
                .iter()
                .any(|space| space.name.eq_ignore_ascii_case(name))
    }

    fn name_exists_other_than(&self, name: &str, other: &str) -> bool {
        self.spaces.iter().any(|space| {
            !space.name.eq_ignore_ascii_case(other) && space.name.eq_ignore_ascii_case(name)
        }) || self
            .exited
            .iter()
            .any(|space| space.name.eq_ignore_ascii_case(name))
    }

    pub(crate) fn rebuild_rows(&mut self) {
        let previous = self.selected.clone();
        let raw_query = self.search.value().trim().to_owned();
        let query = raw_query.to_lowercase();
        let mut rows = Vec::new();
        let mut visible: Vec<(u8, usize, Vec<usize>)> = Vec::new();
        for (space_idx, space) in self.spaces.iter().enumerate() {
            let self_match = query.is_empty()
                || space.name.to_lowercase().contains(&query)
                || space
                    .cwd
                    .as_deref()
                    .map(|cwd| cwd.to_lowercase().contains(&query))
                    .unwrap_or(false);
            let matching_panes: Vec<usize> = space
                .panes
                .iter()
                .enumerate()
                .filter(|(_, pane)| self.filter.matches(pane.status))
                .filter(|(_, pane)| {
                    query.is_empty() || self_match || pane_score(pane, &query).is_some()
                })
                .map(|(pane_idx, _)| pane_idx)
                .collect();
            if !query.is_empty() && !self_match && matching_panes.is_empty() {
                continue;
            }
            if self.filter != AttentionFilter::All && matching_panes.is_empty() {
                continue;
            }
            let score = if query.is_empty() {
                0
            } else {
                space_score(space, &query).unwrap_or(130)
            };
            visible.push((score, space_idx, matching_panes));
        }
        visible.sort_by(|(score_a, idx_a, _), (score_b, idx_b, _)| {
            score_a
                .cmp(score_b)
                .then_with(|| {
                    self.spaces[*idx_b]
                        .is_current
                        .cmp(&self.spaces[*idx_a].is_current)
                })
                .then_with(|| {
                    self.spaces[*idx_a]
                        .status
                        .priority()
                        .cmp(&self.spaces[*idx_b].status.priority())
                })
                .then_with(|| {
                    self.spaces[*idx_a]
                        .name
                        .to_lowercase()
                        .cmp(&self.spaces[*idx_b].name.to_lowercase())
                })
        });
        for (_, space_idx, matching_panes) in visible {
            let space = &self.spaces[space_idx];
            rows.push(Row {
                kind: RowKind::Space,
                target: Some(Target::Space(space.name.clone())),
                space_idx: Some(space_idx),
                pane_idx: None,
            });
            let show_panes = if query.is_empty() && self.filter == AttentionFilter::All {
                !self.collapsed.contains(&space.name)
            } else {
                true
            };
            if !show_panes {
                continue;
            }
            for pane_idx in matching_panes {
                let pane = &space.panes[pane_idx];
                rows.push(Row {
                    kind: RowKind::Pane,
                    target: Some(Target::Pane {
                        space: space.name.clone(),
                        tab: pane.tab_position,
                        pane: pane.id,
                    }),
                    space_idx: Some(space_idx),
                    pane_idx: Some(pane_idx),
                });
            }
        }
        let exited: Vec<(usize, &ExitedSpace)> = if self.filter == AttentionFilter::All {
            self.exited
                .iter()
                .enumerate()
                .filter(|(_, space)| query.is_empty() || space.name.to_lowercase().contains(&query))
                .collect()
        } else {
            Vec::new()
        };
        if !exited.is_empty() {
            rows.push(Row {
                kind: RowKind::ExitedHeader,
                target: None,
                space_idx: None,
                pane_idx: None,
            });
            for (exited_idx, space) in exited {
                rows.push(Row {
                    kind: RowKind::Exited {
                        name: space.name.clone(),
                    },
                    target: Some(Target::Exited(space.name.clone())),
                    space_idx: Some(self.spaces.len() + exited_idx),
                    pane_idx: None,
                });
            }
        }
        if !query.is_empty()
            && util::sanitize_space_name(&raw_query).is_ok()
            && !self.name_exists(&raw_query)
        {
            rows.push(Row {
                kind: RowKind::Create {
                    name: raw_query.clone(),
                },
                target: Some(Target::Create(raw_query)),
                space_idx: None,
                pane_idx: None,
            });
        }
        self.rows = rows;
        let still_present = previous
            .as_ref()
            .map(|target| {
                self.rows
                    .iter()
                    .any(|row| row.target.as_ref() == Some(target))
            })
            .unwrap_or(false);
        if !still_present {
            self.selected = self.default_target();
        }
    }

    fn default_target(&self) -> Option<Target> {
        let prefer_first = !self.search.is_empty();
        if prefer_first {
            return self.rows.iter().find_map(|row| row.target.clone());
        }
        if let Some(current) = self.current.as_deref() {
            if let Some(row) = self
                .rows
                .iter()
                .find(|row| row.target == Some(Target::Space(current.to_owned())))
            {
                return row.target.clone();
            }
        }
        self.rows.iter().find_map(|row| row.target.clone())
    }

    fn selectable_indices(&self) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.target.is_some())
            .map(|(index, _)| index)
            .collect()
    }

    fn move_selection(&mut self, delta: isize) {
        let selectable = self.selectable_indices();
        if selectable.is_empty() {
            self.selected = None;
            return;
        }
        let current = self
            .selected_index()
            .and_then(|index| selectable.iter().position(|candidate| *candidate == index))
            .unwrap_or(0);
        let next = (current as isize + delta).clamp(0, selectable.len() as isize - 1) as usize;
        self.selected = self.rows[selectable[next]].target.clone();
    }

    fn select_first(&mut self) {
        self.selected = self
            .selectable_indices()
            .first()
            .and_then(|index| self.rows.get(*index).and_then(|row| row.target.clone()));
    }

    fn select_last(&mut self) {
        self.selected = self
            .selectable_indices()
            .last()
            .and_then(|index| self.rows.get(*index).and_then(|row| row.target.clone()));
    }

    fn jump_space(&mut self, direction: isize) {
        let mut spaces: Vec<(String, usize)> = Vec::new();
        for (index, row) in self.rows.iter().enumerate() {
            if matches!(row.kind, RowKind::Space) {
                if let Some(Target::Space(name)) = row.target.as_ref() {
                    spaces.push((name.clone(), index));
                }
            }
        }
        if spaces.is_empty() {
            return;
        }
        let current_name = self.selected_space_name();
        let current = current_name
            .as_ref()
            .and_then(|name| spaces.iter().position(|(candidate, _)| candidate == name))
            .unwrap_or(0);
        let next = (current as isize + direction).clamp(0, spaces.len() as isize - 1) as usize;
        let name = spaces[next].0.clone();
        self.selected = self
            .rows
            .iter()
            .find(|row| {
                row.target.is_some()
                    && matches!(
                        row.target.as_ref(),
                        Some(Target::Pane { space, .. }) if space == &name
                    )
            })
            .and_then(|row| row.target.clone())
            .or(Some(Target::Space(name)));
    }

    fn select_space_number(&mut self, number: usize) {
        let mut spaces =
            self.rows
                .iter()
                .filter_map(|row| match (&row.kind, row.target.as_ref()) {
                    (RowKind::Space, Some(Target::Space(name))) => Some(name.clone()),
                    _ => None,
                });
        if let Some(name) = spaces.nth(number.saturating_sub(1)) {
            self.selected = Some(Target::Space(name));
        }
    }

    fn toggle_collapse(&mut self) {
        let Some(name) = self.selected_space_name() else {
            return;
        };
        if !self.collapsed.remove(&name) {
            self.collapsed.insert(name.clone());
        }
        self.rebuild_rows();
        self.selected = Some(Target::Space(name));
    }

    fn set_filter(&mut self, filter: AttentionFilter) {
        self.filter = if self.filter == filter {
            AttentionFilter::All
        } else {
            filter
        };
        self.selected = None;
        self.rebuild_rows();
    }

    fn cycle_filter(&mut self, backwards: bool) {
        let filters = AttentionFilter::all();
        let current = filters
            .iter()
            .position(|filter| *filter == self.filter)
            .unwrap_or(0);
        let next = if backwards {
            (current + filters.len() - 1) % filters.len()
        } else {
            (current + 1) % filters.len()
        };
        self.filter = filters[next];
        self.selected = None;
        self.rebuild_rows();
    }

    fn enter_search(&mut self) {
        self.mode = Mode::Search;
    }

    fn open_hook_detail(&mut self) -> Vec<Effect> {
        let Some((session, pane)) = self.hook_detail_target() else {
            self.toast = Some("Select an agent pane with hook data.".to_owned());
            return vec![];
        };
        let Some(summary) = self.hook_summary_for(&(session.clone(), pane)) else {
            self.toast = Some("No hook data for this pane.".to_owned());
            return vec![];
        };
        let loading = summary.has_detail;
        self.mode = Mode::HookDetail { scroll: 0, loading };
        if loading {
            vec![Effect::LoadHookDetail { session, pane }]
        } else {
            vec![]
        }
    }

    pub(crate) fn open_new(&mut self, prefill: Option<&str>) {
        let cwd = self
            .selected_space()
            .and_then(|space| space.cwd.clone())
            .or_else(|| self.current_space().and_then(|space| space.cwd.clone()))
            .or_else(|| self.default_cwd.clone())
            .or_else(|| self.home.clone());
        let existing = self
            .spaces
            .iter()
            .map(|space| space.name.clone())
            .chain(self.exited.iter().map(|space| space.name.clone()));
        let base = prefill
            .map(|value| value.to_owned())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                cwd.as_deref()
                    .map(util::basename)
                    .unwrap_or("space")
                    .to_owned()
            });
        let base = util::sanitize_space_name(&base).unwrap_or_else(|_| "space".to_owned());
        let name = util::unique_name(&base, existing);
        self.mode = Mode::NewSpace(NewSpaceForm {
            name: TextField::new(&name),
            cwd: TextField::new(cwd.as_deref().unwrap_or("")),
            field: FormField::Name,
            error: None,
            cwd_base: cwd,
        });
    }

    fn request_rename(&mut self) {
        let Some(name) = self.selected_space_name() else {
            self.toast = Some("Select a running space to rename.".to_owned());
            return;
        };
        if self.current.as_deref() != Some(name.as_str()) {
            self.toast = Some("Switch to this space before renaming it.".to_owned());
            return;
        }
        self.mode = Mode::Rename(RenameForm {
            name: TextField::new(&name),
            error: None,
        });
    }

    fn request_kill(&mut self) {
        match self.selected.clone() {
            Some(Target::Exited(name)) => {
                self.mode = Mode::Confirm(ConfirmAction::DeleteExited { name });
            }
            Some(Target::Space(name)) | Some(Target::Pane { space: name, .. }) => {
                if self.current.as_deref() == Some(name.as_str()) {
                    self.toast = Some(
                        "This is the current space. Switch away before closing it.".to_owned(),
                    );
                    return;
                }
                let Some(space) = self.spaces.iter().find(|space| space.name == name) else {
                    return;
                };
                self.mode = Mode::Confirm(ConfirmAction::KillSpace {
                    name,
                    tabs: space.tab_count(),
                    panes: space.panes.len(),
                });
            }
            _ => {}
        }
    }

    fn activate_selected(&mut self) -> Vec<Effect> {
        let Some(target) = self.selected.clone() else {
            return vec![];
        };
        match target {
            Target::Space(name) => {
                if self.current.as_deref() == Some(name.as_str()) {
                    vec![Effect::Close]
                } else {
                    vec![Effect::OpenSpace { name, focus: None }]
                }
            }
            Target::Pane { space, tab, pane } => {
                if self.current.as_deref() == Some(space.as_str()) {
                    vec![Effect::FocusPane { tab, pane }]
                } else {
                    vec![Effect::OpenSpace {
                        name: space,
                        focus: Some((tab, pane)),
                    }]
                }
            }
            Target::Exited(name) => vec![Effect::OpenSpace { name, focus: None }],
            Target::Create(name) => {
                self.open_new(Some(&name));
                vec![]
            }
        }
    }

    pub fn handle_key(&mut self, key: Key) -> Vec<Effect> {
        self.toast = None;
        let mode = std::mem::replace(&mut self.mode, Mode::Navigate);
        match mode {
            Mode::Navigate => self.handle_navigate(key),
            Mode::Search => self.handle_search(key),
            Mode::Help => {
                if key.code == KeyCode::Esc
                    || key.code == KeyCode::F(1)
                    || matches!(key.code, KeyCode::Char('?'))
                {
                    self.mode = Mode::Navigate;
                } else {
                    self.mode = Mode::Help;
                }
                vec![]
            }
            Mode::NewSpace(form) => self.handle_new_space(key, form),
            Mode::Rename(form) => self.handle_rename(key, form),
            Mode::Confirm(action) => self.handle_confirm(key, action),
            Mode::HookDetail { scroll, loading } => {
                self.handle_hook_detail(key, scroll, loading);
                vec![]
            }
        }
    }

    fn handle_hook_detail(&mut self, key: Key, mut scroll: usize, loading: bool) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('H') => {
                self.mode = Mode::Navigate;
                return;
            }
            KeyCode::Up | KeyCode::Char('k') => scroll = scroll.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => scroll = scroll.saturating_add(1),
            KeyCode::PageUp => scroll = scroll.saturating_sub(10),
            KeyCode::PageDown => scroll = scroll.saturating_add(10),
            KeyCode::Home | KeyCode::Char('g') => scroll = 0,
            KeyCode::End | KeyCode::Char('G') => scroll = usize::MAX,
            _ => {}
        }
        self.mode = Mode::HookDetail { scroll, loading };
    }

    fn handle_navigate(&mut self, key: Key) -> Vec<Effect> {
        if key.ctrl && matches!(key.code, KeyCode::Char('r')) {
            return vec![Effect::Refresh];
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Left | KeyCode::Char('h') => self.jump_space(-1),
            KeyCode::Right | KeyCode::Char('l') => self.jump_space(1),
            KeyCode::Home | KeyCode::Char('g') => self.select_first(),
            KeyCode::End | KeyCode::Char('G') => self.select_last(),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::Tab => self.cycle_filter(key.shift),
            KeyCode::Enter => return self.activate_selected(),
            KeyCode::Esc | KeyCode::Char('q') => return vec![Effect::Close],
            KeyCode::Char('/') => self.enter_search(),
            KeyCode::Char('?') | KeyCode::F(1) => self.mode = Mode::Help,
            KeyCode::Char('H') => return self.open_hook_detail(),
            KeyCode::Char('n') => self.open_new(None),
            KeyCode::Char('r') => self.request_rename(),
            KeyCode::Char('x') | KeyCode::Delete => self.request_kill(),
            KeyCode::Char(' ') => self.toggle_collapse(),
            KeyCode::Char('b') => self.set_filter(AttentionFilter::Needs),
            KeyCode::Char('w') => self.set_filter(AttentionFilter::Working),
            KeyCode::Char('d') => self.set_filter(AttentionFilter::Done),
            KeyCode::Char('i') => self.set_filter(AttentionFilter::Idle),
            KeyCode::Char('a') => self.set_filter(AttentionFilter::All),
            KeyCode::Char(ch) if ch.is_ascii_digit() && ch != '0' => {
                self.select_space_number(ch.to_digit(10).unwrap_or(1) as usize)
            }
            _ => {}
        }
        vec![]
    }

    fn handle_search(&mut self, key: Key) -> Vec<Effect> {
        match key.code {
            KeyCode::Esc => {
                self.search.clear();
                self.selected = None;
                self.mode = Mode::Navigate;
                self.rebuild_rows();
            }
            KeyCode::Enter => {
                self.mode = Mode::Navigate;
                return self.activate_selected();
            }
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::PageDown => self.move_selection(10),
            KeyCode::Left => {
                self.search.left();
                self.mode = Mode::Search;
            }
            KeyCode::Right => {
                self.search.right();
                self.mode = Mode::Search;
            }
            KeyCode::Home => {
                self.search.home();
                self.mode = Mode::Search;
            }
            KeyCode::End => {
                self.search.end();
                self.mode = Mode::Search;
            }
            KeyCode::Backspace => {
                self.search.backspace();
                self.changed_search();
            }
            KeyCode::Delete => {
                self.search.delete();
                self.changed_search();
            }
            KeyCode::Char('u') if key.ctrl => {
                self.search.clear();
                self.changed_search();
            }
            KeyCode::Char('w') if key.ctrl => {
                self.search.delete_word();
                self.changed_search();
            }
            KeyCode::Char('a') if key.ctrl => {
                self.search.home();
                self.mode = Mode::Search;
            }
            KeyCode::Char('e') if key.ctrl => {
                self.search.end();
                self.mode = Mode::Search;
            }
            KeyCode::Char(ch) if !key.ctrl && !key.alt && !ch.is_control() => {
                self.search.insert(ch);
                self.changed_search();
            }
            _ => {
                self.mode = Mode::Search;
            }
        }
        vec![]
    }

    fn changed_search(&mut self) {
        self.mode = Mode::Search;
        self.selected = None;
        self.rebuild_rows();
    }

    fn handle_new_space(&mut self, key: Key, mut form: NewSpaceForm) -> Vec<Effect> {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Navigate;
                return vec![];
            }
            KeyCode::Enter => {
                let name = match util::sanitize_space_name(&form.name.value()) {
                    Ok(name) => name,
                    Err(error) => {
                        form.error = Some(error);
                        self.mode = Mode::NewSpace(form);
                        return vec![];
                    }
                };
                if self.name_exists(&name) {
                    form.error = Some(format!("A space named '{}' already exists.", name));
                    self.mode = Mode::NewSpace(form);
                    return vec![];
                }
                let cwd = util::normalize_path(
                    &form.cwd.value(),
                    form.cwd_base.as_deref().or(self.default_cwd.as_deref()),
                    self.home.as_deref(),
                );
                self.mode = Mode::Navigate;
                return vec![Effect::CreateSpace { name, cwd }];
            }
            KeyCode::Tab | KeyCode::Down => {
                form.field = match form.field {
                    FormField::Name => FormField::Cwd,
                    FormField::Cwd => FormField::Name,
                };
                form.error = None;
                self.mode = Mode::NewSpace(form);
                return vec![];
            }
            KeyCode::Up => {
                form.field = match form.field {
                    FormField::Name => FormField::Cwd,
                    FormField::Cwd => FormField::Name,
                };
                form.error = None;
                self.mode = Mode::NewSpace(form);
                return vec![];
            }
            _ => {}
        }
        form.error = None;
        let field = match form.field {
            FormField::Name => &mut form.name,
            FormField::Cwd => &mut form.cwd,
        };
        edit_field(field, key);
        self.mode = Mode::NewSpace(form);
        vec![]
    }

    fn handle_rename(&mut self, key: Key, mut form: RenameForm) -> Vec<Effect> {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Navigate;
                return vec![];
            }
            KeyCode::Enter => {
                let Some(old) = self.current.clone() else {
                    self.mode = Mode::Navigate;
                    return vec![];
                };
                let name = match util::sanitize_space_name(&form.name.value()) {
                    Ok(name) => name,
                    Err(error) => {
                        form.error = Some(error);
                        self.mode = Mode::Rename(form);
                        return vec![];
                    }
                };
                if self.name_exists_other_than(&name, &old) {
                    form.error = Some(format!("A space named '{}' already exists.", name));
                    self.mode = Mode::Rename(form);
                    return vec![];
                }
                if name == old {
                    self.mode = Mode::Navigate;
                    return vec![];
                }
                for space in &mut self.spaces {
                    if space.name == old {
                        space.name = name.clone();
                        space.is_current = true;
                    }
                }
                self.current = Some(name.clone());
                self.selected = Some(Target::Space(name.clone()));
                self.rebuild_rows();
                self.selected = Some(Target::Space(name.clone()));
                self.mode = Mode::Navigate;
                return vec![Effect::RenameCurrent(name)];
            }
            _ => {}
        }
        form.error = None;
        edit_field(&mut form.name, key);
        self.mode = Mode::Rename(form);
        vec![]
    }

    fn handle_confirm(&mut self, key: Key, action: ConfirmAction) -> Vec<Effect> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('n') => {
                self.mode = Mode::Navigate;
                vec![]
            }
            KeyCode::Enter | KeyCode::Char('y') => match action {
                ConfirmAction::KillSpace { name, .. } => {
                    if self.current.as_deref() == Some(name.as_str()) {
                        self.toast = Some(
                            "This is the current space. Switch away before closing it.".to_owned(),
                        );
                        self.mode = Mode::Navigate;
                        return vec![];
                    }
                    self.spaces.retain(|space| space.name != name);
                    self.selected = None;
                    self.rebuild_rows();
                    self.mode = Mode::Navigate;
                    vec![Effect::KillSpace(name)]
                }
                ConfirmAction::DeleteExited { name } => {
                    self.exited.retain(|space| space.name != name);
                    self.selected = None;
                    self.rebuild_rows();
                    self.mode = Mode::Navigate;
                    vec![Effect::DeleteExited(name)]
                }
            },
            _ => {
                self.mode = Mode::Confirm(action);
                vec![]
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseInput, rows: usize, cols: usize) -> Vec<Effect> {
        let geometry = layout::geometry(rows, cols);
        match mouse {
            MouseInput::ScrollUp(lines) => {
                self.move_selection(-(lines.max(1) as isize));
                vec![]
            }
            MouseInput::ScrollDown(lines) => {
                self.move_selection(lines.max(1) as isize);
                vec![]
            }
            MouseInput::Click(line, _) | MouseInput::DoubleClick(line, _) => {
                if matches!(
                    self.mode,
                    Mode::NewSpace(_)
                        | Mode::Rename(_)
                        | Mode::Confirm(_)
                        | Mode::Help
                        | Mode::HookDetail { .. }
                ) {
                    return vec![];
                }
                if geometry.search_y == Some(line) {
                    self.enter_search();
                    return vec![];
                }
                if line < geometry.list_top || line >= geometry.list_top + geometry.list_height {
                    return vec![];
                }
                let selected = self.selected_index().unwrap_or(0);
                let start = layout::window_start(selected, self.rows.len(), geometry.list_height);
                let row_index = start + (line - geometry.list_top);
                let Some(row) = self.rows.get(row_index) else {
                    return vec![];
                };
                let Some(target) = row.target.clone() else {
                    return vec![];
                };
                self.selected = Some(target);
                self.activate_selected()
            }
        }
    }
}

fn edit_field(field: &mut TextField, key: Key) -> bool {
    match key.code {
        KeyCode::Char('u') if key.ctrl => {
            field.clear();
            true
        }
        KeyCode::Char('w') if key.ctrl => {
            field.delete_word();
            true
        }
        KeyCode::Char('a') if key.ctrl => {
            field.home();
            true
        }
        KeyCode::Char('e') if key.ctrl => {
            field.end();
            true
        }
        KeyCode::Char(ch) if !key.ctrl && !key.alt && !ch.is_control() => {
            field.insert(ch);
            true
        }
        KeyCode::Backspace => {
            field.backspace();
            true
        }
        KeyCode::Delete => {
            field.delete();
            true
        }
        KeyCode::Left => {
            field.left();
            true
        }
        KeyCode::Right => {
            field.right();
            true
        }
        KeyCode::Home => {
            field.home();
            true
        }
        KeyCode::End => {
            field.end();
            true
        }
        _ => false,
    }
}

const HOOK_WORKING_STALE_MS: u64 = 120_000;
const HOOK_DONE_TTL_MS: u64 = 30 * 60_000;
const HOOK_FAILED_TTL_MS: u64 = 30 * 60_000;
const HOOK_BLOCK_TTL_MS: u64 = 30 * 60_000;
const HOOK_AUTH_TTL_MS: u64 = 60 * 60_000;

struct HookView<'a> {
    status: Status,
    age_ms: u64,
    text: &'a str,
}

/// Decide whether a hook summary should own this pane's state.
///
/// Hook events win while fresh; OSC 7501 fills in once the hook has gone
/// silent on a state the program has left.  Hook `idle` is deliberately weak:
/// it never overrides a live OSC/screen state, so it returns `None` and the
/// normal fallback path decides.
fn hook_view<'a>(
    record: &'a HookSummary,
    base: Status,
    base_age_ms: u64,
    now_ms: u64,
) -> Option<HookView<'a>> {
    let ts = record.ts_ms;
    let since = if record.state_since_ms > 0 {
        record.state_since_ms
    } else {
        ts
    };
    let state_age = now_ms.saturating_sub(since);
    let silence = now_ms.saturating_sub(ts);
    match record.state.as_str() {
        "working" | "idle" if silence > HOOK_WORKING_STALE_MS => return None,
        "done" if state_age > HOOK_DONE_TTL_MS => return None,
        "failed" if state_age > HOOK_FAILED_TTL_MS => return None,
        "blocked" => {
            if record.is_auth_block() {
                if state_age > HOOK_AUTH_TTL_MS {
                    return None;
                }
            } else if state_age > HOOK_BLOCK_TTL_MS {
                return None;
            }
        }
        _ => {}
    }
    let base_is_newer = base_age_ms > 0 && base_age_ms < silence;
    match record.state.as_str() {
        "working" => {
            if matches!(base, Status::Done | Status::Failed | Status::Idle) && base_is_newer {
                return None;
            }
            Some(HookView {
                status: Status::Working,
                age_ms: state_age,
                text: &record.text,
            })
        }
        "blocked" | "waiting" => {
            if !record.is_auth_block()
                && matches!(base, Status::Done | Status::Idle)
                && base_is_newer
            {
                return None;
            }
            Some(HookView {
                status: Status::NeedsYou,
                age_ms: state_age,
                text: &record.text,
            })
        }
        "done" => Some(HookView {
            status: Status::Done,
            age_ms: state_age,
            text: &record.text,
        }),
        "failed" => Some(HookView {
            status: Status::Failed,
            age_ms: state_age,
            text: &record.text,
        }),
        _ => None,
    }
}

fn apply_fallback(spaces: &mut [Space], fallback: &mut ScreenFallback, now_ms: u64) {
    // Keep per-pane readings across a stale peer manifest that temporarily
    // omits a pane; only drop state for sessions that are gone entirely.
    let sessions: BTreeSet<String> = spaces.iter().map(|space| space.name.clone()).collect();
    fallback.sync_session_epochs(spaces);
    fallback.retain_sessions(&sessions);

    for space in spaces.iter_mut() {
        let is_current = space.is_current;
        let session = space.name.clone();
        for pane in space.panes.iter_mut() {
            let key = (session.clone(), pane.id);
            let base = pane.base_status;
            let mut status = base;
            let mut clear_done = false;
            let mut had_reading = false;
            let mut stale_reading = false;

            pane.age_ms = pane.base_age_ms;
            pane.text = pane.base_text.clone();

            let hook = pane
                .hook
                .as_ref()
                .and_then(|record| hook_view(record, base, pane.base_age_ms, now_ms));

            let reading = fallback.readings.get(&key).map(|reading| {
                (
                    reading.canonical.clone(),
                    reading.raw_app.clone(),
                    reading.detected.clone(),
                )
            });
            let visible_blocker = reading
                .as_ref()
                .map(|(canonical, raw_app, detected)| {
                    let current =
                        *canonical == screen::canonical_app(&pane.app) || *raw_app == pane.app;
                    current && detected.visible_blocker
                })
                .unwrap_or(false);
            if let Some((canonical, raw_app, detected)) = reading {
                let pane_canonical = screen::canonical_app(&pane.app);
                if canonical != pane_canonical && raw_app != pane.app {
                    stale_reading = true;
                } else {
                    had_reading = true;
                    if detected.skip_state_update {
                        fallback.idle_confirmations.remove(&key);
                    } else if base == Status::Failed {
                        status = Status::Failed;
                        clear_done = true;
                        fallback.idle_confirmations.remove(&key);
                    } else {
                        let authority_active = screen::osc_authority(&pane.app) && !idle_like(base);
                        if detected.visible_blocker
                            || (detected.state == ScreenState::Blocked && !authority_active)
                        {
                            status = Status::NeedsYou;
                            clear_done = true;
                            fallback.idle_confirmations.remove(&key);
                        } else if base == Status::Done {
                            // Native Done stays until a new live turn or a
                            // visible blocker. An idle viewport must not erase
                            // Zellij's remembered finish signal.
                            status = if detected.state == ScreenState::Working
                                && detected.visible_working
                            {
                                Status::Working
                            } else {
                                Status::Done
                            };
                            clear_done = true;
                            fallback.idle_confirmations.remove(&key);
                        } else if authority_active {
                            // A complete OSC lifecycle owns working/idle.
                            status = base;
                            fallback.idle_confirmations.remove(&key);
                        } else {
                            match detected.state {
                                ScreenState::Working if detected.visible_working => {
                                    status = Status::Working;
                                    clear_done = true;
                                    fallback.idle_confirmations.remove(&key);
                                }
                                ScreenState::Idle if detected.visible_idle => {
                                    let focused = is_current && pane.is_focused;
                                    let was_working =
                                        matches!(base, Status::Working | Status::NeedsYou)
                                            || fallback.previous.get(&key)
                                                == Some(&ScreenState::Working);
                                    if was_working {
                                        if focused {
                                            status = Status::Idle;
                                            clear_done = true;
                                        } else {
                                            fallback.done.insert(key.clone(), now_ms);
                                            status = Status::Done;
                                        }
                                    } else if fallback.done.contains_key(&key) {
                                        if focused {
                                            status = Status::Idle;
                                            clear_done = true;
                                        } else {
                                            status = Status::Done;
                                        }
                                    } else {
                                        status = Status::Idle;
                                    }
                                    fallback.idle_confirmations.remove(&key);
                                }
                                ScreenState::Idle => {
                                    // Without a visible idle prompt, require
                                    // two consecutive observations before
                                    // releasing working/blocked.
                                    if matches!(base, Status::Working | Status::NeedsYou) {
                                        let count = fallback
                                            .idle_confirmations
                                            .entry(key.clone())
                                            .or_insert(0);
                                        *count = count.saturating_add(1);
                                        if *count >= 2 {
                                            status = Status::Idle;
                                            clear_done = true;
                                        } else {
                                            status = base;
                                        }
                                    } else {
                                        status = base;
                                    }
                                }
                                _ => {
                                    status = base;
                                }
                            }
                        }
                    }
                    if detected.state != ScreenState::Unknown || detected.has_evidence() {
                        fallback.previous.insert(key.clone(), detected.state);
                    }
                }
            }

            if stale_reading {
                fallback.remove(&key);
            }
            if !had_reading {
                fallback.idle_confirmations.remove(&key);
            }

            if let Some(done_at) = fallback.done.get(&key).copied() {
                let focused = is_current && pane.is_focused;
                let expired = now_ms.saturating_sub(done_at) > DONE_TTL_MS;
                if clear_done || focused || expired {
                    fallback.done.remove(&key);
                    if status == Status::Done {
                        status = Status::Idle;
                    }
                } else if status == Status::Idle {
                    status = Status::Done;
                }
                if status == Status::Done {
                    pane.age_ms = now_ms.saturating_sub(done_at);
                }
            }

            // Hooks win over OSC and screen evidence, with one exception: a
            // permission/question prompt that is actually visible on screen.
            if let Some(view) = hook {
                let hook_status = view.status;
                let hook_age = view.age_ms;
                let hook_text = view.text.to_owned();
                status = hook_status;
                pane.age_ms = hook_age;
                if !hook_text.trim().is_empty() {
                    pane.text = hook_text;
                }
                if visible_blocker && status != Status::NeedsYou {
                    status = Status::NeedsYou;
                }
            }

            pane.status = status;
        }

        let mut counts = Counts::default();
        for pane in &space.panes {
            counts.add(pane.status);
        }
        space.counts = counts;
        space.status = counts.rollup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Counts, Pane, Space, Status};

    fn pane(id: u32, tab: usize, status: Status, app: &str, text: &str) -> Pane {
        Pane {
            id,
            tab_position: tab,
            tab_name: format!("tab-{}", tab),
            title: format!("{} title", app),
            app: app.to_owned(),
            status,
            base_status: status,
            base_text: text.to_owned(),
            text: text.to_owned(),
            hook: None,
            cwd: Some("/home/me/dev/dots".to_owned()),
            age_ms: 0,
            base_age_ms: 0,
            is_focused: false,
        }
    }

    fn space(name: &str, current: bool, panes: Vec<Pane>) -> Space {
        let mut counts = Counts::default();
        for pane in &panes {
            counts.add(pane.status);
        }
        Space {
            name: name.to_owned(),
            is_current: current,
            tabs: panes
                .iter()
                .fold(BTreeSet::new(), |mut positions, pane| {
                    positions.insert(pane.tab_position);
                    positions
                })
                .into_iter()
                .map(|position| crate::model::TabEntry {
                    position,
                    name: format!("tab-{}", position),
                })
                .collect(),
            panes,
            cwd: Some("/home/me/dev/dots".to_owned()),
            counts,
            status: counts.rollup(),
            connected_clients: 1,
            created: Duration::from_secs(60),
        }
    }

    fn app_with_spaces() -> App {
        let mut app = App::new(Some("/home/me".to_owned()));
        app.spaces = vec![
            space(
                "dots",
                true,
                vec![
                    pane(1, 0, Status::Working, "opencode", "editing auth"),
                    pane(2, 1, Status::NeedsYou, "claude", "approve the migration"),
                ],
            ),
            space("server", false, vec![pane(3, 0, Status::Idle, "shell", "")]),
        ];
        app.current = Some("dots".to_owned());
        app.exited = vec![ExitedSpace {
            name: "old-work".to_owned(),
            age: Duration::from_secs(120),
        }];
        app.rebuild_rows();
        app
    }

    #[test]
    fn live_agent_apps_replace_unknown_terminal_labels() {
        let mut app = App::new(None);
        app.spaces = vec![space(
            "dots",
            true,
            vec![
                pane(7, 0, Status::Idle, "terminal", ""),
                pane(8, 0, Status::Done, "shell", ""),
            ],
        )];
        let mut apps = BTreeMap::new();
        let mut panes = BTreeMap::new();
        panes.insert(7, "claude-code".to_owned());
        apps.insert("dots".to_owned(), panes);
        app.set_agent_apps(apps, 0);
        assert_eq!(app.spaces[0].panes[0].app, "claude-code");
        assert_eq!(app.spaces[0].panes[1].app, "shell");
    }

    #[test]
    fn rows_group_agents_under_their_space_and_skip_plugin_rows() {
        let app = app_with_spaces();
        assert_eq!(app.rows.len(), 7);
        assert!(matches!(app.rows[0].kind, RowKind::Space));
        assert!(matches!(app.rows[1].kind, RowKind::Pane));
        assert!(matches!(app.rows[3].kind, RowKind::Space));
        assert!(matches!(app.rows[4].kind, RowKind::Pane));
        assert!(matches!(app.rows[5].kind, RowKind::ExitedHeader));
    }

    #[test]
    fn navigation_skips_headers_and_can_jump_between_spaces() {
        let mut app = app_with_spaces();
        app.selected = Some(Target::Space("dots".to_owned()));
        app.handle_key(Key {
            code: KeyCode::Down,
            ctrl: false,
            alt: false,
            shift: false,
        });
        assert!(matches!(app.selected, Some(Target::Pane { .. })));
        app.handle_key(Key {
            code: KeyCode::Right,
            ctrl: false,
            alt: false,
            shift: false,
        });
        assert_eq!(
            app.selected,
            Some(Target::Pane {
                space: "server".to_owned(),
                tab: 0,
                pane: 3,
            })
        );
    }

    #[test]
    fn search_filters_and_offers_to_create_a_missing_space() {
        let mut app = app_with_spaces();
        app.handle_key(Key {
            code: KeyCode::Char('/'),
            ctrl: false,
            alt: false,
            shift: false,
        });
        for ch in "newproject".chars() {
            app.handle_key(Key {
                code: KeyCode::Char(ch),
                ctrl: false,
                alt: false,
                shift: false,
            });
        }
        assert_eq!(app.search.value(), "newproject");
        assert!(app
            .rows
            .iter()
            .any(|row| matches!(row.kind, RowKind::Create { .. })));
        app.handle_key(Key {
            code: KeyCode::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        });
        assert!(matches!(app.mode, Mode::NewSpace(_)));
    }

    #[test]
    fn exact_search_match_selects_the_space_header() {
        let mut app = app_with_spaces();
        app.configure_search("dots");
        assert_eq!(app.selected, Some(Target::Space("dots".to_owned())));
        let effects = app.activate_selected();
        assert_eq!(effects, vec![Effect::Close]);
    }

    #[test]
    fn attention_filters_keep_only_matching_agents() {
        let mut app = app_with_spaces();
        app.set_filter(AttentionFilter::Needs);
        assert_eq!(app.rows.len(), 2);
        assert!(matches!(app.rows[0].kind, RowKind::Space));
        assert!(matches!(app.rows[1].kind, RowKind::Pane));
        app.set_filter(AttentionFilter::Needs);
        assert_eq!(app.filter, AttentionFilter::All);
        assert!(app.rows.len() > 2);
    }

    #[test]
    fn current_space_cannot_be_killed() {
        let mut app = app_with_spaces();
        app.selected = Some(Target::Space("dots".to_owned()));
        app.handle_key(Key {
            code: KeyCode::Char('x'),
            ctrl: false,
            alt: false,
            shift: false,
        });
        assert!(matches!(app.mode, Mode::Navigate));
        assert!(app.toast.is_some());
    }

    #[test]
    fn a_new_space_uses_the_selected_cwd_and_a_unique_name() {
        let mut app = app_with_spaces();
        app.selected = Some(Target::Space("server".to_owned()));
        app.open_new(Some("dots"));
        match &app.mode {
            Mode::NewSpace(form) => {
                assert_eq!(form.name.value(), "dots-2");
                assert_eq!(form.cwd.value(), "/home/me/dev/dots");
            }
            _ => panic!("expected new-space form"),
        }
    }

    #[test]
    fn submitting_a_new_space_returns_a_create_effect() {
        let mut app = app_with_spaces();
        app.open_new(Some("fresh"));
        let effects = app.handle_key(Key {
            code: KeyCode::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        });
        assert_eq!(
            effects,
            vec![Effect::CreateSpace {
                name: "fresh".to_owned(),
                cwd: Some("/home/me/dev/dots".to_owned()),
            }]
        );
        assert!(matches!(app.mode, Mode::Navigate));
    }

    #[test]
    fn renaming_is_only_offered_for_the_current_space() {
        let mut app = app_with_spaces();
        app.selected = Some(Target::Space("server".to_owned()));
        app.request_rename();
        assert!(matches!(app.mode, Mode::Navigate));
        assert!(app.toast.is_some());
        app.selected = Some(Target::Space("dots".to_owned()));
        app.request_rename();
        assert!(matches!(app.mode, Mode::Rename(_)));
    }

    #[test]
    fn confirmation_removes_the_space_and_emits_a_kill() {
        let mut app = app_with_spaces();
        app.selected = Some(Target::Space("server".to_owned()));
        app.request_kill();
        let effects = app.handle_key(Key {
            code: KeyCode::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        });
        assert_eq!(effects, vec![Effect::KillSpace("server".to_owned())]);
        assert!(!app.spaces.iter().any(|space| space.name == "server"));
    }

    fn snapshot(session: &str, pane_id: u32, app: &str, screen: &str, dump_ok: bool) -> Snapshot {
        let mut panes = BTreeMap::new();
        panes.insert(
            pane_id.to_string(),
            SnapshotPane {
                app: app.to_owned(),
                title: String::new(),
                screen: screen.to_owned(),
                dump_ok,
            },
        );
        let mut sessions = BTreeMap::new();
        sessions.insert(session.to_owned(), SnapshotSession { ok: true, panes });
        Snapshot { ok: true, sessions }
    }

    fn app_with_pane(status: Status, app_label: &str) -> App {
        let mut app = App::new(None);
        app.spaces = vec![space("dots", true, vec![pane(1, 0, status, app_label, "")])];
        app.current = Some("dots".to_owned());
        app.rebuild_rows();
        app
    }

    const GEMINI_WORKING: &str = "Running the command\n esc to cancel\n";
    const GEMINI_BLOCKED: &str = "│ Allow execution\n Do you want to proceed?\n";
    const CLAUDE_IDLE: &str = "\
──────────────────────────────\n\
❯ \n\
──────────────────────────────\n\
  ? for shortcuts\n";
    const CLAUDE_BLOCKER: &str = "run a dynamic workflow?\n esc to cancel\n";
    const QWEN_WORKING: &str = "⠏ Working (5s · esc to cancel)\n";
    const QWEN_IDLE: &str = "> type your message\n";

    #[test]
    fn screen_working_overrides_a_stale_idle_record() {
        let mut app = app_with_pane(Status::Idle, "gemini");
        app.set_screen_snapshot(snapshot("dots", 1, "gemini", GEMINI_WORKING, true), 1_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
    }

    #[test]
    fn screen_blocker_overrides_a_working_state() {
        let mut app = app_with_pane(Status::Working, "gemini");
        app.set_screen_snapshot(snapshot("dots", 1, "gemini", GEMINI_BLOCKED, true), 1_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::NeedsYou);
    }

    #[test]
    fn authority_apps_keep_live_osc_state_over_screen_evidence() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        app.set_screen_snapshot(snapshot("dots", 1, "claude-code", CLAUDE_IDLE, true), 1_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
    }

    #[test]
    fn visible_blockers_override_authority_working() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        app.set_screen_snapshot(
            snapshot("dots", 1, "claude-code", CLAUDE_BLOCKER, true),
            1_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::NeedsYou);
    }

    #[test]
    fn visible_blockers_override_native_done() {
        let mut app = app_with_pane(Status::Done, "claude-code");
        app.set_screen_snapshot(
            snapshot("dots", 1, "claude-code", CLAUDE_BLOCKER, true),
            1_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::NeedsYou);
    }

    #[test]
    fn native_done_is_not_downgraded_by_an_idle_viewport() {
        let mut app = app_with_pane(Status::Done, "claude-code");
        app.set_screen_snapshot(snapshot("dots", 1, "claude-code", CLAUDE_IDLE, true), 1_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Done);
    }

    #[test]
    fn working_to_visible_idle_synthesizes_done_until_focused() {
        let mut app = app_with_pane(Status::Idle, "qwen");
        app.set_screen_snapshot(snapshot("dots", 1, "qwen", QWEN_WORKING, true), 1_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);

        app.set_screen_snapshot(snapshot("dots", 1, "qwen", QWEN_IDLE, true), 2_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Done);

        app.spaces[0].panes[0].is_focused = true;
        app.set_screen_snapshot(snapshot("dots", 1, "qwen", QWEN_IDLE, true), 3_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Idle);
    }

    #[test]
    fn failed_helper_snapshots_keep_previous_state() {
        let mut app = app_with_pane(Status::Idle, "gemini");
        app.set_screen_snapshot(snapshot("dots", 1, "gemini", GEMINI_WORKING, true), 1_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);

        let mut failed = snapshot("dots", 1, "gemini", GEMINI_WORKING, true);
        failed.ok = false;
        app.set_screen_snapshot(failed, 2_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
    }

    #[test]
    fn failed_dump_retains_previous_reading_then_expires() {
        let mut app = app_with_pane(Status::Idle, "gemini");
        app.set_screen_snapshot(snapshot("dots", 1, "gemini", GEMINI_WORKING, true), 1_000);
        app.set_screen_snapshot(snapshot("dots", 1, "gemini", "", false), 2_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);

        app.set_screen_snapshot(snapshot("dots", 1, "gemini", "", false), 70_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Idle);
    }

    #[test]
    fn snapshot_payload_deserializes() {
        let payload = r#"{"ok":true,"sessions":{"dots":{"ok":true,"panes":{"2":{"app":"gemini","title":"t","screen":"esc to cancel","dump_ok":true}}}}}"#;
        let snapshot: Snapshot = serde_json::from_str(payload).unwrap();
        assert!(snapshot.ok);
        let session = snapshot.sessions.get("dots").unwrap();
        assert!(session.ok);
        let pane = session.panes.get("2").unwrap();
        assert_eq!(pane.app, "gemini");
        assert!(pane.dump_ok);
    }

    fn hook_summary(state: &str, text: &str, ts: u64) -> HookSummary {
        HookSummary {
            agent: "claude".to_owned(),
            app: "claude-code".to_owned(),
            state: state.to_owned(),
            kind: if state == "blocked" {
                Some("permission".to_owned())
            } else {
                None
            },
            event: "pre_tool".to_owned(),
            ts_ms: ts,
            state_since_ms: ts,
            text: text.to_owned(),
            ..Default::default()
        }
    }

    fn hook_snapshot(id: u32, summary: HookSummary) -> HookSnapshot {
        let mut panes = BTreeMap::new();
        panes.insert(id.to_string(), summary);
        let mut sessions = BTreeMap::new();
        sessions.insert("dots".to_owned(), panes);
        HookSnapshot {
            ok: true,
            version: 1,
            panes: sessions,
        }
    }

    fn hook_detail(summary: HookSummary, prompt: &str) -> HookDetail {
        HookDetail {
            summary,
            prompt: Some(prompt.to_owned()),
            tool: None,
            reply: None,
            children: Vec::new(),
            background: 0,
        }
    }

    #[test]
    fn fresh_hook_working_wins_over_an_idle_base() {
        let mut app = app_with_pane(Status::Idle, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("working", "Bash · cargo test", 9_000)),
            10_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
        assert_eq!(app.spaces[0].panes[0].text, "Bash · cargo test");
    }

    #[test]
    fn stale_hook_working_falls_back_to_the_base_state() {
        let mut app = app_with_pane(Status::Idle, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("working", "Bash · cargo test", 0)),
            200_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::Idle);
    }

    #[test]
    fn hook_permission_block_shows_needs_you() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(
                1,
                hook_summary("blocked", "approve Write · src/x.rs", 9_000),
            ),
            10_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::NeedsYou);
        assert_eq!(app.spaces[0].panes[0].text, "approve Write · src/x.rs");
    }

    #[test]
    fn hook_done_persists_for_thirty_minutes() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("done", "turn complete", 9_000)),
            10_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::Done);
        app.set_agent_apps(BTreeMap::new(), 29 * 60_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Done);
        app.set_agent_apps(BTreeMap::new(), 9_000 + 31 * 60_000);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
    }

    #[test]
    fn newer_osc_state_beats_an_old_hook() {
        let mut app = app_with_pane(Status::Done, "claude-code");
        app.spaces[0].panes[0].base_age_ms = 500;
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("working", "Bash · old", 9_000)),
            10_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::Done);
    }

    #[test]
    fn hook_idle_never_overrides_native_work() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("idle", "idle", 9_000)),
            10_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
    }

    #[test]
    fn visible_screen_blocker_overrides_hook_working() {
        let mut app = app_with_pane(Status::Idle, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("working", "Bash · cargo test", 9_000)),
            10_000,
        );
        app.set_screen_snapshot(
            snapshot("dots", 1, "claude-code", CLAUDE_BLOCKER, true),
            10_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::NeedsYou);
    }

    #[test]
    fn hook_summary_payload_deserializes() {
        let payload = r#"{"ok":true,"version":1,"now_ms":10,"spool_pending":1,"panes":{"dots":{"2":{"agent":"claude","app":"claude-code","state":"working","kind":null,"event":"pre_tool","ts_ms":9,"state_since_ms":9,"turn_started_ms":8,"text":"Bash · cargo test","model":"m","session_id":"s","cwd":"/tmp","child_count":2,"has_detail":true}}}}"#;
        let snapshot: HookSnapshot = serde_json::from_str(payload).unwrap();
        assert!(snapshot.ok);
        let summary = snapshot.panes.get("dots").unwrap().get("2").unwrap();
        assert_eq!(summary.state, "working");
        assert_eq!(summary.child_count, 2);
        assert_eq!(summary.model.as_deref(), Some("m"));
    }

    #[test]
    fn hook_detail_payload_deserializes() {
        let payload = r#"{"ok":true,"version":1,"record":{"agent":"claude","app":"claude-code","state":"working","kind":null,"event":"pre_tool","ts_ms":9,"state_since_ms":9,"turn_started_ms":8,"text":"Bash · cargo test","model":"m","session_id":"s","cwd":"/tmp","child_count":1,"has_detail":true,"prompt":"fix the test","tool":{"name":"Bash","input":"cargo test","output":null},"reply":null,"children":[{"id":"a1","type":"Explore","state":"working","since_ms":9,"text":"find it","ts_ms":9}],"background":0}}"#;
        let snapshot: HookDetailSnapshot = serde_json::from_str(payload).unwrap();
        let record = snapshot.record.unwrap();
        assert_eq!(record.prompt.as_deref(), Some("fix the test"));
        assert_eq!(
            record.tool.as_ref().map(|tool| tool.name.as_str()),
            Some("Bash")
        );
        assert_eq!(record.children.len(), 1);
        assert_eq!(record.children[0].kind, "Explore");
    }

    #[test]
    fn hook_model_and_session_are_searchable() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        let mut summary = hook_summary("working", "Bash · cargo test", 9_000);
        summary.model = Some("specialmodel".to_owned());
        summary.session_id = Some("session-1234".to_owned());
        app.set_hook_snapshot(hook_snapshot(1, summary), 10_000);
        assert_eq!(
            pane_score(&app.spaces[0].panes[0], "specialmodel"),
            Some(58)
        );
        assert_eq!(
            pane_score(&app.spaces[0].panes[0], "session-1234"),
            Some(58)
        );
    }

    #[test]
    fn hook_state_survives_a_reapply() {
        let mut app = app_with_pane(Status::Idle, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("working", "Bash · cargo test", 9_000)),
            10_000,
        );
        app.set_agent_apps(BTreeMap::new(), 10_001);
        assert_eq!(app.spaces[0].panes[0].status, Status::Working);
    }

    #[test]
    fn a_recreated_session_drops_old_hook_epochs() {
        let mut app = app_with_pane(Status::Idle, "claude-code");
        app.set_hook_snapshot(
            hook_snapshot(1, hook_summary("working", "Bash · cargo test", 9_000)),
            10_000,
        );
        app.spaces[0].created = Duration::from_secs(120);
        app.set_agent_apps(BTreeMap::new(), 10_001);
        assert_eq!(app.spaces[0].panes[0].status, Status::Idle);
    }

    #[test]
    fn hook_detail_is_cached_only_for_the_current_summary() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        let summary = hook_summary("working", "Bash · cargo test", 9_000);
        app.set_hook_snapshot(hook_snapshot(1, summary.clone()), 10_000);
        let mut stale = hook_detail(summary.clone(), "old prompt");
        stale.summary.ts_ms = 8_000;
        app.set_hook_detail("dots".to_owned(), 1, stale, 10_000);
        assert!(app.hook_detail_for(&("dots".to_owned(), 1)).is_none());
        app.set_hook_detail(
            "dots".to_owned(),
            1,
            hook_detail(summary, "fix the tests"),
            10_000,
        );
        let detail = app.hook_detail_for(&("dots".to_owned(), 1)).unwrap();
        assert_eq!(detail.prompt.as_deref(), Some("fix the tests"));
    }

    #[test]
    fn hook_overlay_renders_the_prompt_and_tool() {
        let mut app = app_with_pane(Status::Working, "claude-code");
        let summary = hook_summary("working", "Bash · cargo test", 9_000);
        app.set_hook_snapshot(hook_snapshot(1, summary.clone()), 10_000);
        let mut detail = hook_detail(summary, "fix the failing test");
        detail.tool = Some(crate::model::HookTool {
            name: "Bash".to_owned(),
            input: "cargo test".to_owned(),
            output: None,
        });
        app.set_hook_detail("dots".to_owned(), 1, detail, 10_000);
        app.selected = Some(Target::Pane {
            space: "dots".to_owned(),
            tab: 0,
            pane: 1,
        });
        app.mode = Mode::HookDetail {
            scroll: 0,
            loading: false,
        };
        let rendered = crate::ui::render_lines(&app, 30, 100).join("\n");
        assert!(rendered.contains("agent hooks"));
        assert!(rendered.contains("fix the failing test"));
        assert!(rendered.contains("cargo test"));
    }

    #[test]
    fn unknown_screens_do_not_make_a_pane_work() {
        let mut app = app_with_pane(Status::Idle, "gemini");
        app.set_screen_snapshot(
            snapshot("dots", 1, "gemini", "$ ls\nCargo.toml\n$ ", true),
            1_000,
        );
        assert_eq!(app.spaces[0].panes[0].status, Status::Idle);
    }
}
