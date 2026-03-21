use std::io;

use anyhow::Result;
use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
};

use contentforge_core::{
    Content, ContentStatus, ContentType, Platform, PlatformAccount, PlatformAdaptation,
    PlatformCredential,
};
use contentforge_db::{
    repo::{AdaptationRepo, ContentRepo, PlatformAccountRepo, PublicationRepo},
    DbPool,
};
use uuid::Uuid;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const TICK_RATE_MS: u64 = 80;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Color palette
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const ACCENT: Color = Color::Rgb(99, 102, 241); // indigo
const SUCCESS: Color = Color::Rgb(34, 197, 94); // green
const WARNING: Color = Color::Rgb(234, 179, 8); // yellow
const INFO: Color = Color::Rgb(59, 130, 246); // blue
const DANGER: Color = Color::Rgb(239, 68, 68); // red
const MUTED: Color = Color::Rgb(107, 114, 128); // gray
const SURFACE: Color = Color::Rgb(30, 30, 46); // dark bg
const SURFACE_HL: Color = Color::Rgb(45, 45, 65); // highlight row
const TEXT: Color = Color::Rgb(226, 232, 240); // light text
const TEXT_DIM: Color = Color::Rgb(148, 163, 184); // dim text
const BORDER: Color = Color::Rgb(63, 63, 80); // border

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tabs
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Drafts,
    Adapt,
    Publish,
    Platforms,
}

impl Tab {
    const ALL: [Tab; 5] = [
        Tab::Dashboard,
        Tab::Drafts,
        Tab::Adapt,
        Tab::Publish,
        Tab::Platforms,
    ];

    fn title(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Drafts => "Drafts",
            Tab::Adapt => "Adapt",
            Tab::Publish => "Publish",
            Tab::Platforms => "Platforms",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Tab::Dashboard => " \u{25a3} ", // box
            Tab::Drafts => " \u{270e} ",    // pencil
            Tab::Adapt => " \u{2b82} ",     // arrows
            Tab::Publish => " \u{2191} ",   // up arrow
            Tab::Platforms => " \u{2630} ", // trigram
        }
    }

    fn next(&self) -> Tab {
        let idx = Tab::ALL.iter().position(|t| t == self).unwrap_or(0);
        Tab::ALL[(idx + 1) % Tab::ALL.len()]
    }

    fn prev(&self) -> Tab {
        let idx = Tab::ALL.iter().position(|t| t == self).unwrap_or(0);
        if idx == 0 {
            Tab::ALL[Tab::ALL.len() - 1]
        } else {
            Tab::ALL[idx - 1]
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// App mode
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// Text input mode for new draft creation. Fields: (title, body, active_field)
    InputNewDraft,
    /// Viewing details of selected content in a popup
    ViewDetail,
    /// Search/filter mode
    Search,
    /// Help overlay
    Help,
    /// Confirm delete
    ConfirmDelete,
    /// Select platform for adaptation
    SelectPlatform,
    /// Publishing confirmation
    PublishConfirm,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Dashboard data
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Default)]
pub struct DashboardData {
    idea_count: i64,
    drafting_count: i64,
    review_count: i64,
    ready_count: i64,
    scheduled_count: i64,
    published_count: i64,
    archived_count: i64,
    total_count: i64,
    recent: Vec<Content>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// App state
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct App {
    pub db: DbPool,
    pub active_tab: Tab,
    pub should_quit: bool,
    pub mode: Mode,

    // List navigation
    pub selected_index: usize,
    pub scroll_offset: usize,

    // Data caches
    pub all_content: Vec<Content>,
    pub filtered_content: Vec<Content>,
    pub dashboard: DashboardData,
    pub platform_accounts: Vec<PlatformAccount>,
    pub selected_content: Option<Content>,

    // Input fields for new draft
    pub input_title: String,
    pub input_body: String,
    pub input_field: usize, // 0 = title, 1 = body, 2 = type select

    // Search
    pub search_query: String,

    // Adapt tab
    pub adapt_platform_idx: usize,

    // Publish tab
    pub publish_items: Vec<Content>,
    pub publish_platform_idx: usize,

    // Status message at bottom
    pub status_message: Option<(String, std::time::Instant)>,

    // Content type selector for new draft
    pub content_type_idx: usize,

    // Track if data needs refresh
    data_dirty: bool,
    last_tab: Option<Tab>,
}

impl App {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            active_tab: Tab::Dashboard,
            should_quit: false,
            mode: Mode::Normal,
            selected_index: 0,
            scroll_offset: 0,
            all_content: Vec::new(),
            filtered_content: Vec::new(),
            dashboard: DashboardData::default(),
            platform_accounts: Vec::new(),
            selected_content: None,
            input_title: String::new(),
            input_body: String::new(),
            input_field: 0,
            search_query: String::new(),
            adapt_platform_idx: 0,
            publish_items: Vec::new(),
            publish_platform_idx: 0,
            status_message: None,
            content_type_idx: 0,
            data_dirty: true,
            last_tab: None,
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    fn refresh_data(&mut self) {
        let tab_changed = self.last_tab != Some(self.active_tab);
        if !self.data_dirty && !tab_changed {
            return;
        }
        self.last_tab = Some(self.active_tab);
        self.data_dirty = false;

        match self.active_tab {
            Tab::Dashboard => self.load_dashboard(),
            Tab::Drafts => self.load_all_content(),
            Tab::Adapt => {
                self.load_all_content();
                self.load_selected_content_full();
            }
            Tab::Publish => self.load_publish_items(),
            Tab::Platforms => self.load_platform_accounts(),
        }
    }

    fn load_dashboard(&mut self) {
        let repo = ContentRepo::new(self.db.clone());
        if let Ok(counts) = repo.count_by_status() {
            self.dashboard = DashboardData::default();
            for (status_str, count) in &counts {
                // status_str is JSON-encoded e.g. "\"idea\""
                let clean = status_str.trim_matches('"');
                match clean {
                    "idea" => self.dashboard.idea_count = *count,
                    "drafting" => self.dashboard.drafting_count = *count,
                    "review" => self.dashboard.review_count = *count,
                    "ready" => self.dashboard.ready_count = *count,
                    "scheduled" => self.dashboard.scheduled_count = *count,
                    "published" => self.dashboard.published_count = *count,
                    "archived" => self.dashboard.archived_count = *count,
                    _ => {}
                }
                self.dashboard.total_count += count;
            }
        }
        if let Ok(all) = repo.list_all() {
            self.dashboard.recent = all.into_iter().take(10).collect();
        }
    }

    fn load_all_content(&mut self) {
        let repo = ContentRepo::new(self.db.clone());
        if let Ok(items) = repo.list_all() {
            self.all_content = items;
            self.apply_filter();
        }
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_content = self.all_content.clone();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_content = self
                .all_content
                .iter()
                .filter(|c| {
                    c.title.to_lowercase().contains(&q)
                        || c.tags.iter().any(|t| t.to_lowercase().contains(&q))
                        || c.status.to_string().contains(&q)
                        || c.content_type.to_string().contains(&q)
                })
                .cloned()
                .collect();
        }
        if self.selected_index >= self.filtered_content.len() {
            self.selected_index = self.filtered_content.len().saturating_sub(1);
        }
    }

    fn load_selected_content_full(&mut self) {
        if self.filtered_content.is_empty() {
            self.selected_content = None;
            return;
        }
        let idx = self
            .selected_index
            .min(self.filtered_content.len().saturating_sub(1));
        let id = self.filtered_content[idx].id;
        let repo = ContentRepo::new(self.db.clone());
        self.selected_content = repo.get_by_id_full(id).ok().flatten();
    }

    fn load_publish_items(&mut self) {
        let repo = ContentRepo::new(self.db.clone());
        let adapt_repo = AdaptationRepo::new(self.db.clone());
        if let Ok(all) = repo.list_all() {
            self.publish_items = all
                .into_iter()
                .filter(|c| matches!(c.status, ContentStatus::Ready | ContentStatus::Scheduled))
                .map(|mut c| {
                    if let Ok(adaptations) = adapt_repo.list_for_content(c.id) {
                        c.adaptations = adaptations;
                    }
                    c
                })
                .filter(|c| !c.adaptations.is_empty())
                .collect();
        }
        if self.selected_index >= self.publish_items.len() {
            self.selected_index = self.publish_items.len().saturating_sub(1);
        }
    }

    fn load_platform_accounts(&mut self) {
        let repo = PlatformAccountRepo::new(self.db.clone());
        if let Ok(accounts) = repo.list_all() {
            self.platform_accounts = accounts;
        }
    }

    fn create_new_draft(&mut self) {
        let title = self.input_title.trim().to_string();
        if title.is_empty() {
            self.set_status("Title cannot be empty");
            return;
        }
        let body = self.input_body.trim().to_string();
        let ct = content_type_options()[self.content_type_idx];
        let content = Content::new(title, body, ct);
        let repo = ContentRepo::new(self.db.clone());
        match repo.insert(&content) {
            Ok(()) => {
                self.set_status(format!("Created draft: {}", &content.title));
                self.input_title.clear();
                self.input_body.clear();
                self.input_field = 0;
                self.content_type_idx = 0;
                self.mode = Mode::Normal;
                self.data_dirty = true;
            }
            Err(e) => {
                self.set_status(format!("Error creating draft: {e}"));
            }
        }
    }

    fn delete_selected(&mut self) {
        if self.filtered_content.is_empty() {
            return;
        }
        let idx = self
            .selected_index
            .min(self.filtered_content.len().saturating_sub(1));
        let content = &self.filtered_content[idx];
        let repo = ContentRepo::new(self.db.clone());
        match repo.delete(content.id) {
            Ok(()) => {
                self.set_status(format!("Deleted: {}", content.title));
                self.mode = Mode::Normal;
                self.data_dirty = true;
                self.load_all_content();
            }
            Err(e) => {
                self.set_status(format!("Error deleting: {e}"));
                self.mode = Mode::Normal;
            }
        }
    }

    fn create_adaptation_for_platform(&mut self, platform: Platform) {
        let content = match &self.selected_content {
            Some(c) => c.clone(),
            None => {
                self.set_status("No content selected");
                return;
            }
        };

        // Create a basic adaptation from the canonical content
        let char_limit = platform.char_limit();
        let mut adapted_body = content.body.clone();
        if let Some(limit) = char_limit {
            if adapted_body.len() > limit {
                adapted_body = adapted_body[..limit.saturating_sub(3)].to_string();
                adapted_body.push_str("...");
            }
        }

        let adaptation = PlatformAdaptation {
            platform,
            title: Some(content.title.clone()),
            body: adapted_body,
            thread_parts: None,
            canonical_url: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        let repo = AdaptationRepo::new(self.db.clone());
        match repo.upsert(content.id, &adaptation) {
            Ok(()) => {
                self.set_status(format!("Adapted for {}", platform));
                self.data_dirty = true;
                self.load_selected_content_full();
            }
            Err(e) => {
                self.set_status(format!("Error: {e}"));
            }
        }
    }

    fn publish_selected(&mut self) {
        if self.publish_items.is_empty() {
            return;
        }
        let idx = self
            .selected_index
            .min(self.publish_items.len().saturating_sub(1));
        let content = &self.publish_items[idx];

        if content.adaptations.is_empty() {
            self.set_status("No adaptations to publish");
            return;
        }

        let pidx = self
            .publish_platform_idx
            .min(content.adaptations.len().saturating_sub(1));
        let adaptation = &content.adaptations[pidx];
        let platform = adaptation.platform;

        let publication = contentforge_core::Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform,
            url: format!(
                "https://{}.example.com/{}",
                platform.to_string().to_lowercase().replace('/', "-"),
                &content.id.to_string()[..8]
            ),
            platform_post_id: Uuid::new_v4().to_string(),
            published_at: Utc::now(),
        };

        let pub_repo = PublicationRepo::new(self.db.clone());
        let content_repo = ContentRepo::new(self.db.clone());

        match pub_repo.insert(&publication) {
            Ok(()) => {
                let _ = content_repo.update_status(content.id, ContentStatus::Published);
                self.set_status(format!("Published to {} -> {}", platform, publication.url));
                self.mode = Mode::Normal;
                self.data_dirty = true;
            }
            Err(e) => {
                self.set_status(format!("Publish error: {e}"));
                self.mode = Mode::Normal;
            }
        }
    }

    fn list_len(&self) -> usize {
        match self.active_tab {
            Tab::Dashboard => self.dashboard.recent.len(),
            Tab::Drafts => self.filtered_content.len(),
            Tab::Adapt => self.filtered_content.len(),
            Tab::Publish => self.publish_items.len(),
            Tab::Platforms => self.platform_accounts.len(),
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        let len = self.list_len();
        if len > 0 && self.selected_index < len - 1 {
            self.selected_index += 1;
        }
    }

    // ──────────────────────────────────────────────────────────────────────
    // Event handling
    // ──────────────────────────────────────────────────────────────────────

    fn handle_event(&mut self, ev: Event) {
        if let Event::Key(key) = ev {
            if key.kind != KeyEventKind::Press {
                return;
            }
            match &self.mode {
                Mode::Normal => self.handle_normal(key.code, key.modifiers),
                Mode::InputNewDraft => self.handle_input(key.code, key.modifiers),
                Mode::ViewDetail => self.handle_popup(key.code),
                Mode::Search => self.handle_search(key.code),
                Mode::Help => self.handle_help(key.code),
                Mode::ConfirmDelete => self.handle_confirm_delete(key.code),
                Mode::SelectPlatform => self.handle_select_platform(key.code),
                Mode::PublishConfirm => self.handle_publish_confirm(key.code),
            }
        }
    }

    fn handle_normal(&mut self, code: KeyCode, _mods: KeyModifiers) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.active_tab = self.active_tab.next();
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::BackTab => {
                self.active_tab = self.active_tab.prev();
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::Char('1') => {
                self.active_tab = Tab::Dashboard;
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::Char('2') => {
                self.active_tab = Tab::Drafts;
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::Char('3') => {
                self.active_tab = Tab::Adapt;
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::Char('4') => {
                self.active_tab = Tab::Publish;
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::Char('5') => {
                self.active_tab = Tab::Platforms;
                self.selected_index = 0;
                self.data_dirty = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Char('/') if self.active_tab == Tab::Drafts => {
                self.mode = Mode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('n') if self.active_tab == Tab::Drafts => {
                self.mode = Mode::InputNewDraft;
                self.input_title.clear();
                self.input_body.clear();
                self.input_field = 0;
                self.content_type_idx = 0;
            }
            KeyCode::Char('d') if self.active_tab == Tab::Drafts => {
                if !self.filtered_content.is_empty() {
                    self.mode = Mode::ConfirmDelete;
                }
            }
            KeyCode::Enter => self.handle_enter(),
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.apply_filter();
                }
            }
            _ => {}
        }
    }

    fn handle_enter(&mut self) {
        match self.active_tab {
            Tab::Dashboard | Tab::Drafts => {
                let items = if self.active_tab == Tab::Dashboard {
                    &self.dashboard.recent
                } else {
                    &self.filtered_content
                };
                if !items.is_empty() {
                    let idx = self.selected_index.min(items.len().saturating_sub(1));
                    let id = items[idx].id;
                    let repo = ContentRepo::new(self.db.clone());
                    self.selected_content = repo.get_by_id_full(id).ok().flatten();
                    self.mode = Mode::ViewDetail;
                }
            }
            Tab::Adapt => {
                if !self.filtered_content.is_empty() {
                    self.load_selected_content_full();
                    self.mode = Mode::SelectPlatform;
                    self.adapt_platform_idx = 0;
                }
            }
            Tab::Publish => {
                if !self.publish_items.is_empty() {
                    self.publish_platform_idx = 0;
                    self.mode = Mode::PublishConfirm;
                }
            }
            Tab::Platforms => {}
        }
    }

    fn handle_input(&mut self, code: KeyCode, _mods: KeyModifiers) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.input_field = (self.input_field + 1) % 3;
            }
            KeyCode::BackTab => {
                self.input_field = if self.input_field == 0 {
                    2
                } else {
                    self.input_field - 1
                };
            }
            KeyCode::Enter => {
                if self.input_field == 2 {
                    // On type selector, Enter cycles through types
                    self.content_type_idx =
                        (self.content_type_idx + 1) % content_type_options().len();
                } else if self.input_field == 1 {
                    self.input_body.push('\n');
                } else {
                    // Submit from title field
                    self.create_new_draft();
                }
            }
            KeyCode::Char(c) => {
                match self.input_field {
                    0 => self.input_title.push(c),
                    1 => self.input_body.push(c),
                    2 => {
                        // Left/right to change type
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => match self.input_field {
                0 => {
                    self.input_title.pop();
                }
                1 => {
                    self.input_body.pop();
                }
                _ => {}
            },
            KeyCode::Left if self.input_field == 2 => {
                if self.content_type_idx == 0 {
                    self.content_type_idx = content_type_options().len() - 1;
                } else {
                    self.content_type_idx -= 1;
                }
            }
            KeyCode::Right if self.input_field == 2 => {
                self.content_type_idx = (self.content_type_idx + 1) % content_type_options().len();
            }
            _ => {}
        }
    }

    fn handle_popup(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn handle_search(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                self.apply_filter();
                self.mode = Mode::Normal;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_filter();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_filter();
            }
            _ => {}
        }
    }

    fn handle_help(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_confirm_delete(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.delete_selected();
            }
            _ => {
                self.mode = Mode::Normal;
            }
        }
    }

    fn handle_select_platform(&mut self, code: KeyCode) {
        let platforms = all_platforms();
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.adapt_platform_idx < platforms.len().saturating_sub(1) {
                    self.adapt_platform_idx += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.adapt_platform_idx = self.adapt_platform_idx.saturating_sub(1);
            }
            KeyCode::Enter => {
                let platform = platforms[self.adapt_platform_idx];
                self.create_adaptation_for_platform(platform);
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_publish_confirm(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self
                    .publish_items
                    .get(self.selected_index)
                    .map(|c| c.adaptations.len())
                    .unwrap_or(0);
                if len > 0 && self.publish_platform_idx < len - 1 {
                    self.publish_platform_idx += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.publish_platform_idx = self.publish_platform_idx.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('y') => {
                self.publish_selected();
            }
            _ => {}
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn content_type_options() -> Vec<ContentType> {
    vec![
        ContentType::Article,
        ContentType::ShortPost,
        ContentType::Thread,
        ContentType::Video,
        ContentType::ImagePost,
        ContentType::LinkShare,
    ]
}

fn all_platforms() -> Vec<Platform> {
    vec![
        Platform::Twitter,
        Platform::LinkedIn,
        Platform::DevTo,
        Platform::Medium,
        Platform::YouTube,
        Platform::Instagram,
        Platform::Substack,
        Platform::HackerNews,
        Platform::Reddit,
    ]
}

fn status_color(status: &ContentStatus) -> Color {
    match status {
        ContentStatus::Idea => MUTED,
        ContentStatus::Drafting => WARNING,
        ContentStatus::Review => Color::Rgb(168, 85, 247), // purple
        ContentStatus::Ready => INFO,
        ContentStatus::Scheduled => Color::Rgb(20, 184, 166), // teal
        ContentStatus::Published => SUCCESS,
        ContentStatus::Archived => Color::Rgb(75, 85, 99),
    }
}

fn status_label(status: &ContentStatus) -> &'static str {
    match status {
        ContentStatus::Idea => "IDEA",
        ContentStatus::Drafting => "DRAFTING",
        ContentStatus::Review => "REVIEW",
        ContentStatus::Ready => "READY",
        ContentStatus::Scheduled => "SCHEDULED",
        ContentStatus::Published => "PUBLISHED",
        ContentStatus::Archived => "ARCHIVED",
    }
}

fn short_id(id: &Uuid) -> String {
    id.to_string()[..8].to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn credential_type_label(cred: &PlatformCredential) -> &'static str {
    match cred {
        PlatformCredential::ApiKey { .. } => "API Key",
        PlatformCredential::OAuth2 { .. } => "OAuth2",
        PlatformCredential::IntegrationToken { .. } => "Token",
        PlatformCredential::Cookie { .. } => "Cookie",
        PlatformCredential::MastodonAuth { .. } => "Mastodon",
        PlatformCredential::BlueskyAuth { .. } => "Bluesky",
    }
}

fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(format!(" {title} "))
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Drawing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Overall background
    frame.render_widget(Block::default().style(Style::default().bg(SURFACE)), area);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // top bar
            Constraint::Length(3), // tab bar
            Constraint::Min(6),    // main content
            Constraint::Length(1), // bottom bar
        ])
        .split(area);

    draw_title_bar(frame, app, outer[0]);
    draw_tab_bar(frame, app, outer[1]);
    draw_main(frame, app, outer[2]);
    draw_status_bar(frame, app, outer[3]);

    // Overlays
    match &app.mode {
        Mode::ViewDetail => draw_detail_popup(frame, app),
        Mode::Help => draw_help_overlay(frame),
        Mode::ConfirmDelete => draw_confirm_delete(frame, app),
        Mode::SelectPlatform => draw_platform_selector(frame, app),
        Mode::PublishConfirm => draw_publish_confirm(frame, app),
        Mode::InputNewDraft => draw_new_draft_overlay(frame, app),
        Mode::Search => {} // search bar is inline
        Mode::Normal => {}
    }
}

fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Min(20)])
        .split(inner);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " ContentForge ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        Span::styled(
            format!("{}{}", app.active_tab.icon(), app.active_tab.title()),
            Style::default().fg(TEXT),
        ),
    ]));
    frame.render_widget(title, layout[0]);

    let now = Utc::now().format("%Y-%m-%d %H:%M UTC");
    let clock = Paragraph::new(Span::styled(
        format!("{now} "),
        Style::default().fg(TEXT_DIM),
    ))
    .alignment(Alignment::Right);
    frame.render_widget(clock, layout[1]);
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let active_idx = Tab::ALL
        .iter()
        .position(|t| *t == app.active_tab)
        .unwrap_or(0);
    let tab_titles: Vec<Line> = Tab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let num = format!(" {} ", i + 1);
            let name = format!("{} ", t.title());
            if i == active_idx {
                Line::from(vec![
                    Span::styled(
                        num,
                        Style::default()
                            .fg(SURFACE)
                            .bg(ACCENT)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        name,
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(num, Style::default().fg(TEXT_DIM)),
                    Span::styled(name, Style::default().fg(TEXT_DIM)),
                ])
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(BORDER)),
        )
        .select(active_idx)
        .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .divider(Span::styled(" \u{2502} ", Style::default().fg(BORDER)));

    frame.render_widget(tabs, area);
}

fn draw_main(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.active_tab {
        Tab::Dashboard => draw_dashboard(frame, app, area),
        Tab::Drafts => draw_drafts(frame, app, area),
        Tab::Adapt => draw_adapt(frame, app, area),
        Tab::Publish => draw_publish(frame, app, area),
        Tab::Platforms => draw_platforms(frame, app, area),
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    // Show status message if recent (within 5 seconds)
    if let Some((msg, when)) = &app.status_message {
        if when.elapsed().as_secs() < 5 {
            let status = Paragraph::new(Span::styled(
                format!(" {msg}"),
                Style::default().fg(WARNING).add_modifier(Modifier::ITALIC),
            ))
            .style(Style::default().bg(Color::Rgb(40, 40, 30)));
            frame.render_widget(status, area);
            return;
        }
    }

    let hints = match app.mode {
        Mode::Search => " / Search  \u{2502}  Enter Confirm  \u{2502}  Esc Cancel",
        Mode::InputNewDraft => " Tab Switch Field  \u{2502}  Enter Submit/Newline  \u{2502}  Esc Cancel",
        _ => match app.active_tab {
            Tab::Dashboard => " q Quit  \u{2502}  Tab/1-5 Switch  \u{2502}  j/k Navigate  \u{2502}  Enter View  \u{2502}  ? Help",
            Tab::Drafts => " q Quit  \u{2502}  n New  \u{2502}  d Delete  \u{2502}  / Search  \u{2502}  Enter View  \u{2502}  ? Help",
            Tab::Adapt => " q Quit  \u{2502}  j/k Navigate  \u{2502}  Enter Adapt  \u{2502}  ? Help",
            Tab::Publish => " q Quit  \u{2502}  j/k Navigate  \u{2502}  Enter Publish  \u{2502}  ? Help",
            Tab::Platforms => " q Quit  \u{2502}  j/k Navigate  \u{2502}  ? Help",
        },
    };

    let bar = Paragraph::new(Span::styled(hints, Style::default().fg(TEXT_DIM)))
        .style(Style::default().bg(Color::Rgb(25, 25, 40)));
    frame.render_widget(bar, area);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tab: Dashboard
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // status boxes
            Constraint::Length(3), // summary
            Constraint::Min(4),    // recent
        ])
        .split(area);

    // Status count boxes
    draw_status_boxes(frame, app, chunks[0]);

    // Quick summary
    let d = &app.dashboard;
    let summary_text = Line::from(vec![
        Span::styled(" Total: ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!("{}", d.total_count),
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  \u{2502}  ", Style::default().fg(BORDER)),
        Span::styled("Active: ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!("{}", d.drafting_count + d.review_count + d.ready_count),
            Style::default().fg(INFO).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  \u{2502}  ", Style::default().fg(BORDER)),
        Span::styled("Pipeline: ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!(
                "{} drafting \u{2192} {} ready \u{2192} {} published",
                d.drafting_count, d.ready_count, d.published_count
            ),
            Style::default().fg(TEXT),
        ),
    ]);
    let summary = Paragraph::new(summary_text)
        .block(styled_block("Summary"))
        .alignment(Alignment::Left);
    frame.render_widget(summary, chunks[1]);

    // Recent content
    draw_recent_list(frame, app, chunks[2]);
}

fn draw_status_boxes(frame: &mut Frame, app: &App, area: Rect) {
    let boxes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
        ])
        .split(area);

    let items: Vec<(&str, i64, Color)> = vec![
        ("Idea", app.dashboard.idea_count, MUTED),
        ("Drafting", app.dashboard.drafting_count, WARNING),
        (
            "Review",
            app.dashboard.review_count,
            Color::Rgb(168, 85, 247),
        ),
        ("Ready", app.dashboard.ready_count, INFO),
        (
            "Scheduled",
            app.dashboard.scheduled_count,
            Color::Rgb(20, 184, 166),
        ),
        ("Published", app.dashboard.published_count, SUCCESS),
    ];

    for (i, (label, count, color)) in items.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(*color))
            .border_type(ratatui::widgets::BorderType::Rounded);
        let inner = block.inner(boxes[i]);
        frame.render_widget(block, boxes[i]);

        let content = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);

        let count_text = Paragraph::new(Span::styled(
            format!("{count}"),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(count_text, content[0]);

        let label_text = Paragraph::new(Span::styled(*label, Style::default().fg(TEXT_DIM)))
            .alignment(Alignment::Center);
        frame.render_widget(label_text, content[1]);
    }
}

fn draw_recent_list(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Recent Content");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.dashboard.recent.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No content yet. Switch to Drafts tab and press 'n' to create one.",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
        ));
        frame.render_widget(empty, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Status",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Type",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Title",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Updated",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        )),
    ])
    .height(1)
    .bottom_margin(0);

    let rows: Vec<Row> = app
        .dashboard
        .recent
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let sc = status_color(&c.status);
            let style = if i == app.selected_index {
                Style::default().bg(SURFACE_HL)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(Span::styled(short_id(&c.id), Style::default().fg(TEXT_DIM))),
                Cell::from(Span::styled(
                    format!(" {} ", status_label(&c.status)),
                    Style::default().fg(Color::Black).bg(sc),
                )),
                Cell::from(Span::styled(
                    c.content_type.to_string(),
                    Style::default().fg(TEXT),
                )),
                Cell::from(Span::styled(
                    truncate(&c.title, 40),
                    Style::default().fg(TEXT),
                )),
                Cell::from(Span::styled(
                    c.updated_at.format("%m/%d %H:%M").to_string(),
                    Style::default().fg(TEXT_DIM),
                )),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(20),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().bg(SURFACE_HL).add_modifier(Modifier::BOLD));

    frame.render_widget(table, inner);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tab: Drafts
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw_drafts(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = if app.mode == Mode::Search {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(4)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(0), Constraint::Min(4)])
            .split(area)
    };

    // Search bar
    if app.mode == Mode::Search {
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(" \u{1f50d} Search ")
            .title_style(Style::default().fg(ACCENT));
        let search_text = Paragraph::new(Line::from(vec![
            Span::styled(&app.search_query, Style::default().fg(TEXT)),
            Span::styled("\u{2588}", Style::default().fg(ACCENT)), // cursor
        ]))
        .block(search_block);
        frame.render_widget(search_text, chunks[0]);
    }

    let content_area = chunks[1];
    let block = styled_block("Content Library");
    let inner = block.inner(content_area);
    frame.render_widget(block, content_area);

    if app.filtered_content.is_empty() {
        let msg = if app.search_query.is_empty() {
            "  No content found. Press 'n' to create a new draft."
        } else {
            "  No results match your search."
        };
        let empty = Paragraph::new(Span::styled(
            msg,
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
        ));
        frame.render_widget(empty, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Status",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Type",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Title",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Tags",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_content
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let sc = status_color(&c.status);
            let is_selected = i == app.selected_index;
            let row_style = if is_selected {
                Style::default().bg(SURFACE_HL)
            } else {
                Style::default()
            };
            let pointer = if is_selected { "\u{25b6} " } else { "  " };
            Row::new(vec![
                Cell::from(Span::styled(
                    format!("{}{}", pointer, short_id(&c.id)),
                    Style::default().fg(if is_selected { ACCENT } else { TEXT_DIM }),
                )),
                Cell::from(Span::styled(
                    format!(" {} ", status_label(&c.status)),
                    Style::default().fg(Color::Black).bg(sc),
                )),
                Cell::from(Span::styled(
                    c.content_type.to_string(),
                    Style::default().fg(TEXT),
                )),
                Cell::from(Span::styled(
                    truncate(&c.title, 45),
                    Style::default().fg(if is_selected { TEXT } else { TEXT_DIM }),
                )),
                Cell::from(Span::styled(
                    if c.tags.is_empty() {
                        "\u{2014}".to_string()
                    } else {
                        c.tags.join(", ")
                    },
                    Style::default().fg(TEXT_DIM),
                )),
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(20),
            Constraint::Length(20),
        ],
    )
    .header(header)
    .row_highlight_style(Style::default().bg(SURFACE_HL));

    frame.render_widget(table, inner);

    // Item count
    if inner.height > 1 {
        let count_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        let count_text = Paragraph::new(Span::styled(
            format!(
                " {} of {} items{}",
                app.selected_index + 1,
                app.filtered_content.len(),
                if !app.search_query.is_empty() {
                    format!("  (filtered: \"{}\")", app.search_query)
                } else {
                    String::new()
                }
            ),
            Style::default().fg(TEXT_DIM),
        ))
        .alignment(Alignment::Right);
        frame.render_widget(count_text, count_area);
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tab: Adapt
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw_adapt(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // selected content detail
            Constraint::Min(6),    // platform list
        ])
        .split(area);

    // Top: selected content details
    let detail_block = styled_block("Selected Content");
    let detail_inner = detail_block.inner(chunks[0]);
    frame.render_widget(detail_block, chunks[0]);

    if let Some(content) = &app.selected_content {
        let detail_lines = vec![
            Line::from(vec![
                Span::styled("  Title:  ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    &content.title,
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    format!(" {} ", status_label(&content.status)),
                    Style::default()
                        .fg(Color::Black)
                        .bg(status_color(&content.status)),
                ),
                Span::styled("    Type: ", Style::default().fg(TEXT_DIM)),
                Span::styled(content.content_type.to_string(), Style::default().fg(TEXT)),
            ]),
            Line::from(vec![
                Span::styled("  Body:   ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    truncate(&content.body, 80),
                    Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Chars:  ", Style::default().fg(TEXT_DIM)),
                Span::styled(format!("{}", content.body.len()), Style::default().fg(TEXT)),
                Span::styled("    Adaptations: ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    format!("{}", content.adaptations.len()),
                    Style::default().fg(ACCENT),
                ),
            ]),
        ];
        let detail_para = Paragraph::new(detail_lines);
        frame.render_widget(detail_para, detail_inner);
    } else {
        let empty = Paragraph::new(Span::styled(
            "  Select content from the list below (j/k to navigate, Enter to adapt)",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
        ));
        frame.render_widget(empty, detail_inner);
    }

    // Bottom: content list with adaptation status
    let list_block = styled_block("Content & Platforms");
    let list_inner = list_block.inner(chunks[1]);
    frame.render_widget(list_block, chunks[1]);

    if app.filtered_content.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No content available.",
            Style::default().fg(TEXT_DIM),
        ));
        frame.render_widget(empty, list_inner);
        return;
    }

    // Load adaptations for display
    let adapt_repo = AdaptationRepo::new(app.db.clone());
    let platforms = all_platforms();

    let header_cells: Vec<Cell> = std::iter::once(Cell::from(Span::styled(
        "Title",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )))
    .chain(platforms.iter().map(|p| {
        let name = p.to_string();
        let short: String = name.chars().take(3).collect();
        Cell::from(Span::styled(
            short,
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        ))
    }))
    .chain(std::iter::once(Cell::from(Span::styled(
        "Chars",
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    ))))
    .collect();

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_content
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let is_selected = i == app.selected_index;
            let adaptations = adapt_repo.list_for_content(c.id).unwrap_or_default();
            let adapted_platforms: Vec<Platform> = adaptations.iter().map(|a| a.platform).collect();

            let mut cells: Vec<Cell> = vec![Cell::from(Span::styled(
                format!(
                    "{}{}",
                    if is_selected { "\u{25b6} " } else { "  " },
                    truncate(&c.title, 25)
                ),
                Style::default().fg(if is_selected { ACCENT } else { TEXT }),
            ))];

            for p in &platforms {
                let has = adapted_platforms.contains(p);
                cells.push(Cell::from(Span::styled(
                    if has { " \u{2713} " } else { " \u{00b7} " },
                    Style::default().fg(if has { SUCCESS } else { Color::Rgb(55, 55, 70) }),
                )));
            }

            cells.push(Cell::from(Span::styled(
                format!("{}", c.body.len()),
                Style::default().fg(TEXT_DIM),
            )));

            let style = if is_selected {
                Style::default().bg(SURFACE_HL)
            } else {
                Style::default()
            };
            Row::new(cells).style(style)
        })
        .collect();

    let mut widths: Vec<Constraint> = vec![Constraint::Min(20)];
    for _ in &platforms {
        widths.push(Constraint::Length(5));
    }
    widths.push(Constraint::Length(7));

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, list_inner);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tab: Publish
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw_publish(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Ready to Publish");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.publish_items.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No content ready for publishing.",
                Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Content must be in 'ready' or 'scheduled' status with at least one adaptation.",
                Style::default().fg(TEXT_DIM),
            )),
        ]);
        frame.render_widget(empty, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "Title",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Status",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Platforms",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Adaptations",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .publish_items
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let is_selected = i == app.selected_index;
            let platform_names: Vec<String> = c
                .adaptations
                .iter()
                .map(|a| a.platform.to_string())
                .collect();
            let style = if is_selected {
                Style::default().bg(SURFACE_HL)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(Span::styled(
                    format!(
                        "{}{}",
                        if is_selected { "\u{25b6} " } else { "  " },
                        truncate(&c.title, 35)
                    ),
                    Style::default().fg(if is_selected { ACCENT } else { TEXT }),
                )),
                Cell::from(Span::styled(
                    format!(" {} ", status_label(&c.status)),
                    Style::default()
                        .fg(Color::Black)
                        .bg(status_color(&c.status)),
                )),
                Cell::from(Span::styled(
                    platform_names.join(", "),
                    Style::default().fg(TEXT),
                )),
                Cell::from(Span::styled(
                    format!("{}", c.adaptations.len()),
                    Style::default().fg(INFO),
                )),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(25),
            Constraint::Length(12),
            Constraint::Min(30),
            Constraint::Length(12),
        ],
    )
    .header(header);
    frame.render_widget(table, inner);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tab: Platforms
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw_platforms(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Configured Platforms");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.platform_accounts.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No platform accounts configured.",
                Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Use the CLI to add platform credentials:",
                Style::default().fg(TEXT_DIM),
            )),
            Line::from(Span::styled(
                "    contentforge platform add --name twitter --key <API_KEY>",
                Style::default().fg(ACCENT),
            )),
        ]);
        frame.render_widget(empty, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "Platform",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Display Name",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Status",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Credential",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Char Limit",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Difficulty",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .platform_accounts
        .iter()
        .enumerate()
        .map(|(i, acc)| {
            let is_selected = i == app.selected_index;
            let status_str = if acc.enabled { "Active" } else { "Disabled" };
            let status_clr = if acc.enabled { SUCCESS } else { DANGER };
            let limit = acc
                .platform
                .char_limit()
                .map(|l| format!("{l}"))
                .unwrap_or_else(|| "None".to_string());
            let difficulty = acc.platform.integration_difficulty();
            let diff_color = match difficulty {
                "easy" => SUCCESS,
                "medium" => WARNING,
                "hard" => DANGER,
                "fragile" => Color::Rgb(168, 85, 247),
                _ => TEXT_DIM,
            };
            let style = if is_selected {
                Style::default().bg(SURFACE_HL)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(Span::styled(
                    format!(
                        "{}{}",
                        if is_selected { "\u{25b6} " } else { "  " },
                        acc.platform
                    ),
                    Style::default().fg(if is_selected { ACCENT } else { TEXT }),
                )),
                Cell::from(Span::styled(&acc.display_name, Style::default().fg(TEXT))),
                Cell::from(Span::styled(
                    format!(" {status_str} "),
                    Style::default().fg(Color::Black).bg(status_clr),
                )),
                Cell::from(Span::styled(
                    credential_type_label(&acc.credential),
                    Style::default().fg(TEXT_DIM),
                )),
                Cell::from(Span::styled(limit, Style::default().fg(TEXT_DIM))),
                Cell::from(Span::styled(difficulty, Style::default().fg(diff_color))),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(18),
            Constraint::Min(18),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(header);
    frame.render_widget(table, inner);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Overlays / Popups
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn draw_detail_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);

    let content = match &app.selected_content {
        Some(c) => c,
        None => {
            let block = styled_block("Content Detail").border_style(Style::default().fg(ACCENT));
            let para = Paragraph::new("No content selected.").block(block);
            frame.render_widget(para, area);
            return;
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .title(format!(" {} ", truncate(&content.title, 50)))
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // metadata
            Constraint::Min(4),    // body
            Constraint::Length(3), // adaptations
        ])
        .split(inner);

    // Metadata
    let meta_lines = vec![
        Line::from(vec![
            Span::styled("ID:      ", Style::default().fg(TEXT_DIM)),
            Span::styled(content.id.to_string(), Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("Status:  ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!(" {} ", status_label(&content.status)),
                Style::default()
                    .fg(Color::Black)
                    .bg(status_color(&content.status)),
            ),
        ]),
        Line::from(vec![
            Span::styled("Type:    ", Style::default().fg(TEXT_DIM)),
            Span::styled(content.content_type.to_string(), Style::default().fg(TEXT)),
            Span::styled("    Tags: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                if content.tags.is_empty() {
                    "none".to_string()
                } else {
                    content.tags.join(", ")
                },
                Style::default().fg(INFO),
            ),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                content.created_at.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(TEXT),
            ),
            Span::styled("    Updated: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                content.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(TEXT),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(meta_lines), chunks[0]);

    // Body
    let body_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" Body ")
        .title_style(Style::default().fg(TEXT_DIM));
    let body_text = Paragraph::new(Span::styled(&content.body, Style::default().fg(TEXT)))
        .block(body_block)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));
    frame.render_widget(body_text, chunks[1]);

    // Adaptations
    let adapt_line = if content.adaptations.is_empty() {
        Line::from(Span::styled(
            "  No adaptations yet.",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
        ))
    } else {
        let mut spans = vec![Span::styled(
            "  Adapted for: ",
            Style::default().fg(TEXT_DIM),
        )];
        for (i, a) in content.adaptations.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(", ", Style::default().fg(BORDER)));
            }
            spans.push(Span::styled(
                a.platform.to_string(),
                Style::default().fg(SUCCESS),
            ));
            let char_count = a.body.len();
            if let Some(limit) = a.platform.char_limit() {
                let ratio_color = if char_count > limit { DANGER } else { TEXT_DIM };
                spans.push(Span::styled(
                    format!(" ({}/{})", char_count, limit),
                    Style::default().fg(ratio_color),
                ));
            }
        }
        Line::from(spans)
    };

    let adapt_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(BORDER));
    let adapt_para = Paragraph::new(vec![Line::from(""), adapt_line]).block(adapt_block);
    frame.render_widget(adapt_para, chunks[2]);
}

fn draw_help_overlay(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .title(" Keyboard Shortcuts ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  NAVIGATION",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    Tab / Shift-Tab  ", Style::default().fg(WARNING)),
            Span::styled("Switch tabs", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    1-5              ", Style::default().fg(WARNING)),
            Span::styled("Jump to tab", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    j/k or Up/Down   ", Style::default().fg(WARNING)),
            Span::styled("Move in lists", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    Enter            ", Style::default().fg(WARNING)),
            Span::styled("Select / action", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    Esc              ", Style::default().fg(WARNING)),
            Span::styled("Cancel / back", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  DRAFTS TAB",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    n                ", Style::default().fg(WARNING)),
            Span::styled("New draft", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    d                ", Style::default().fg(WARNING)),
            Span::styled("Delete selected", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    /                ", Style::default().fg(WARNING)),
            Span::styled("Search / filter", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  GENERAL",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    ?                ", Style::default().fg(WARNING)),
            Span::styled("Toggle this help", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("    q                ", Style::default().fg(WARNING)),
            Span::styled("Quit", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Esc or ? to close",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
        )),
    ];

    let para = Paragraph::new(help_text);
    frame.render_widget(para, inner);
}

fn draw_confirm_delete(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 25, frame.area());
    frame.render_widget(Clear, area);

    let title_text = if !app.filtered_content.is_empty() {
        let idx = app
            .selected_index
            .min(app.filtered_content.len().saturating_sub(1));
        format!("\"{}\"", truncate(&app.filtered_content[idx].title, 30))
    } else {
        "this item".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(DANGER))
        .title(" Confirm Delete ")
        .title_style(Style::default().fg(DANGER).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Delete {title_text}?"),
            Style::default().fg(TEXT),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                "y",
                Style::default().fg(DANGER).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to confirm, any other key to cancel",
                Style::default().fg(TEXT_DIM),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(text), inner);
}

fn draw_platform_selector(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .title(" Select Platform ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let platforms = all_platforms();

    // Check which platforms already have adaptations
    let adapted: Vec<Platform> = app
        .selected_content
        .as_ref()
        .map(|c| c.adaptations.iter().map(|a| a.platform).collect())
        .unwrap_or_default();

    let body_len = app
        .selected_content
        .as_ref()
        .map(|c| c.body.len())
        .unwrap_or(0);

    let items: Vec<ListItem> = platforms
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == app.adapt_platform_idx;
            let has_adaptation = adapted.contains(p);
            let check = if has_adaptation { "\u{2713}" } else { " " };
            let pointer = if is_selected { "\u{25b6}" } else { " " };

            let limit_info = match p.char_limit() {
                Some(limit) => {
                    let color = if body_len > limit { DANGER } else { SUCCESS };
                    Span::styled(
                        format!("  {}/{}", body_len, limit),
                        Style::default().fg(color),
                    )
                }
                None => Span::styled("  no limit", Style::default().fg(TEXT_DIM)),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {pointer} "),
                    Style::default().fg(if is_selected { ACCENT } else { TEXT_DIM }),
                ),
                Span::styled(
                    format!("[{check}] "),
                    Style::default().fg(if has_adaptation { SUCCESS } else { TEXT_DIM }),
                ),
                Span::styled(
                    format!("{:<14}", p.to_string()),
                    Style::default().fg(if is_selected { TEXT } else { TEXT_DIM }),
                ),
                limit_info,
            ]);

            let style = if is_selected {
                Style::default().bg(SURFACE_HL)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn draw_publish_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(55, 45, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(SUCCESS))
        .title(" Publish ")
        .title_style(Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.publish_items.is_empty() {
        frame.render_widget(Paragraph::new("No items to publish."), inner);
        return;
    }

    let idx = app
        .selected_index
        .min(app.publish_items.len().saturating_sub(1));
    let content = &app.publish_items[idx];

    let mut lines = vec![
        Line::from(vec![
            Span::styled("  Content: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                &content.title,
                Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Select platform to publish to:",
            Style::default().fg(TEXT_DIM),
        )),
        Line::from(""),
    ];

    for (i, a) in content.adaptations.iter().enumerate() {
        let is_selected = i == app.publish_platform_idx;
        let pointer = if is_selected { "\u{25b6}" } else { " " };
        lines.push(Line::from(vec![
            Span::styled(
                format!("    {pointer} "),
                Style::default().fg(if is_selected { ACCENT } else { TEXT_DIM }),
            ),
            Span::styled(
                a.platform.to_string(),
                Style::default().fg(if is_selected { ACCENT } else { TEXT }),
            ),
            Span::styled(
                format!("  ({} chars)", a.body.len()),
                Style::default().fg(TEXT_DIM),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Press ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            "Enter",
            Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to publish, ", Style::default().fg(TEXT_DIM)),
        Span::styled("Esc", Style::default().fg(WARNING)),
        Span::styled(" to cancel", Style::default().fg(TEXT_DIM)),
    ]));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_new_draft_overlay(frame: &mut Frame, app: &App) {
    let area = centered_rect(65, 55, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .title(" New Draft ")
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(3), // type
            Constraint::Min(4),    // body
            Constraint::Length(2), // hints
        ])
        .split(inner);

    // Title field
    let title_border_color = if app.input_field == 0 { ACCENT } else { BORDER };
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_border_color))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" Title ")
        .title_style(Style::default().fg(if app.input_field == 0 {
            ACCENT
        } else {
            TEXT_DIM
        }));

    let cursor = if app.input_field == 0 { "\u{2588}" } else { "" };
    let title_para = Paragraph::new(Line::from(vec![
        Span::styled(&app.input_title, Style::default().fg(TEXT)),
        Span::styled(cursor, Style::default().fg(ACCENT)),
    ]))
    .block(title_block);
    frame.render_widget(title_para, chunks[0]);

    // Type selector
    let type_border_color = if app.input_field == 2 { ACCENT } else { BORDER };
    let type_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(type_border_color))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" Type ")
        .title_style(Style::default().fg(if app.input_field == 2 {
            ACCENT
        } else {
            TEXT_DIM
        }));

    let ct = content_type_options()[app.content_type_idx];
    let type_para = Paragraph::new(Line::from(vec![
        Span::styled(
            " \u{25c0} ",
            Style::default().fg(if app.input_field == 2 {
                ACCENT
            } else {
                TEXT_DIM
            }),
        ),
        Span::styled(
            ct.to_string(),
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " \u{25b6} ",
            Style::default().fg(if app.input_field == 2 {
                ACCENT
            } else {
                TEXT_DIM
            }),
        ),
    ]))
    .block(type_block);
    frame.render_widget(type_para, chunks[1]);

    // Body field
    let body_border_color = if app.input_field == 1 { ACCENT } else { BORDER };
    let body_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(body_border_color))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" Body ")
        .title_style(Style::default().fg(if app.input_field == 1 {
            ACCENT
        } else {
            TEXT_DIM
        }));

    let body_cursor = if app.input_field == 1 { "\u{2588}" } else { "" };
    let body_para = Paragraph::new(Line::from(vec![
        Span::styled(&app.input_body, Style::default().fg(TEXT)),
        Span::styled(body_cursor, Style::default().fg(ACCENT)),
    ]))
    .block(body_block)
    .wrap(Wrap { trim: false });
    frame.render_widget(body_para, chunks[2]);

    // Hints
    let hints = Paragraph::new(Line::from(vec![
        Span::styled("  Tab", Style::default().fg(WARNING)),
        Span::styled(" switch field  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Enter", Style::default().fg(WARNING)),
        Span::styled(
            " submit (from title) / newline (in body)  ",
            Style::default().fg(TEXT_DIM),
        ),
        Span::styled("Esc", Style::default().fg(WARNING)),
        Span::styled(" cancel", Style::default().fg(TEXT_DIM)),
    ]));
    frame.render_widget(hints, chunks[3]);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Main entry point
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Run the TUI application. This blocks until the user quits.
pub fn run(db: DbPool) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db);

    // Main loop
    loop {
        // Refresh data if needed (on tab switch or after mutations)
        app.refresh_data();

        // Refresh selected content on Adapt tab when navigating
        if app.active_tab == Tab::Adapt && app.mode == Mode::Normal {
            app.load_selected_content_full();
        }

        terminal.draw(|frame| draw(frame, &mut app))?;

        if event::poll(std::time::Duration::from_millis(TICK_RATE_MS))? {
            let ev = event::read()?;
            app.handle_event(ev);
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
