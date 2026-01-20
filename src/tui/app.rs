//! TUI Application state and logic

use crate::core::{Project, ScanConfig, ScanResult, Scanner};
use crate::plugins::PluginRegistry;
use crate::scanner::ParallelScanner;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

/// Message sent from scan thread
pub enum ScanMessage {
    /// Progress update
    Progress { dirs_scanned: usize, message: String },
    /// Scan completed for projects
    CompleteProjects(ScanResult),
    /// Scan completed for caches
    CompleteCaches(Vec<CacheEntry>),
    /// Scan completed for cleaners
    CompleteCleaners(Vec<CleanerEntry>),
    /// Scan error
    Error(String),
}

/// Scan mode - what type of scan to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    /// Scan everything at once
    All,
    /// Scan for development projects (node_modules, target, venv, etc.)
    Projects,
    /// Scan global caches (npm, pip, cargo, brew, etc.)
    Caches,
    /// Scan Xcode artifacts
    Xcode,
    /// Scan Docker resources
    Docker,
    /// Scan IDE caches
    IDECaches,
    /// Scan ML/AI caches
    MLCaches,
}

impl ScanMode {
    /// Get all scan modes
    pub fn all_modes() -> Vec<ScanMode> {
        vec![
            ScanMode::All,
            ScanMode::Projects,
            ScanMode::Caches,
            ScanMode::Xcode,
            ScanMode::Docker,
            ScanMode::IDECaches,
            ScanMode::MLCaches,
        ]
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            ScanMode::All => "Scan All",
            ScanMode::Projects => "Projects",
            ScanMode::Caches => "Global Caches",
            ScanMode::Xcode => "Xcode",
            ScanMode::Docker => "Docker",
            ScanMode::IDECaches => "IDE Caches",
            ScanMode::MLCaches => "ML/AI Models",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            ScanMode::All => "Everything - projects, caches, Docker, Xcode, IDEs, ML",
            ScanMode::Projects => "node_modules, target, venv, .gradle, vendor",
            ScanMode::Caches => "npm, pip, cargo, brew, CocoaPods cache",
            ScanMode::Xcode => "DerivedData, Archives, Simulators",
            ScanMode::Docker => "Images, Containers, Volumes, Build Cache",
            ScanMode::IDECaches => "JetBrains, VS Code, Cursor caches",
            ScanMode::MLCaches => "Huggingface, Ollama, PyTorch models",
        }
    }

    /// Get icon
    pub fn icon(&self) -> &'static str {
        match self {
            ScanMode::All => "🔥",
            ScanMode::Projects => "📦",
            ScanMode::Caches => "🗄️",
            ScanMode::Xcode => "🍎",
            ScanMode::Docker => "🐳",
            ScanMode::IDECaches => "💻",
            ScanMode::MLCaches => "🤖",
        }
    }
}

/// Cache entry for display
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub icon: String,
    pub description: String,
    pub selected: bool,
    pub visible: bool,
}

/// Cleaner entry for display
#[derive(Debug, Clone)]
pub struct CleanerEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub icon: String,
    pub category: String,
    pub selected: bool,
    pub visible: bool,
}

/// Main TUI application state
pub struct App {
    /// Current screen/state
    pub state: AppState,
    /// Selected scan mode
    pub scan_mode: ScanMode,
    /// Menu selection index (for ready screen)
    pub menu_index: usize,
    /// Projects to display
    pub projects: Vec<ProjectEntry>,
    /// Caches to display
    pub caches: Vec<CacheEntry>,
    /// Cleaner items to display
    pub cleaners: Vec<CleanerEntry>,
    /// Currently selected index
    pub selected: usize,
    /// Scroll offset for the list
    pub scroll_offset: usize,
    /// Expanded project indices
    pub expanded: HashSet<usize>,
    /// Status message
    pub status_message: Option<String>,
    /// Should quit
    pub should_quit: bool,
    /// Show help popup
    pub show_help: bool,
    /// Current tab/category
    pub current_tab: usize,
    /// Available tabs
    pub tabs: Vec<String>,
    /// Search query
    pub search_query: String,
    /// Is searching
    pub is_searching: bool,
    /// Scan paths
    pub scan_paths: Vec<PathBuf>,
    /// Scan progress (0.0 - 1.0)
    pub scan_progress: f64,
    /// Scan message
    pub scan_message: String,
    /// Directories scanned
    pub dirs_scanned: usize,
    /// Total size found
    pub total_size: u64,
    /// Scan result receiver (for async scanning)
    scan_receiver: Option<Receiver<ScanMessage>>,
    /// Animation frame counter
    pub anim_frame: usize,
}

/// Application state/screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Initial state, showing menu
    Ready,
    /// Currently scanning
    Scanning,
    /// Showing project results
    Results,
    /// Showing cache results
    CacheResults,
    /// Showing cleaner results
    CleanerResults,
    /// Confirming deletion
    Confirming,
    /// Cleaning in progress
    Cleaning,
    /// Error state
    Error(String),
}

/// A project entry in the list
#[derive(Debug, Clone)]
pub struct ProjectEntry {
    /// The project data
    pub project: Project,
    /// Is this entry selected for deletion
    pub selected: bool,
    /// Is this entry visible (after filtering)
    pub visible: bool,
}

impl App {
    /// Create a new app with scan paths
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            state: AppState::Ready,
            scan_mode: ScanMode::All,
            menu_index: 0,
            projects: Vec::new(),
            caches: Vec::new(),
            cleaners: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            expanded: HashSet::new(),
            status_message: Some("Select a scan mode and press Enter".to_string()),
            should_quit: false,
            show_help: false,
            current_tab: 0,
            tabs: vec![
                "All".to_string(),
                "Node".to_string(),
                "Rust".to_string(),
                "Python".to_string(),
                "Java".to_string(),
                "Other".to_string(),
            ],
            search_query: String::new(),
            is_searching: false,
            scan_paths: paths,
            scan_progress: 0.0,
            scan_message: String::new(),
            dirs_scanned: 0,
            total_size: 0,
            scan_receiver: None,
            anim_frame: 0,
        }
    }

    /// Move menu selection up
    pub fn menu_up(&mut self) {
        let modes = ScanMode::all_modes();
        if self.menu_index > 0 {
            self.menu_index -= 1;
        } else {
            self.menu_index = modes.len() - 1;
        }
        self.scan_mode = modes[self.menu_index];
    }

    /// Move menu selection down
    pub fn menu_down(&mut self) {
        let modes = ScanMode::all_modes();
        if self.menu_index < modes.len() - 1 {
            self.menu_index += 1;
        } else {
            self.menu_index = 0;
        }
        self.scan_mode = modes[self.menu_index];
    }

    /// Start scanning based on current mode
    pub fn start_scan(&mut self) {
        self.state = AppState::Scanning;
        self.projects.clear();
        self.caches.clear();
        self.cleaners.clear();
        self.scan_progress = 0.0;
        self.scan_message = format!("Initializing {} scan...", self.scan_mode.name());
        self.dirs_scanned = 0;
        self.anim_frame = 0;
        self.selected = 0;
        self.scroll_offset = 0;

        // Create channel for communication
        let (tx, rx): (Sender<ScanMessage>, Receiver<ScanMessage>) = mpsc::channel();
        self.scan_receiver = Some(rx);

        // Clone paths for the thread
        let paths = if self.scan_paths.is_empty() {
            vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
        } else {
            self.scan_paths.clone()
        };

        let mode = self.scan_mode;

        // Spawn scanning thread
        thread::spawn(move || {
            match mode {
                ScanMode::All => {
                    Self::scan_all(tx, paths);
                }
                ScanMode::Projects => {
                    Self::scan_projects(tx, paths);
                }
                ScanMode::Caches => {
                    Self::scan_caches(tx);
                }
                ScanMode::Xcode => {
                    Self::scan_xcode(tx);
                }
                ScanMode::Docker => {
                    Self::scan_docker(tx);
                }
                ScanMode::IDECaches => {
                    Self::scan_ide_caches(tx);
                }
                ScanMode::MLCaches => {
                    Self::scan_ml_caches(tx);
                }
            }
        });
    }

    /// Scan everything at once
    fn scan_all(tx: Sender<ScanMessage>, paths: Vec<PathBuf>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning everything...".to_string(),
        });

        // Collect all cleaners
        let mut all_cleaners: Vec<CleanerEntry> = Vec::new();

        // 1. Scan caches
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning global caches...".to_string(),
        });
        if let Ok(mut caches) = crate::caches::detect_caches() {
            let _ = crate::caches::calculate_all_sizes(&mut caches);
            for c in caches.into_iter().filter(|c| c.size > 0) {
                all_cleaners.push(CleanerEntry {
                    name: c.name.clone(),
                    path: c.path.clone(),
                    size: c.size,
                    icon: c.icon.to_string(),
                    category: "Cache".to_string(),
                    selected: false,
                    visible: true,
                });
            }
        }

        // 2. Scan Xcode
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning Xcode...".to_string(),
        });
        if let Some(cleaner) = crate::cleaners::xcode::XcodeCleaner::new() {
            if let Ok(items) = cleaner.detect() {
                for item in items {
                    all_cleaners.push(CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    });
                }
            }
        }

        // 3. Scan Docker
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning Docker...".to_string(),
        });
        let docker = crate::cleaners::docker::DockerCleaner::new();
        if docker.is_available() {
            if let Ok(items) = docker.detect() {
                for item in items {
                    all_cleaners.push(CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    });
                }
            }
        }

        // 4. Scan IDE caches
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning IDE caches...".to_string(),
        });
        if let Some(cleaner) = crate::cleaners::ide::IdeCleaner::new() {
            if let Ok(items) = cleaner.detect() {
                for item in items {
                    all_cleaners.push(CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    });
                }
            }
        }

        // 5. Scan ML caches
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning ML/AI models...".to_string(),
        });
        if let Some(cleaner) = crate::cleaners::ml::MlCleaner::new() {
            if let Ok(items) = cleaner.detect() {
                for item in items {
                    all_cleaners.push(CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    });
                }
            }
        }

        // 6. Scan projects
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning development projects...".to_string(),
        });
        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);
        let mut config = ScanConfig::default();
        config.roots = paths;

        if let Ok(result) = scanner.scan(&config) {
            for p in result.projects {
                for artifact in &p.artifacts {
                    all_cleaners.push(CleanerEntry {
                        name: format!("{} ({})", p.name, artifact.name()),
                        path: artifact.path.clone(),
                        size: artifact.size,
                        icon: p.kind.icon().to_string(),
                        category: format!("{:?}", p.kind),
                        selected: false,
                        visible: true,
                    });
                }
            }
        }

        let _ = tx.send(ScanMessage::CompleteCleaners(all_cleaners));
    }

    /// Scan for projects
    fn scan_projects(tx: Sender<ScanMessage>, paths: Vec<PathBuf>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning for development projects...".to_string(),
        });

        let registry = Arc::new(PluginRegistry::with_builtins());
        let scanner = ParallelScanner::new(registry);

        let mut config = ScanConfig::default();
        config.roots = paths;

        match scanner.scan(&config) {
            Ok(result) => {
                let _ = tx.send(ScanMessage::CompleteProjects(result));
            }
            Err(e) => {
                let _ = tx.send(ScanMessage::Error(e.to_string()));
            }
        }
    }

    /// Scan for global caches
    fn scan_caches(tx: Sender<ScanMessage>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Detecting global caches...".to_string(),
        });

        match crate::caches::detect_caches() {
            Ok(mut caches) => {
                let _ = crate::caches::calculate_all_sizes(&mut caches);

                let entries: Vec<CacheEntry> = caches
                    .into_iter()
                    .filter(|c| c.size > 0)
                    .map(|c| CacheEntry {
                        name: c.name.clone(),
                        path: c.path.clone(),
                        size: c.size,
                        icon: c.icon.to_string(),
                        description: c.description.to_string(),
                        selected: false,
                        visible: true,
                    })
                    .collect();

                let _ = tx.send(ScanMessage::CompleteCaches(entries));
            }
            Err(e) => {
                let _ = tx.send(ScanMessage::Error(e.to_string()));
            }
        }
    }

    /// Scan Xcode artifacts
    fn scan_xcode(tx: Sender<ScanMessage>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning Xcode artifacts...".to_string(),
        });

        if let Some(cleaner) = crate::cleaners::xcode::XcodeCleaner::new() {
            if let Ok(items) = cleaner.detect() {
                let entries: Vec<CleanerEntry> = items
                    .into_iter()
                    .map(|item| CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    })
                    .collect();
                let _ = tx.send(ScanMessage::CompleteCleaners(entries));
                return;
            }
        }
        let _ = tx.send(ScanMessage::CompleteCleaners(vec![]));
    }

    /// Scan Docker resources
    fn scan_docker(tx: Sender<ScanMessage>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning Docker resources...".to_string(),
        });

        let cleaner = crate::cleaners::docker::DockerCleaner::new();
        if cleaner.is_available() {
            if let Ok(items) = cleaner.detect() {
                let entries: Vec<CleanerEntry> = items
                    .into_iter()
                    .map(|item| CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    })
                    .collect();
                let _ = tx.send(ScanMessage::CompleteCleaners(entries));
                return;
            }
        }
        let _ = tx.send(ScanMessage::CompleteCleaners(vec![]));
    }

    /// Scan IDE caches
    fn scan_ide_caches(tx: Sender<ScanMessage>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning IDE caches...".to_string(),
        });

        if let Some(cleaner) = crate::cleaners::ide::IdeCleaner::new() {
            if let Ok(items) = cleaner.detect() {
                let entries: Vec<CleanerEntry> = items
                    .into_iter()
                    .map(|item| CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    })
                    .collect();
                let _ = tx.send(ScanMessage::CompleteCleaners(entries));
                return;
            }
        }
        let _ = tx.send(ScanMessage::CompleteCleaners(vec![]));
    }

    /// Scan ML caches
    fn scan_ml_caches(tx: Sender<ScanMessage>) {
        let _ = tx.send(ScanMessage::Progress {
            dirs_scanned: 0,
            message: "Scanning ML/AI caches...".to_string(),
        });

        if let Some(cleaner) = crate::cleaners::ml::MlCleaner::new() {
            if let Ok(items) = cleaner.detect() {
                let entries: Vec<CleanerEntry> = items
                    .into_iter()
                    .map(|item| CleanerEntry {
                        name: item.name,
                        path: item.path,
                        size: item.size,
                        icon: item.icon.to_string(),
                        category: item.category,
                        selected: false,
                        visible: true,
                    })
                    .collect();
                let _ = tx.send(ScanMessage::CompleteCleaners(entries));
                return;
            }
        }
        let _ = tx.send(ScanMessage::CompleteCleaners(vec![]));
    }

    /// Check for scan updates (call this on tick)
    pub fn check_scan_progress(&mut self) {
        // Increment animation frame
        self.anim_frame = self.anim_frame.wrapping_add(1);

        if let Some(ref rx) = self.scan_receiver {
            // Try to receive without blocking
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    ScanMessage::Progress { dirs_scanned, message } => {
                        self.dirs_scanned = dirs_scanned;
                        self.scan_message = message;
                    }
                    ScanMessage::CompleteProjects(result) => {
                        self.handle_project_scan_complete(result);
                        self.scan_receiver = None;
                        return;
                    }
                    ScanMessage::CompleteCaches(caches) => {
                        self.handle_cache_scan_complete(caches);
                        self.scan_receiver = None;
                        return;
                    }
                    ScanMessage::CompleteCleaners(cleaners) => {
                        self.handle_cleaner_scan_complete(cleaners);
                        self.scan_receiver = None;
                        return;
                    }
                    ScanMessage::Error(err) => {
                        self.state = AppState::Error(err.clone());
                        self.status_message = Some(format!("Error: {}", err));
                        self.scan_receiver = None;
                        return;
                    }
                }
            }
        }
    }

    /// Handle project scan completion
    fn handle_project_scan_complete(&mut self, scan_result: ScanResult) {
        self.projects = scan_result
            .projects
            .iter()
            .map(|p: &Project| ProjectEntry {
                project: p.clone(),
                selected: false,
                visible: true,
            })
            .collect();

        // Sort by size (largest first)
        self.projects
            .sort_by(|a, b| b.project.cleanable_size.cmp(&a.project.cleanable_size));

        self.total_size = scan_result.total_cleanable;
        self.dirs_scanned = scan_result.directories_scanned;
        self.state = AppState::Results;
        self.status_message = Some(format!(
            "Found {} projects ({}) - Use j/k to navigate, Space to select",
            self.projects.len(),
            format_size(self.total_size)
        ));
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Handle cache scan completion
    fn handle_cache_scan_complete(&mut self, caches: Vec<CacheEntry>) {
        self.caches = caches;
        self.caches.sort_by(|a, b| b.size.cmp(&a.size));

        self.total_size = self.caches.iter().map(|c| c.size).sum();
        self.state = AppState::CacheResults;
        self.status_message = Some(format!(
            "Found {} caches ({}) - Use j/k to navigate, Space to select",
            self.caches.len(),
            format_size(self.total_size)
        ));
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Handle cleaner scan completion
    fn handle_cleaner_scan_complete(&mut self, cleaners: Vec<CleanerEntry>) {
        self.cleaners = cleaners;
        self.cleaners.sort_by(|a, b| b.size.cmp(&a.size));

        self.total_size = self.cleaners.iter().map(|c| c.size).sum();
        self.state = AppState::CleanerResults;
        self.status_message = Some(format!(
            "Found {} items ({}) - Use j/k to navigate, Space to select",
            self.cleaners.len(),
            format_size(self.total_size)
        ));
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        let count = self.item_count();
        if count == 0 {
            return;
        }

        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = count - 1;
        }
        self.ensure_visible();
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        let count = self.item_count();
        if count == 0 {
            return;
        }

        if self.selected < count - 1 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Get total item count based on current state
    fn item_count(&self) -> usize {
        match self.state {
            AppState::Results => self.visible_count(),
            AppState::CacheResults => self.caches.iter().filter(|c| c.visible).count(),
            AppState::CleanerResults => self.cleaners.iter().filter(|c| c.visible).count(),
            _ => 0,
        }
    }

    /// Page up
    pub fn page_up(&mut self, page_size: usize) {
        if self.selected >= page_size {
            self.selected -= page_size;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Page down
    pub fn page_down(&mut self, page_size: usize) {
        let count = self.item_count();
        if count == 0 {
            return;
        }

        if self.selected + page_size < count {
            self.selected += page_size;
        } else {
            self.selected = count - 1;
        }
        self.ensure_visible();
    }

    /// Go to top
    pub fn go_top(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Go to bottom
    pub fn go_bottom(&mut self) {
        let count = self.item_count();
        if count > 0 {
            self.selected = count - 1;
        }
        self.ensure_visible();
    }

    /// Ensure selected item is visible
    fn ensure_visible(&mut self) {
        // Will be handled by ensure_visible_with_height
    }

    /// Ensure selected item is visible with viewport height
    pub fn ensure_visible_with_height(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.selected - viewport_height + 1;
        }
    }

    /// Toggle selection of current item
    pub fn toggle_select(&mut self) {
        match self.state {
            AppState::Results => self.toggle_select_project(),
            AppState::CacheResults => self.toggle_select_cache(),
            AppState::CleanerResults => self.toggle_select_cleaner(),
            _ => {}
        }
        self.update_status();
    }

    fn toggle_select_project(&mut self) {
        let selected_idx = self.selected;
        let visible_indices: Vec<usize> = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = visible_indices.get(selected_idx) {
            if let Some(entry) = self.projects.get_mut(idx) {
                entry.selected = !entry.selected;
            }
        }
    }

    fn toggle_select_cache(&mut self) {
        if let Some(cache) = self.caches.get_mut(self.selected) {
            cache.selected = !cache.selected;
        }
    }

    fn toggle_select_cleaner(&mut self) {
        if let Some(cleaner) = self.cleaners.get_mut(self.selected) {
            cleaner.selected = !cleaner.selected;
        }
    }

    /// Toggle expanded state of current item
    pub fn toggle_expand(&mut self) {
        let visible_indices: Vec<usize> = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = visible_indices.get(self.selected) {
            if self.expanded.contains(&idx) {
                self.expanded.remove(&idx);
            } else {
                self.expanded.insert(idx);
            }
        }
    }

    /// Expand current item (right arrow)
    pub fn expand(&mut self) {
        let visible_indices: Vec<usize> = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = visible_indices.get(self.selected) {
            self.expanded.insert(idx);
        }
    }

    /// Collapse current item (left arrow)
    pub fn collapse(&mut self) {
        let visible_indices: Vec<usize> = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = visible_indices.get(self.selected) {
            self.expanded.remove(&idx);
        }
    }

    /// Scroll up by one item
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            if self.selected > self.scroll_offset + 20 {
                self.selected = self.scroll_offset + 20;
            }
        }
    }

    /// Scroll down by one item
    pub fn scroll_down(&mut self) {
        let count = self.item_count();
        if count > 0 && self.scroll_offset < count.saturating_sub(1) {
            self.scroll_offset += 1;
            if self.selected < self.scroll_offset {
                self.selected = self.scroll_offset;
            }
        }
    }

    /// Select all visible items
    pub fn select_all(&mut self) {
        match self.state {
            AppState::Results => {
                for entry in &mut self.projects {
                    if entry.visible {
                        entry.selected = true;
                    }
                }
            }
            AppState::CacheResults => {
                for cache in &mut self.caches {
                    if cache.visible {
                        cache.selected = true;
                    }
                }
            }
            AppState::CleanerResults => {
                for cleaner in &mut self.cleaners {
                    if cleaner.visible {
                        cleaner.selected = true;
                    }
                }
            }
            _ => {}
        }
        self.update_status();
    }

    /// Deselect all items
    pub fn deselect_all(&mut self) {
        for entry in &mut self.projects {
            entry.selected = false;
        }
        for cache in &mut self.caches {
            cache.selected = false;
        }
        for cleaner in &mut self.cleaners {
            cleaner.selected = false;
        }
        self.update_status();
    }

    /// Get visible projects
    pub fn visible_projects(&self) -> Vec<&ProjectEntry> {
        self.projects.iter().filter(|p| p.visible).collect()
    }

    /// Count visible projects
    pub fn visible_count(&self) -> usize {
        self.projects.iter().filter(|p| p.visible).count()
    }

    /// Get selected projects
    pub fn selected_projects(&self) -> Vec<&ProjectEntry> {
        self.projects.iter().filter(|p| p.selected).collect()
    }

    /// Get total selected size
    pub fn selected_size(&self) -> u64 {
        let project_size: u64 = self.projects
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.project.cleanable_size)
            .sum();
        let cache_size: u64 = self.caches
            .iter()
            .filter(|c| c.selected)
            .map(|c| c.size)
            .sum();
        let cleaner_size: u64 = self.cleaners
            .iter()
            .filter(|c| c.selected)
            .map(|c| c.size)
            .sum();
        project_size + cache_size + cleaner_size
    }

    /// Get number of selected items
    pub fn selected_count(&self) -> usize {
        let projects = self.projects.iter().filter(|p| p.selected).count();
        let caches = self.caches.iter().filter(|c| c.selected).count();
        let cleaners = self.cleaners.iter().filter(|c| c.selected).count();
        projects + caches + cleaners
    }

    /// Update status message
    fn update_status(&mut self) {
        let selected = self.selected_count();
        let size = self.selected_size();
        if selected > 0 {
            self.status_message = Some(format!(
                "Selected: {} items ({}) - Press 'd' to delete",
                selected,
                format_size(size)
            ));
        } else {
            let count = self.item_count();
            self.status_message = Some(format!(
                "Found {} items ({}) - Use j/k to navigate, Space to select",
                count,
                format_size(self.total_size)
            ));
        }
    }

    /// Filter projects by current tab
    pub fn filter_by_tab(&mut self) {
        let tab = &self.tabs[self.current_tab];

        for entry in &mut self.projects {
            entry.visible = match tab.as_str() {
                "All" => true,
                "Node" => entry.project.kind.is_node(),
                "Rust" => entry.project.kind.is_rust(),
                "Python" => entry.project.kind.is_python(),
                "Java" => entry.project.kind.is_java(),
                "Other" => {
                    !entry.project.kind.is_node()
                        && !entry.project.kind.is_rust()
                        && !entry.project.kind.is_python()
                        && !entry.project.kind.is_java()
                }
                _ => true,
            };

            // Also apply search filter
            if entry.visible && !self.search_query.is_empty() {
                let query = self.search_query.to_lowercase();
                entry.visible = entry.project.name.to_lowercase().contains(&query)
                    || entry
                        .project
                        .root
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query);
            }
        }

        // Reset selection if current selection is not visible
        if self.selected >= self.visible_count() {
            self.selected = 0;
        }
        self.scroll_offset = 0;
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
        self.filter_by_tab();
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tabs.len() - 1;
        }
        self.filter_by_tab();
    }

    /// Toggle help
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Enter search mode
    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.search_query.clear();
    }

    /// Exit search mode
    pub fn end_search(&mut self) {
        self.is_searching = false;
    }

    /// Add character to search query
    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.filter_by_tab();
    }

    /// Remove last character from search query
    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.filter_by_tab();
    }

    /// Get current project (selected)
    pub fn current_project(&self) -> Option<&ProjectEntry> {
        self.visible_projects().get(self.selected).copied()
    }

    /// Is project expanded
    pub fn is_expanded(&self, visible_index: usize) -> bool {
        let visible_indices: Vec<usize> = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.visible)
            .map(|(i, _)| i)
            .collect();

        visible_indices
            .get(visible_index)
            .map(|&idx| self.expanded.contains(&idx))
            .unwrap_or(false)
    }

    /// Request deletion confirmation
    pub fn request_delete(&mut self) {
        if self.selected_count() > 0 {
            self.state = AppState::Confirming;
        } else {
            self.status_message = Some("No items selected. Use Space to select items.".to_string());
        }
    }

    /// Cancel deletion
    pub fn cancel_delete(&mut self) {
        // Return to appropriate results state
        if !self.projects.is_empty() {
            self.state = AppState::Results;
        } else if !self.caches.is_empty() {
            self.state = AppState::CacheResults;
        } else if !self.cleaners.is_empty() {
            self.state = AppState::CleanerResults;
        } else {
            self.state = AppState::Ready;
        }
        self.update_status();
    }

    /// Confirm and perform deletion
    pub fn confirm_delete(&mut self) -> Vec<PathBuf> {
        self.state = AppState::Cleaning;

        // Collect paths to delete
        let mut paths: Vec<PathBuf> = self
            .projects
            .iter()
            .filter(|p| p.selected)
            .flat_map(|p| p.project.artifacts.iter().map(|a| a.path.clone()))
            .collect();

        // Add cache paths
        for cache in &self.caches {
            if cache.selected {
                paths.push(cache.path.clone());
            }
        }

        // Add cleaner paths
        for cleaner in &self.cleaners {
            if cleaner.selected {
                paths.push(cleaner.path.clone());
            }
        }

        paths
    }

    /// Mark deletion complete
    pub fn deletion_complete(&mut self, success_count: usize, fail_count: usize, freed: u64) {
        // Remove deleted items
        self.projects.retain(|p| !p.selected);
        self.caches.retain(|c| !c.selected);
        self.cleaners.retain(|c| !c.selected);

        // Return to appropriate state
        if !self.projects.is_empty() {
            self.state = AppState::Results;
        } else if !self.caches.is_empty() {
            self.state = AppState::CacheResults;
        } else if !self.cleaners.is_empty() {
            self.state = AppState::CleanerResults;
        } else {
            self.state = AppState::Ready;
        }

        self.status_message = Some(format!(
            "Deleted {} items, freed {} ({} failed)",
            success_count,
            format_size(freed),
            fail_count
        ));

        // Recalculate total
        self.total_size = self.projects.iter().map(|p| p.project.cleanable_size).sum::<u64>()
            + self.caches.iter().map(|c| c.size).sum::<u64>()
            + self.cleaners.iter().map(|c| c.size).sum::<u64>();

        // Reset selection
        let count = self.item_count();
        if self.selected >= count && count > 0 {
            self.selected = count - 1;
        }
    }

    /// Go back to menu
    pub fn go_back(&mut self) {
        self.state = AppState::Ready;
        self.projects.clear();
        self.caches.clear();
        self.cleaners.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.total_size = 0;
        self.status_message = Some("Select a scan mode and press Enter".to_string());
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(vec![])
    }
}

/// Format size in human-readable form
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
