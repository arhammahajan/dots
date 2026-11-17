mod app;
mod canvas;
mod layout;
mod model;
mod screen;
mod theme;
mod ui;
mod util;

use std::collections::BTreeMap;
use std::path::PathBuf;

use zellij_tile::prelude::*;

use app::{App, Effect, Key, KeyCode, MouseInput};

/// The plugin has two roles:
///
/// * the **opener** is the floating pane launched by a keybinding. It opens the
///   real popup and exits immediately.
/// * the **popup** is the navigator itself. Zellij gives popups a focus and key
///   capture layer, so keystrokes never reach the terminal underneath.
struct SpacesPlugin {
    app: App,
    rows: usize,
    cols: usize,
    visible: bool,
    granted: bool,
    cursor_hidden: bool,
    target: Option<String>,
    target_wait: u8,
    pending_deletes: Vec<(String, u8)>,
    restoring: Option<PendingRestore>,
    agents_refreshing: bool,
    agents_last_ms: u64,
    hooks_refreshing: bool,
    hooks_last_ms: u64,
    screen_fallback: bool,
    popup_child: bool,
    close_after_open: bool,
    configuration: BTreeMap<String, String>,
}

struct PendingRestore {
    name: String,
    focus: Option<(usize, u32)>,
    wait: u8,
}

impl Default for SpacesPlugin {
    fn default() -> Self {
        let home = std::env::var("HOME").ok();
        SpacesPlugin {
            app: App::new(home),
            rows: 0,
            cols: 0,
            visible: true,
            granted: false,
            cursor_hidden: false,
            target: None,
            target_wait: 0,
            pending_deletes: Vec::new(),
            restoring: None,
            agents_refreshing: false,
            agents_last_ms: 0,
            hooks_refreshing: false,
            hooks_last_ms: 0,
            screen_fallback: true,
            popup_child: false,
            close_after_open: false,
            configuration: BTreeMap::new(),
        }
    }
}

impl ZellijPlugin for SpacesPlugin {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.configuration = configuration.clone();
        self.popup_child = configuration
            .get("popup_child")
            .map(|value| value == "true")
            .unwrap_or(false);

        if !self.popup_child {
            // Opener mode. It only needs enough state to place the real popup,
            // but it must request the same permissions as the popup child so
            // the permission cache keeps the full set for this plugin path.
            subscribe(&[EventType::PermissionRequestResult, EventType::Timer]);
            request_permission(&[
                PermissionType::ReadApplicationState,
                PermissionType::ChangeApplicationState,
                PermissionType::RunCommands,
            ]);
            set_selectable(false);
            show_cursor(None);
            self.cursor_hidden = true;
            return;
        }

        if configuration
            .get("open_new")
            .map(|value| value == "true")
            .unwrap_or(false)
        {
            self.app.request_new();
        }
        if let Some(target) = configuration.get("target") {
            self.target = Some(target.clone());
        }
        if let Some(query) = configuration.get("query") {
            self.app.configure_search(query);
        } else if configuration.get("mode").map(|mode| mode.as_str()) == Some("search") {
            self.app.configure_search("");
        }
        self.screen_fallback = configuration
            .get("screen_fallback")
            .map(|value| value != "false")
            .unwrap_or(true);
        subscribe(&[
            EventType::Key,
            EventType::Mouse,
            EventType::SessionUpdate,
            EventType::GlobalAttentionUpdate,
            EventType::Timer,
            EventType::Visible,
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
        ]);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        set_selectable(true);
        show_cursor(None);
        self.cursor_hidden = true;
        set_timeout(0.25);
    }

    fn update(&mut self, event: Event) -> bool {
        if !self.popup_child {
            match event {
                Event::PermissionRequestResult(status) => match status {
                    PermissionStatus::Granted => self.open_child_popup(),
                    PermissionStatus::Denied => close_self(),
                },
                Event::Timer(_) if self.close_after_open => close_self(),
                _ => {}
            }
            return false;
        }

        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.granted = true;
                self.agents_last_ms = 0;
                self.hooks_last_ms = 0;
                self.refresh();
                self.refresh_agent_apps();
                self.refresh_hooks();
                self.maybe_run_target();
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.app.toast = Some(
                    "Spaces needs ReadApplicationState, ChangeApplicationState and RunCommands."
                        .to_owned(),
                );
                self.fallback_restore();
                true
            }
            Event::SessionUpdate(sessions, resurrectable) => {
                self.app
                    .set_sessions(&sessions, &resurrectable, model::now_ms());
                self.maybe_run_target();
                true
            }
            Event::GlobalAttentionUpdate(global) => {
                self.app.set_global_attention(global, model::now_ms());
                true
            }
            Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                if context.contains_key("zj_snapshot") {
                    self.finish_snapshot(exit_code, stdout);
                } else if context.contains_key("zj_agents") {
                    self.finish_agents_refresh(exit_code, stdout);
                } else if context.contains_key("zj_hooks") {
                    self.finish_hooks(exit_code, stdout);
                } else if context.contains_key("zj_hook_detail") {
                    self.finish_hook_detail(exit_code, stdout, &context);
                } else {
                    self.finish_restore(exit_code, stderr, context);
                }
                true
            }
            Event::Key(key) => {
                if let Some(key) = map_key(key) {
                    let effects = self.app.handle_key(key);
                    self.run_effects(effects);
                    true
                } else {
                    false
                }
            }
            Event::Mouse(mouse) => {
                if let Some(mouse) = map_mouse(mouse) {
                    let effects = self.app.handle_mouse(mouse, self.rows, self.cols);
                    self.run_effects(effects);
                    true
                } else {
                    false
                }
            }
            Event::Timer(_) => {
                let urgent = self.target.is_some()
                    || !self.pending_deletes.is_empty()
                    || self.restoring.is_some();
                set_timeout(if urgent { 0.3 } else { 2.0 });
                let restore_timed_out = if let Some(restoring) = self.restoring.as_mut() {
                    restoring.wait = restoring.wait.saturating_add(1);
                    restoring.wait > 100
                } else {
                    false
                };
                if restore_timed_out {
                    self.fallback_restore();
                    return true;
                }
                if self.visible && self.granted {
                    self.refresh();
                    self.refresh_agent_apps();
                    self.refresh_hooks();
                    self.retry_pending_deletes();
                    self.maybe_run_target();
                    true
                } else {
                    false
                }
            }
            Event::Visible(visible) => {
                self.visible = visible;
                if visible {
                    self.agents_last_ms = 0;
                    self.hooks_last_ms = 0;
                    if self.granted {
                        self.refresh();
                        self.refresh_agent_apps();
                        self.refresh_hooks();
                    }
                    set_timeout(2.0);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        if !self.popup_child {
            return;
        }
        self.rows = rows;
        self.cols = cols;
        if !self.cursor_hidden {
            show_cursor(None);
            self.cursor_hidden = true;
        }
        print!("{}", ui::render(&self.app, rows, cols));
    }
}

impl SpacesPlugin {
    fn open_child_popup(&mut self) {
        let (screen_rows, screen_cols) = self.screen_viewport().unwrap_or((30, 100));
        let width = (screen_cols * 86 / 100)
            .max(40)
            .min(screen_cols.saturating_sub(2));
        let height = (screen_rows * 82 / 100)
            .max(10)
            .min(screen_rows.saturating_sub(2));
        let x = screen_cols.saturating_sub(width) / 2;
        let y = screen_rows.saturating_sub(height) / 2;

        // OpenPluginPopup coordinates are relative to the requesting plugin.
        // Move the transient opener to the top-left corner first, so the
        // centered screen coordinates below are not offset by its own position.
        let plugin_id = get_plugin_ids().plugin_id;
        let opener_pane = get_focused_pane_info()
            .ok()
            .map(|(_, pane_id)| pane_id)
            .filter(|pane_id| matches!(pane_id, PaneId::Plugin(_)))
            .unwrap_or(PaneId::Plugin(plugin_id));
        if get_pane_info(opener_pane).is_some() {
            if let Some(coordinates) = FloatingPaneCoordinates::new(
                Some("0".to_owned()),
                Some("0".to_owned()),
                Some("1".to_owned()),
                Some("1".to_owned()),
                None,
                Some(true),
            ) {
                change_floating_panes_coordinates(vec![(opener_pane, coordinates)]);
            }
        }

        let mut configuration = self.configuration.clone();
        configuration.insert("popup_child".to_owned(), "true".to_owned());

        let options = PopupOptions::default()
            .focused(true)
            .capture_all_keys(true)
            .outside_click(OutsideClick::Close);
        open_plugin_popup_with_options("spacesy", configuration, y, x, width, height, options);
        close_self();
    }

    fn screen_viewport(&self) -> Option<(usize, usize)> {
        let snapshot = get_session_list().ok()?;
        let session = snapshot
            .live_sessions
            .iter()
            .find(|session| session.is_current_session)
            .or_else(|| snapshot.live_sessions.first())?;
        let tab = session
            .tabs
            .iter()
            .find(|tab| tab.active)
            .or_else(|| session.tabs.first())?;
        let rows = tab.display_area_rows.max(tab.viewport_rows);
        let cols = tab.display_area_columns.max(tab.viewport_columns);
        if rows == 0 || cols == 0 {
            None
        } else {
            Some((rows, cols))
        }
    }

    fn refresh(&mut self) {
        match get_session_list() {
            Ok(snapshot) => {
                self.app.set_sessions(
                    &snapshot.live_sessions,
                    &snapshot.resurrectable_sessions,
                    model::now_ms(),
                );
            }
            Err(error) => {
                self.app.toast = Some(format!("Could not read spaces: {}", error));
            }
        }
    }

    fn refresh_agent_apps(&mut self) {
        if !self.granted {
            return;
        }
        let now = model::now_ms();
        if self.agents_refreshing {
            // Do not wedge the navigator if a helper result never arrives.
            if now.saturating_sub(self.agents_last_ms) < 15_000 {
                return;
            }
            self.agents_refreshing = false;
        }
        if now.saturating_sub(self.agents_last_ms) < 5_000 {
            return;
        }
        self.agents_refreshing = true;
        self.agents_last_ms = now;
        let mut context = BTreeMap::new();
        if self.screen_fallback {
            context.insert("zj_snapshot".to_owned(), "1".to_owned());
            run_command(&["zj", "__space-snapshot"], context);
        } else {
            context.insert("zj_agents".to_owned(), "1".to_owned());
            run_command(&["zj", "__space-agents"], context);
        }
    }

    fn finish_snapshot(&mut self, exit_code: Option<i32>, stdout: Vec<u8>) {
        self.agents_refreshing = false;
        if !matches!(exit_code, Some(0)) {
            return;
        }
        let Ok(snapshot) = serde_json::from_slice::<app::Snapshot>(&stdout) else {
            return;
        };
        self.app.set_screen_snapshot(snapshot, model::now_ms());
    }

    fn finish_agents_refresh(&mut self, exit_code: Option<i32>, stdout: Vec<u8>) {
        self.agents_refreshing = false;
        if !matches!(exit_code, Some(0)) {
            return;
        }
        let Ok(raw) = serde_json::from_slice::<BTreeMap<String, BTreeMap<String, String>>>(&stdout)
        else {
            return;
        };
        let apps: BTreeMap<String, BTreeMap<u32, String>> = raw
            .into_iter()
            .map(|(session, panes)| {
                let panes = panes
                    .into_iter()
                    .filter_map(|(id, app)| id.parse::<u32>().ok().map(|id| (id, app)))
                    .collect();
                (session, panes)
            })
            .collect();
        self.app.set_agent_apps(apps, model::now_ms());
    }

    fn refresh_hooks(&mut self) {
        if !self.granted {
            return;
        }
        let now = model::now_ms();
        if self.hooks_refreshing {
            if now.saturating_sub(self.hooks_last_ms) < 15_000 {
                return;
            }
            self.hooks_refreshing = false;
        }
        if now.saturating_sub(self.hooks_last_ms) < 1_500 {
            return;
        }
        self.hooks_refreshing = true;
        self.hooks_last_ms = now;
        let mut context = BTreeMap::new();
        context.insert("zj_hooks".to_owned(), "1".to_owned());
        run_command(&["zj", "__space-hooks"], context);
    }

    fn finish_hooks(&mut self, exit_code: Option<i32>, stdout: Vec<u8>) {
        self.hooks_refreshing = false;
        if !matches!(exit_code, Some(0)) {
            return;
        }
        let Ok(snapshot) = serde_json::from_slice::<app::HookSnapshot>(&stdout) else {
            return;
        };
        self.app.set_hook_snapshot(snapshot, model::now_ms());
    }

    fn finish_hook_detail(
        &mut self,
        exit_code: Option<i32>,
        stdout: Vec<u8>,
        context: &BTreeMap<String, String>,
    ) {
        self.app.set_hook_detail_loading(false);
        if !matches!(exit_code, Some(0)) {
            return;
        }
        let Some(session) = context.get("session").cloned() else {
            return;
        };
        let Some(pane) = context
            .get("pane")
            .and_then(|value| value.parse::<u32>().ok())
        else {
            return;
        };
        let Ok(snapshot) = serde_json::from_slice::<app::HookDetailSnapshot>(&stdout) else {
            return;
        };
        if !snapshot.ok || (snapshot.version != 0 && snapshot.version != 1) {
            return;
        }
        let Some(record) = snapshot.record else {
            return;
        };
        self.app
            .set_hook_detail(session, pane, record, model::now_ms());
    }

    fn maybe_run_target(&mut self) {
        if !self.granted {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        let exists = self.app.spaces.iter().any(|space| space.name == target)
            || self.app.exited.iter().any(|space| space.name == target)
            || get_session_list()
                .map(|snapshot| {
                    snapshot
                        .resurrectable_sessions
                        .iter()
                        .any(|(session, _)| session == &target)
                })
                .unwrap_or(false);
        if exists {
            self.target = None;
            self.target_wait = 0;
            self.app.toast = None;
            self.run_effects(vec![Effect::OpenSpace {
                name: target,
                focus: None,
            }]);
        } else {
            self.target_wait = self.target_wait.saturating_add(1);
            if self.target_wait > 20 {
                self.target = None;
                self.target_wait = 0;
                self.app.configure_search(&target);
            }
        }
    }

    fn retry_pending_deletes(&mut self) {
        if self.pending_deletes.is_empty() {
            return;
        }
        let mut remaining = Vec::new();
        for (name, attempts) in self.pending_deletes.drain(..) {
            if delete_dead_session(&name).is_ok() || attempts >= 10 {
                continue;
            }
            remaining.push((name, attempts + 1));
        }
        self.pending_deletes = remaining;
    }

    /// Open a space. Exited spaces are prepared first: the helper strips the
    /// `start_suspended` markers from the saved layout so Zellij runs every
    /// saved command immediately instead of pausing on a "Waiting to run"
    /// banner. The actual session switch happens when the helper exits.
    fn open_space(&mut self, name: String, focus: Option<(usize, u32)>) {
        if self.app.spaces.iter().any(|space| space.name == name) {
            self.switch_space(&name, focus);
            return;
        }
        let is_exited = self.app.exited.iter().any(|space| space.name == name)
            || get_session_list()
                .map(|snapshot| {
                    snapshot
                        .resurrectable_sessions
                        .iter()
                        .any(|(session, _)| session == &name)
                })
                .unwrap_or(false);
        if is_exited {
            let mut context = BTreeMap::new();
            context.insert("zj_restore".to_owned(), name.clone());
            self.app.toast = Some(format!("Restoring {}...", name));
            self.restoring = Some(PendingRestore {
                name: name.clone(),
                focus,
                wait: 0,
            });
            run_command_with_env_variables_and_cwd(
                &["zj", "__prepare-restore", &name],
                BTreeMap::new(),
                PathBuf::from("."),
                context,
            );
            set_timeout(0.3);
            return;
        }
        self.switch_space(&name, focus);
    }

    fn switch_space(&mut self, name: &str, focus: Option<(usize, u32)>) {
        if let Some((tab, pane)) = focus {
            switch_session_with_focus(name, Some(tab), Some((pane, false)));
        } else {
            switch_session(Some(name));
        }
        close_self();
    }

    fn finish_restore(
        &mut self,
        exit_code: Option<i32>,
        stderr: Vec<u8>,
        context: BTreeMap<String, String>,
    ) {
        let Some(pending) = self.restoring.take() else {
            return;
        };
        if context.get("zj_restore") != Some(&pending.name) {
            self.restoring = Some(pending);
            return;
        }
        if !matches!(exit_code, Some(0)) {
            let message = String::from_utf8_lossy(&stderr).trim().to_owned();
            self.app.toast = Some(if message.is_empty() {
                "Could not prepare this space for restore.".to_owned()
            } else {
                format!("Restore preparation failed: {}", message)
            });
        }
        self.switch_space(&pending.name, pending.focus);
    }

    fn fallback_restore(&mut self) {
        let Some(pending) = self.restoring.take() else {
            return;
        };
        self.switch_space(&pending.name, pending.focus);
    }

    fn run_effects(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Close => close_self(),
                Effect::OpenSpace { name, focus } => {
                    self.open_space(name, focus);
                }
                Effect::FocusPane { tab, pane } => {
                    go_to_tab(tab as u32 + 1);
                    focus_terminal_pane(pane, true, false);
                    close_self();
                }
                Effect::CreateSpace { name, cwd } => {
                    switch_session_with_cwd(Some(&name), cwd.map(PathBuf::from));
                    close_self();
                }
                Effect::RenameCurrent(name) => rename_session(&name),
                Effect::KillSpace(name) => match kill_sessions(&[name.as_str()]) {
                    Ok(()) => {
                        if delete_dead_session(&name).is_err() {
                            self.pending_deletes.push((name, 0));
                            set_timeout(0.3);
                        }
                    }
                    Err(error) => {
                        self.app.toast = Some(format!("Could not close '{}': {}", name, error));
                    }
                },
                Effect::DeleteExited(name) => {
                    if let Err(error) = delete_dead_session(&name) {
                        self.app.toast = Some(format!("Could not delete '{}': {}", name, error));
                    }
                }
                Effect::Refresh => self.refresh(),
                Effect::LoadHookDetail { session, pane } => {
                    let pane_text = pane.to_string();
                    let mut context = BTreeMap::new();
                    context.insert("zj_hook_detail".to_owned(), "1".to_owned());
                    context.insert("session".to_owned(), session.clone());
                    context.insert("pane".to_owned(), pane_text.clone());
                    run_command(
                        &[
                            "zj",
                            "__space-hooks",
                            "--detail",
                            session.as_str(),
                            pane_text.as_str(),
                        ],
                        context,
                    );
                }
            }
        }
    }
}

fn map_key(key: KeyWithModifier) -> Option<Key> {
    let ctrl = key.key_modifiers.contains(&KeyModifier::Ctrl);
    let alt = key.key_modifiers.contains(&KeyModifier::Alt);
    let shift = key.key_modifiers.contains(&KeyModifier::Shift);
    let code = match key.bare_key {
        BareKey::Char(ch) => {
            let ch = if shift && ch.is_ascii_lowercase() {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            KeyCode::Char(ch)
        }
        BareKey::Up => KeyCode::Up,
        BareKey::Down => KeyCode::Down,
        BareKey::Left => KeyCode::Left,
        BareKey::Right => KeyCode::Right,
        BareKey::Home => KeyCode::Home,
        BareKey::End => KeyCode::End,
        BareKey::PageUp => KeyCode::PageUp,
        BareKey::PageDown => KeyCode::PageDown,
        BareKey::Enter => KeyCode::Enter,
        BareKey::Esc => KeyCode::Esc,
        BareKey::Backspace => KeyCode::Backspace,
        BareKey::Delete => KeyCode::Delete,
        BareKey::Tab => KeyCode::Tab,
        BareKey::F(index) => KeyCode::F(index),
        _ => return None,
    };
    Some(Key {
        code,
        ctrl,
        alt,
        shift,
    })
}

fn map_mouse(mouse: Mouse) -> Option<MouseInput> {
    match mouse {
        Mouse::ScrollUp(lines) => Some(MouseInput::ScrollUp(lines)),
        Mouse::ScrollDown(lines) => Some(MouseInput::ScrollDown(lines)),
        Mouse::LeftClick(line, column) if line >= 0 => {
            Some(MouseInput::Click(line as usize, column))
        }
        Mouse::DoubleClick(line, column) if line >= 0 => {
            Some(MouseInput::DoubleClick(line as usize, column))
        }
        _ => None,
    }
}

register_plugin!(SpacesPlugin);
