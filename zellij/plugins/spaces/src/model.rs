use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use zellij_tile::prelude::{
    AttentionStatus, GlobalAttention, PaneAttention, PaneId, PaneInfo, SessionInfo,
};

use crate::util;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    NeedsYou,
    Failed,
    Done,
    Working,
    Quiet,
    NewOutput,
    Idle,
}

/// Zellij resolves a finished turn to Idle once it has been seen, but keeps a
/// remembered Done record. The stock Inbox holds that Done item; the navigator
/// should do the same instead of showing a completed agent as idle.
fn status_from_attention(attention: &PaneAttention) -> Status {
    let effective = attention.effective_status();
    if effective == AttentionStatus::Idle {
        if let Some(remembered) = attention.remembered.as_ref() {
            match remembered.status {
                AttentionStatus::Done => return Status::Done,
                AttentionStatus::Failed => return Status::Failed,
                _ => {}
            }
        }
    }
    Status::from_attention(effective)
}

impl Status {
    pub fn from_attention(status: AttentionStatus) -> Self {
        match status {
            AttentionStatus::NeedsYou => Status::NeedsYou,
            AttentionStatus::Failed => Status::Failed,
            AttentionStatus::Done => Status::Done,
            AttentionStatus::Working => Status::Working,
            AttentionStatus::Quiet => Status::Quiet,
            AttentionStatus::NewOutput => Status::NewOutput,
            AttentionStatus::Idle => Status::Idle,
        }
    }

    pub fn priority(self) -> u8 {
        match self {
            Status::NeedsYou => 0,
            Status::Failed => 1,
            Status::Done => 2,
            Status::Working => 3,
            Status::Quiet => 4,
            Status::NewOutput => 5,
            Status::Idle => 6,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Status::NeedsYou => "●",
            Status::Failed => "✗",
            Status::Done => "✓",
            Status::Working => "◐",
            Status::Quiet => "◌",
            Status::NewOutput => "•",
            Status::Idle => "○",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Status::NeedsYou => "needs you",
            Status::Failed => "failed",
            Status::Done => "done",
            Status::Working => "working",
            Status::Quiet => "quiet",
            Status::NewOutput => "new output",
            Status::Idle => "idle",
        }
    }

    pub fn is_attention(self) -> bool {
        matches!(self, Status::NeedsYou | Status::Failed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttentionFilter {
    All,
    Needs,
    Working,
    Done,
    Idle,
}

impl AttentionFilter {
    pub fn label(self) -> &'static str {
        match self {
            AttentionFilter::All => "all",
            AttentionFilter::Needs => "blocked",
            AttentionFilter::Working => "working",
            AttentionFilter::Done => "done",
            AttentionFilter::Idle => "idle",
        }
    }

    pub fn matches(self, status: Status) -> bool {
        match self {
            AttentionFilter::All => true,
            AttentionFilter::Needs => status.is_attention(),
            AttentionFilter::Working => status == Status::Working,
            AttentionFilter::Done => status == Status::Done,
            AttentionFilter::Idle => status == Status::Idle,
        }
    }

    pub fn all() -> [AttentionFilter; 5] {
        [
            AttentionFilter::All,
            AttentionFilter::Needs,
            AttentionFilter::Working,
            AttentionFilter::Done,
            AttentionFilter::Idle,
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub needs: usize,
    pub failed: usize,
    pub working: usize,
    pub done: usize,
    pub quiet: usize,
    pub new_output: usize,
    pub idle: usize,
}

impl Counts {
    pub fn add(&mut self, status: Status) {
        match status {
            Status::NeedsYou => self.needs += 1,
            Status::Failed => self.failed += 1,
            Status::Done => self.done += 1,
            Status::Working => self.working += 1,
            Status::Quiet => self.quiet += 1,
            Status::NewOutput => self.new_output += 1,
            Status::Idle => self.idle += 1,
        }
    }

    pub fn attention_total(&self) -> usize {
        self.needs + self.failed
    }

    pub fn rollup(&self) -> Status {
        if self.needs > 0 {
            Status::NeedsYou
        } else if self.failed > 0 {
            Status::Failed
        } else if self.done > 0 {
            Status::Done
        } else if self.working > 0 {
            Status::Working
        } else if self.quiet > 0 {
            Status::Quiet
        } else if self.new_output > 0 {
            Status::NewOutput
        } else {
            Status::Idle
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct HookSummary {
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub ts_ms: u64,
    #[serde(default)]
    pub state_since_ms: u64,
    #[serde(default)]
    pub turn_started_ms: u64,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub child_count: usize,
    #[serde(default)]
    pub has_detail: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct HookTool {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub output: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct HookChild {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub since_ms: u64,
    #[serde(default)]
    pub text: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct HookDetail {
    #[serde(flatten)]
    pub summary: HookSummary,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub tool: Option<HookTool>,
    #[serde(default)]
    pub reply: Option<String>,
    #[serde(default)]
    pub children: Vec<HookChild>,
    #[serde(default)]
    pub background: usize,
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub id: u32,
    pub tab_position: usize,
    pub tab_name: String,
    pub title: String,
    pub app: String,
    pub status: Status,
    /// Status before the screen-reader fallback is applied. Recomputing from
    /// this keeps the fallback idempotent across SessionUpdate/attention
    /// events.
    pub base_status: Status,
    pub text: String,
    /// The OSC/attention text. `text` is rewritten from this on every reapply,
    /// then overridden by a fresh hook record.
    pub base_text: String,
    pub hook: Option<HookSummary>,
    pub cwd: Option<String>,
    pub age_ms: u64,
    pub base_age_ms: u64,
    pub is_focused: bool,
}

impl Pane {
    /// Text shown in the row and the detail line.
    pub fn display_text(&self) -> &str {
        &self.text
    }
}

impl HookSummary {
    pub fn is_auth_block(&self) -> bool {
        self.state == "blocked" && self.kind.as_deref() == Some("auth")
    }
}

#[derive(Clone, Debug)]
pub struct TabEntry {
    pub position: usize,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Space {
    pub name: String,
    pub is_current: bool,
    pub tabs: Vec<TabEntry>,
    pub panes: Vec<Pane>,
    pub cwd: Option<String>,
    pub counts: Counts,
    pub status: Status,
    pub connected_clients: usize,
    pub created: Duration,
}

impl Space {
    pub fn from_session_info(session: &SessionInfo, now_ms: u64) -> Space {
        let tab_names: HashMap<usize, String> = session
            .tabs
            .iter()
            .map(|tab| (tab.position, tab.name.clone()))
            .collect();
        let mut tabs: Vec<TabEntry> = session
            .tabs
            .iter()
            .map(|tab| TabEntry {
                position: tab.position,
                name: tab.name.clone(),
            })
            .collect();
        tabs.sort_by_key(|tab| tab.position);
        let mut panes = Vec::new();
        for (tab_position, pane_list) in &session.panes.panes {
            for pane in pane_list {
                if pane.is_plugin {
                    continue;
                }
                panes.push(pane_from_info(pane, *tab_position, &tab_names, now_ms));
            }
        }
        panes.sort_by_key(|pane| (pane.status.priority(), pane.tab_position, pane.id));
        let mut counts = Counts::default();
        for pane in &panes {
            counts.add(pane.status);
        }
        let cwd = panes
            .iter()
            .find(|pane| pane.is_focused)
            .and_then(|pane| pane.cwd.clone())
            .or_else(|| panes.iter().find_map(|pane| pane.cwd.clone()));
        Space {
            name: session.name.clone(),
            is_current: session.is_current_session,
            tabs,
            panes,
            cwd,
            counts,
            status: counts.rollup(),
            connected_clients: session.connected_clients,
            created: session.creation_time,
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn tab(&self, position: usize) -> Option<&TabEntry> {
        self.tabs.iter().find(|tab| tab.position == position)
    }

    pub fn tab_name(&self, position: usize) -> String {
        self.tab(position)
            .map(|tab| tab.name.clone())
            .unwrap_or_else(|| format!("Tab {}", position + 1))
    }
}

#[derive(Clone, Debug)]
pub struct ExitedSpace {
    pub name: String,
    pub age: Duration,
}

fn pane_from_info(
    pane: &PaneInfo,
    tab_position: usize,
    tab_names: &HashMap<usize, String>,
    now_ms: u64,
) -> Pane {
    let app = pane
        .attention
        .detail
        .app
        .clone()
        .or_else(|| {
            pane.program_status
                .iter()
                .rev()
                .find_map(|record| record.app.clone())
        })
        .or_else(|| {
            pane.terminal_command
                .as_deref()
                .and_then(util::command_name)
        })
        .unwrap_or_else(|| "terminal".to_owned());
    let app = if util::is_shell(&app) {
        "shell".to_owned()
    } else {
        app
    };
    let mut text = if !pane.attention.text.trim().is_empty() {
        pane.attention.text.clone()
    } else if let Some(title) = pane.attention.detail.title.clone() {
        title
    } else if let Some(body) = pane.attention.detail.body.clone() {
        body
    } else if let Some(msg) = pane
        .program_status
        .iter()
        .rev()
        .find_map(|record| record.msg.clone())
    {
        msg
    } else {
        pane.command_state
            .running
            .as_ref()
            .and_then(|running| running.command.as_ref())
            .map(|command| command.text.clone())
            .or_else(|| {
                pane.command_state
                    .last
                    .as_ref()
                    .and_then(|last| last.command.as_ref())
                    .map(|command| command.text.clone())
            })
            .unwrap_or_default()
    };
    if text.trim().is_empty() {
        if let Some(remembered) = pane.attention.remembered.as_ref() {
            if matches!(
                remembered.status,
                AttentionStatus::Done | AttentionStatus::Failed
            ) {
                text = remembered.text.clone();
            }
        }
    }
    let status = status_from_attention(&pane.attention);
    let since_ms = pane.attention.since_ms;
    let age_ms = if since_ms > 0 && now_ms >= since_ms {
        now_ms - since_ms
    } else {
        0
    };
    Pane {
        id: pane.id,
        tab_position,
        tab_name: tab_names
            .get(&tab_position)
            .cloned()
            .unwrap_or_else(|| format!("tab {}", tab_position + 1)),
        title: pane.title.clone(),
        app,
        status,
        base_status: status,
        base_text: text.clone(),
        text,
        hook: None,
        cwd: pane.command_state.cwd.as_ref().map(|cwd| cwd.path.clone()),
        age_ms,
        base_age_ms: age_ms,
        is_focused: pane.is_focused,
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

pub fn sort_spaces(spaces: &mut [Space]) {
    spaces.sort_by(|a, b| {
        b.is_current
            .cmp(&a.is_current)
            .then_with(|| a.status.priority().cmp(&b.status.priority()))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

fn session_is_better(candidate: &SessionInfo, current: &SessionInfo) -> bool {
    if candidate.is_current_session != current.is_current_session {
        return candidate.is_current_session;
    }
    let pane_count = |session: &SessionInfo| -> usize {
        session.panes.panes.values().map(|panes| panes.len()).sum()
    };
    let candidate_panes = pane_count(candidate);
    let current_panes = pane_count(current);
    if candidate_panes != current_panes {
        return candidate_panes > current_panes;
    }
    if candidate.tabs.len() != current.tabs.len() {
        return candidate.tabs.len() > current.tabs.len();
    }
    candidate.creation_time <= current.creation_time
}

pub fn build_spaces(sessions: &[SessionInfo], now_ms: u64) -> Vec<Space> {
    // Peer session snapshots can overlap during a rename or a newly created
    // session. Keep the richest snapshot for each name so the navigator never
    // shows the same space twice.
    let mut by_name: BTreeMap<String, &SessionInfo> = BTreeMap::new();
    for session in sessions {
        match by_name.get(&session.name) {
            Some(existing) if !session_is_better(session, existing) => {}
            _ => {
                by_name.insert(session.name.clone(), session);
            }
        }
    }
    let mut spaces: Vec<Space> = by_name
        .values()
        .map(|session| Space::from_session_info(session, now_ms))
        .collect();
    sort_spaces(&mut spaces);
    spaces
}

/// Zellij's `GlobalAttention` is the cross-session source of truth for agent
/// state: it carries every non-idle pane, including detached sessions. Session
/// manifests alone can be stale for sessions this client is not attached to.
pub fn apply_global_attention(spaces: &mut [Space], global: &GlobalAttention, now_ms: u64) {
    if global.attention_disabled {
        return;
    }
    for space in &mut *spaces {
        let Some(session_attention) = global
            .sessions
            .iter()
            .find(|session| session.name == space.name)
        else {
            continue;
        };
        let items: HashMap<u32, &zellij_tile::prelude::AttentionItem> = session_attention
            .attention
            .items
            .iter()
            .filter_map(|item| match item.pane_id {
                PaneId::Terminal(id) => Some((id, item)),
                PaneId::Plugin(_) => None,
            })
            .collect();
        for pane in &mut space.panes {
            if let Some(item) = items.get(&pane.id) {
                pane.base_status = status_from_attention(&item.attention);
                pane.status = pane.base_status;
                if !item.attention.text.trim().is_empty() {
                    pane.base_text = item.attention.text.clone();
                } else if item
                    .attention
                    .remembered
                    .as_ref()
                    .map(|remembered| {
                        matches!(
                            remembered.status,
                            AttentionStatus::Done | AttentionStatus::Failed
                        )
                    })
                    .unwrap_or(false)
                {
                    pane.base_text = item
                        .attention
                        .remembered
                        .as_ref()
                        .map(|remembered| remembered.text.clone())
                        .unwrap_or_default();
                }
                pane.text = pane.base_text.clone();
                if let Some(app) = item.attention.detail.app.clone() {
                    pane.app = if util::is_shell(&app) {
                        "shell".to_owned()
                    } else {
                        app
                    };
                }
                let since_ms = item.attention.since_ms;
                pane.age_ms = if since_ms > 0 && now_ms >= since_ms {
                    now_ms - since_ms
                } else {
                    0
                };
                pane.base_age_ms = pane.age_ms;
            } else {
                pane.base_status = Status::Idle;
                pane.status = Status::Idle;
                pane.age_ms = 0;
                pane.base_age_ms = 0;
                pane.base_text.clear();
                pane.text.clear();
            }
        }
        let mut counts = Counts::default();
        for pane in &space.panes {
            counts.add(pane.status);
        }
        space.counts = counts;
        space.status = counts.rollup();
        space
            .panes
            .sort_by_key(|pane| (pane.status.priority(), pane.tab_position, pane.id));
    }
    sort_spaces(spaces);
}

pub fn build_exited(resurrectable: &[(String, Duration)]) -> Vec<ExitedSpace> {
    let mut exited: Vec<ExitedSpace> = resurrectable
        .iter()
        .map(|(name, age)| ExitedSpace {
            name: name.clone(),
            age: *age,
        })
        .collect();
    exited.sort_by(|a, b| a.age.cmp(&b.age).then_with(|| a.name.cmp(&b.name)));
    exited.truncate(5);
    exited
}

pub fn space_score(space: &Space, query: &str) -> Option<u8> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Some(0);
    }
    let name = space.name.to_lowercase();
    let cwd = space.cwd.clone().unwrap_or_default().to_lowercase();
    if name == query {
        return Some(0);
    }
    if name.starts_with(&query) {
        return Some(5);
    }
    if name.contains(&query) {
        return Some(10);
    }
    if cwd.contains(&query) {
        return Some(20);
    }
    let mut best: Option<u8> = None;
    for pane in &space.panes {
        if let Some(score) = pane_score(pane, &query) {
            best = Some(best.map_or(score, |current| current.min(score)));
        }
    }
    best
}

pub fn pane_score(pane: &Pane, query: &str) -> Option<u8> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Some(0);
    }
    let app = pane.app.to_lowercase();
    let title = pane.title.to_lowercase();
    let tab = pane.tab_name.to_lowercase();
    let text = pane.text.to_lowercase();
    let cwd = pane.cwd.clone().unwrap_or_default().to_lowercase();
    let hook_text = pane
        .hook
        .as_ref()
        .map(|hook| {
            format!(
                "{} {} {} {}",
                hook.model.clone().unwrap_or_default(),
                hook.session_id.clone().unwrap_or_default(),
                hook.event,
                hook.kind.clone().unwrap_or_default()
            )
            .to_lowercase()
        })
        .unwrap_or_default();
    if app == query {
        Some(20)
    } else if app.starts_with(&query) {
        Some(25)
    } else if tab == query {
        Some(30)
    } else if tab.starts_with(&query) {
        Some(35)
    } else if title.contains(&query) {
        Some(40)
    } else if app.contains(&query) {
        Some(45)
    } else if tab.contains(&query) {
        Some(50)
    } else if text.contains(&query) {
        Some(55)
    } else if hook_text.contains(&query) {
        Some(58)
    } else if cwd.contains(&query) {
        Some(60)
    } else if pane.status.label().contains(&query) {
        Some(65)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn space(name: &str, panes: Vec<Pane>) -> Space {
        let mut counts = Counts::default();
        for pane in &panes {
            counts.add(pane.status);
        }
        Space {
            name: name.to_owned(),
            is_current: false,
            tabs: panes
                .iter()
                .map(|pane| pane.tab_position)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .map(|position| TabEntry {
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

    #[test]
    fn remembered_done_is_shown_after_a_turn() {
        let mut attention = PaneAttention::default();
        attention.status = AttentionStatus::Idle;
        attention.remembered = Some(zellij_tile::prelude::RememberedStatus {
            status: AttentionStatus::Done,
            text: "done".to_owned(),
            since_ms: 1,
        });
        assert_eq!(status_from_attention(&attention), Status::Done);
        attention.seen = true;
        assert_eq!(status_from_attention(&attention), Status::Done);
    }

    #[test]
    fn rollup_prefers_what_needs_a_person() {
        let mut counts = Counts::default();
        counts.add(Status::Working);
        counts.add(Status::Done);
        assert_eq!(counts.rollup(), Status::Done);
        counts.add(Status::NeedsYou);
        assert_eq!(counts.rollup(), Status::NeedsYou);
    }

    #[test]
    fn attention_filters_match_the_navigation_model() {
        assert!(AttentionFilter::Needs.matches(Status::NeedsYou));
        assert!(AttentionFilter::Needs.matches(Status::Failed));
        assert!(!AttentionFilter::Needs.matches(Status::Working));
        assert!(AttentionFilter::Working.matches(Status::Working));
        assert!(AttentionFilter::Done.matches(Status::Done));
        assert!(AttentionFilter::Idle.matches(Status::Idle));
        assert!(AttentionFilter::All.matches(Status::Quiet));
    }

    #[test]
    fn space_search_prefers_the_space_name_over_agent_text() {
        let dots = space(
            "dots",
            vec![pane(1, 0, Status::Working, "opencode", "editing auth")],
        );
        assert_eq!(space_score(&dots, "dots"), Some(0));
        assert_eq!(space_score(&dots, "dev"), Some(20));
        assert!(space_score(&dots, "auth").is_some());
        assert_eq!(space_score(&dots, "missing"), None);
    }

    #[test]
    fn pane_search_understands_apps_tabs_titles_and_messages() {
        let p = pane(1, 2, Status::NeedsYou, "claude", "approve the migration");
        assert_eq!(pane_score(&p, "claude"), Some(20));
        assert_eq!(pane_score(&p, "tab-2"), Some(30));
        assert_eq!(pane_score(&p, "approve"), Some(55));
        assert_eq!(pane_score(&p, "needs you"), Some(65));
        assert_eq!(pane_score(&p, "nope"), None);
    }

    #[test]
    fn duplicate_session_snapshots_are_collapsed_to_the_richest_one() {
        use zellij_tile::prelude::{PaneInfo, SessionInfo, TabInfo};
        let mut small = SessionInfo {
            name: "dots".to_owned(),
            ..Default::default()
        };
        small.tabs.push(TabInfo {
            position: 0,
            name: "shell".to_owned(),
            active: true,
            ..Default::default()
        });
        small.panes.panes.insert(0, vec![PaneInfo::default()]);
        let mut rich = small.clone();
        rich.panes.panes.insert(
            0,
            vec![
                PaneInfo {
                    id: 1,
                    ..Default::default()
                },
                PaneInfo {
                    id: 2,
                    ..Default::default()
                },
            ],
        );
        let spaces = build_spaces(&[small, rich], 0);
        assert_eq!(spaces.len(), 1);
        assert_eq!(spaces[0].panes.len(), 2);
    }

    #[test]
    fn only_the_five_most_recent_exited_spaces_are_kept() {
        let exited: Vec<(String, Duration)> = (0..8)
            .map(|index| (format!("space{}", index), Duration::from_secs(index)))
            .collect();
        let exited = build_exited(&exited);
        assert_eq!(exited.len(), 5);
        assert_eq!(exited[0].name, "space0");
        assert_eq!(exited[4].name, "space4");
    }

    #[test]
    fn current_space_sorts_first_then_attention() {
        let mut a = space("zeta", vec![pane(1, 0, Status::Idle, "shell", "")]);
        a.is_current = true;
        let b = space("alpha", vec![pane(2, 0, Status::NeedsYou, "claude", "")]);
        let c = space("beta", vec![pane(3, 0, Status::Working, "pi", "")]);
        let sessions: Vec<zellij_tile::prelude::SessionInfo> = Vec::new();
        let mut spaces = vec![b, c, a];
        spaces.sort_by(|a, b| {
            b.is_current
                .cmp(&a.is_current)
                .then_with(|| a.status.priority().cmp(&b.status.priority()))
                .then_with(|| a.name.cmp(&b.name))
        });
        assert_eq!(spaces[0].name, "zeta");
        assert_eq!(spaces[1].name, "alpha");
        assert_eq!(spaces[2].name, "beta");
        assert!(sessions.is_empty());
    }
}
