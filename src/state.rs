use std::collections::{HashMap, HashSet};

use crate::command::Mode;
use zellij_tile::prelude::{PaneInfo, PaneManifest, TabInfo};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaneRef {
    Terminal(u32),
    Plugin(u32),
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub original_title: String,
    pub emojis: String,
}

#[derive(Default)]
pub struct EmotitleState {
    pub pane_manifest: Option<PaneManifest>,
    pub tab_infos: Vec<TabInfo>,
    pane_entries: HashMap<PaneRef, Entry>,
    tab_entries: HashMap<usize, Entry>,
    pending_pane_restores: HashMap<PaneRef, String>,
    pending_tab_restores: HashMap<usize, String>,
}

impl EmotitleState {
    pub fn update_pane_manifest(&mut self, pane_manifest: PaneManifest) -> bool {
        let current_panes: HashSet<PaneRef> = pane_manifest
            .panes
            .values()
            .flat_map(|panes| panes.iter())
            .map(pane_ref_from_pane_info)
            .collect();

        self.pane_entries
            .retain(|pane_ref, _| current_panes.contains(pane_ref));

        self.pending_pane_restores
            .retain(|pane_ref, _| current_panes.contains(pane_ref));

        self.pane_manifest = Some(pane_manifest.clone());
        let focused: Vec<PaneRef> = pane_manifest
            .panes
            .values()
            .flat_map(|panes| panes.iter())
            .filter(|pane| pane.is_focused)
            .map(pane_ref_from_pane_info)
            .collect();
        let cleaned = self.clean_focused_panes_on_focus(&pane_manifest);
        let _ = focused;
        cleaned
    }

    pub fn update_tab_infos(&mut self, tab_infos: Vec<TabInfo>) -> bool {
        let current_tabs: HashSet<usize> = tab_infos.iter().map(|tab| tab.position).collect();

        self.tab_entries
            .retain(|tab_index, _| current_tabs.contains(tab_index));

        self.pending_tab_restores
            .retain(|tab_index, _| current_tabs.contains(tab_index));

        self.tab_infos = tab_infos.clone();
        let focused: Vec<usize> = tab_infos
            .iter()
            .filter(|tab| tab.active)
            .map(|tab| tab.position)
            .collect();
        let cleaned = self.clean_focused_tabs_on_focus(&tab_infos);
        let _ = focused;
        cleaned
    }

    pub fn resolve_tab_index_from_pane_id(&self, pane_id: u32) -> Option<usize> {
        self.pane_manifest.as_ref().and_then(|manifest| {
            manifest.panes.iter().find_map(|(tab_index, panes)| {
                panes
                    .iter()
                    .any(|pane| pane.id == pane_id)
                    .then_some(*tab_index)
            })
        })
    }

    pub fn focused_pane_ref(&self) -> Option<PaneRef> {
        self.pane_manifest.as_ref().and_then(|manifest| {
            manifest
                .panes
                .values()
                .flat_map(|panes| panes.iter())
                .find(|pane| pane.is_focused)
                .map(pane_ref_from_pane_info)
        })
    }

    pub fn focused_tab_index(&self) -> Option<usize> {
        self.tab_infos
            .iter()
            .find(|tab| tab.active)
            .map(|tab| tab.position)
    }

    pub fn pane_title(&self, pane_ref: &PaneRef) -> Option<String> {
        self.pane_manifest.as_ref().and_then(|manifest| {
            manifest
                .panes
                .values()
                .flat_map(|panes| panes.iter())
                .find(|pane| pane_matches(pane, pane_ref))
                .map(|pane| pane.title.clone())
        })
    }

    pub fn pane_effective_title(&self, pane_ref: &PaneRef) -> Option<String> {
        let pane_title = self.pane_title(pane_ref);
        let entry = self.pane_entries.get(pane_ref);

        match (pane_title, entry) {
            (Some(title), Some(entry)) if title == entry.original_title => {
                Some(title_with_emojis(&entry.original_title, &entry.emojis))
            }
            (Some(title), _) => Some(title),
            (None, Some(entry)) => Some(title_with_emojis(&entry.original_title, &entry.emojis)),
            (None, None) => None,
        }
    }

    pub fn tab_title(&self, tab_index: usize) -> Option<String> {
        self.tab_infos
            .iter()
            .find(|tab| tab.position == tab_index)
            .map(|tab| tab.name.clone())
    }

    pub fn tab_effective_title(&self, tab_index: usize) -> Option<String> {
        let tab_title = self.tab_title(tab_index);
        let entry = self.tab_entries.get(&tab_index);

        match (tab_title, entry) {
            (Some(title), Some(entry)) if title == entry.original_title => {
                Some(title_with_emojis(&entry.original_title, &entry.emojis))
            }
            (Some(title), _) => Some(title),
            (None, Some(entry)) => Some(title_with_emojis(&entry.original_title, &entry.emojis)),
            (None, None) => None,
        }
    }

    pub fn upsert_pane_entry(
        &mut self,
        pane_ref: PaneRef,
        original_title: String,
        emojis: String,
        _mode: Mode,
    ) {
        let original_title = self
            .pane_entries
            .get(&pane_ref)
            .map(|e| e.original_title.clone())
            .unwrap_or(original_title);
        self.pane_entries.insert(
            pane_ref,
            Entry {
                original_title,
                emojis,
            },
        );
    }

    pub fn upsert_tab_entry(
        &mut self,
        tab_index: usize,
        original_title: String,
        emojis: String,
        _mode: Mode,
    ) {
        let original_title = self
            .tab_entries
            .get(&tab_index)
            .map(|e| e.original_title.clone())
            .unwrap_or(original_title);
        self.tab_entries.insert(
            tab_index,
            Entry {
                original_title,
                emojis,
            },
        );
    }

    pub fn pane_original_title(&self, pane_ref: &PaneRef) -> Option<String> {
        self.pane_entries
            .get(pane_ref)
            .map(|entry| entry.original_title.clone())
    }

    pub fn tab_original_title(&self, tab_index: usize) -> Option<String> {
        self.tab_entries
            .get(&tab_index)
            .map(|entry| entry.original_title.clone())
    }

    fn clean_focused_panes_on_focus(&mut self, pane_manifest: &PaneManifest) -> bool {
        let mut set_timer = false;

        for pane in pane_manifest.panes.values().flat_map(|panes| panes.iter()) {
            if !pane.is_focused {
                continue;
            }

            let pane_ref = pane_ref_from_pane_info(pane);
            let original_title = self
                .pane_entries
                .get(&pane_ref)
                .map(|entry| entry.original_title.as_str())
                .unwrap_or_else(|| pane.title.split(" | ").next().unwrap_or(&pane.title));
            let cleaned_title = title_with_pinned_segments(original_title, &pane.title);

            if pane.title != original_title {
                let mut remove_entry = false;
                if let Some(entry) = self.pane_entries.get_mut(&pane_ref) {
                    let suffix = emojis_suffix_from_title(&entry.original_title, &cleaned_title);
                    if suffix.is_empty() {
                        remove_entry = true;
                    } else {
                        entry.emojis = suffix;
                    }
                }
                if remove_entry {
                    self.pane_entries.remove(&pane_ref);
                }
            }

            if cleaned_title != pane.title {
                self.pending_pane_restores.insert(pane_ref, cleaned_title);
                set_timer = true;
            }
        }

        set_timer
    }

    fn clean_focused_tabs_on_focus(&mut self, tab_infos: &[TabInfo]) -> bool {
        let mut set_timer = false;

        for tab in tab_infos {
            if !tab.active {
                continue;
            }

            let original_title = self
                .tab_entries
                .get(&tab.position)
                .map(|entry| entry.original_title.as_str())
                .unwrap_or_else(|| tab.name.split(" | ").next().unwrap_or(&tab.name));
            let cleaned_title = title_with_pinned_segments(original_title, &tab.name);

            if tab.name != original_title {
                let mut remove_entry = false;
                if let Some(entry) = self.tab_entries.get_mut(&tab.position) {
                    let suffix = emojis_suffix_from_title(&entry.original_title, &cleaned_title);
                    if suffix.is_empty() {
                        remove_entry = true;
                    } else {
                        entry.emojis = suffix;
                    }
                }
                if remove_entry {
                    self.tab_entries.remove(&tab.position);
                }
            }

            if cleaned_title != tab.name {
                self.pending_tab_restores
                    .insert(tab.position, cleaned_title);
                set_timer = true;
            }
        }

        set_timer
    }

    pub fn take_pending_pane_restores(&mut self) -> Vec<(PaneRef, String)> {
        self.pending_pane_restores.drain().collect()
    }

    pub fn take_pending_tab_restores(&mut self) -> Vec<(usize, String)> {
        self.pending_tab_restores.drain().collect()
    }
}

pub fn title_with_emojis(original_title: &str, emojis: &str) -> String {
    format!("{original_title} | {emojis}")
}

fn emojis_suffix_from_title(original_title: &str, title: &str) -> String {
    title
        .strip_prefix(&format!("{original_title} | "))
        .unwrap_or("")
        .to_string()
}

pub fn title_with_pinned_segments(original_title: &str, current_title: &str) -> String {
    let Some(rest) = current_title.strip_prefix(original_title) else {
        return original_title.to_string();
    };

    let pinned_segments: Vec<String> = rest
        .split(" | ")
        .map(str::trim)
        .filter(|segment| !segment.is_empty() && segment.starts_with('📌'))
        .map(str::to_string)
        .collect();

    if pinned_segments.is_empty() {
        original_title.to_string()
    } else {
        format!("{original_title} | {}", pinned_segments.join(" | "))
    }
}

fn pane_ref_from_pane_info(pane_info: &PaneInfo) -> PaneRef {
    if pane_info.is_plugin {
        PaneRef::Plugin(pane_info.id)
    } else {
        PaneRef::Terminal(pane_info.id)
    }
}

fn pane_matches(pane_info: &PaneInfo, pane_ref: &PaneRef) -> bool {
    match pane_ref {
        PaneRef::Terminal(id) => !pane_info.is_plugin && pane_info.id == *id,
        PaneRef::Plugin(id) => pane_info.is_plugin && pane_info.id == *id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zellij_tile::prelude::TabInfo;

    fn pane_info(id: u32, is_plugin: bool, is_focused: bool, title: &str) -> PaneInfo {
        PaneInfo {
            id,
            is_plugin,
            is_focused,
            is_fullscreen: false,
            is_floating: false,
            is_suppressed: false,
            title: title.to_string(),
            exited: false,
            exit_status: None,
            is_held: false,
            pane_x: 0,
            pane_content_x: 0,
            pane_y: 0,
            pane_content_y: 0,
            pane_rows: 0,
            pane_content_rows: 0,
            pane_columns: 0,
            pane_content_columns: 0,
            cursor_coordinates_in_pane: None,
            terminal_command: None,
            plugin_url: None,
            is_selectable: true,
            index_in_pane_group: Default::default(),
        }
    }

    fn tab_info(position: usize, active: bool, name: &str) -> TabInfo {
        TabInfo {
            position,
            name: name.to_string(),
            active,
            panes_to_hide: 0,
            is_fullscreen_active: false,
            is_sync_panes_active: false,
            are_floating_panes_visible: true,
            other_focused_clients: vec![],
            active_swap_layout_name: None,
            is_swap_layout_dirty: false,
            viewport_rows: 0,
            viewport_columns: 0,
            display_area_rows: 0,
            display_area_columns: 0,
            selectable_tiled_panes_count: 0,
            selectable_floating_panes_count: 0,
        }
    }

    #[test]
    fn resolve_tab_index_from_pane_id() {
        let mut state = EmotitleState::default();
        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(10, false, false, "zsh")]);
        panes.insert(1, vec![pane_info(20, false, false, "vim")]);
        state.update_pane_manifest(PaneManifest { panes });

        assert_eq!(state.resolve_tab_index_from_pane_id(20), Some(1));
        assert_eq!(state.resolve_tab_index_from_pane_id(999), None);
    }

    #[test]
    fn temp_pane_is_restored_when_it_gets_focus() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "🚀".to_string(),
            Mode::Temp,
        );

        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(10, false, true, "bash | 🚀")]);
        let set_timer = state.update_pane_manifest(PaneManifest { panes });

        assert!(set_timer);
        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_none());

        let restored = state.take_pending_pane_restores();
        assert_eq!(restored, vec![(PaneRef::Terminal(10), "bash".to_string())]);
    }

    #[test]
    fn all_temp_panes_are_restored_on_focus() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "🚀".to_string(),
            Mode::Temp,
        );
        state.upsert_pane_entry(
            PaneRef::Terminal(20),
            "vim".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );
        state.upsert_pane_entry(
            PaneRef::Terminal(30),
            "zsh".to_string(),
            "🔥".to_string(),
            Mode::Permanent,
        );

        let mut panes = HashMap::new();
        panes.insert(
            0,
            vec![
                pane_info(10, false, true, "bash | 🚀"),
                pane_info(20, false, false, "vim | 📚"),
                pane_info(30, false, false, "zsh | 🔥"),
            ],
        );
        let set_timer = state.update_pane_manifest(PaneManifest { panes });

        assert!(set_timer);
        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_none());
        assert!(state.pane_original_title(&PaneRef::Terminal(20)).is_none());
        assert!(state.pane_original_title(&PaneRef::Terminal(30)).is_some());

        let restored = state.take_pending_pane_restores();
        assert_eq!(restored.len(), 2, "should restore both temp panes");
        assert!(restored.iter().any(|(p, _)| *p == PaneRef::Terminal(10)));
        assert!(restored.iter().any(|(p, _)| *p == PaneRef::Terminal(20)));
    }

    #[test]
    fn permanent_panes_preserved_on_focus() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "📌🚀".to_string(),
            Mode::Permanent,
        );
        state.upsert_pane_entry(
            PaneRef::Terminal(20),
            "vim".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

        let mut panes = HashMap::new();
        panes.insert(
            0,
            vec![
                pane_info(10, false, true, "bash | 📌🚀"),
                pane_info(20, false, false, "vim | 📚"),
            ],
        );
        let set_timer = state.update_pane_manifest(PaneManifest { panes });

        assert!(set_timer);
        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_some());
        assert!(state.pane_original_title(&PaneRef::Terminal(20)).is_none());

        let restored = state.take_pending_pane_restores();
        assert_eq!(restored, vec![(PaneRef::Terminal(20), "vim".to_string())]);
    }

    #[test]
    fn pane_focus_keeps_only_pinned_segments() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "📌🚀".to_string(),
            Mode::Permanent,
        );

        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(10, false, true, "bash | 📚 | 📌🚀 | 🚗")]);

        let set_timer = state.update_pane_manifest(PaneManifest { panes });

        assert!(set_timer);
        let restored = state.take_pending_pane_restores();
        assert_eq!(
            restored,
            vec![(PaneRef::Terminal(10), "bash | 📌🚀".to_string())]
        );
    }

    #[test]
    fn temp_tab_is_restored_when_it_gets_focus() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(2, "build".to_string(), "✅".to_string(), Mode::Temp);

        let set_timer = state.update_tab_infos(vec![tab_info(2, true, "build | ✅")]);

        assert!(set_timer);
        assert!(state.tab_original_title(2).is_none());

        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(2, "build".to_string())]);
    }

    #[test]
    fn all_temp_tabs_are_restored_on_focus() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(1, "main".to_string(), "🚀".to_string(), Mode::Temp);
        state.upsert_tab_entry(2, "build".to_string(), "📚".to_string(), Mode::Temp);
        state.upsert_tab_entry(3, "test".to_string(), "📌✅".to_string(), Mode::Permanent);

        let set_timer = state.update_tab_infos(vec![
            tab_info(1, true, "main | 🚀"),
            tab_info(2, false, "build | 📚"),
            tab_info(3, false, "test | 📌✅"),
        ]);

        assert!(set_timer);
        assert!(state.tab_original_title(1).is_none());
        assert!(state.tab_original_title(2).is_none());
        assert!(state.tab_original_title(3).is_some());

        let restored = state.take_pending_tab_restores();
        assert_eq!(restored.len(), 2);
    }

    #[test]
    fn permanent_tabs_preserved_on_focus() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(1, "main".to_string(), "📌🚀".to_string(), Mode::Permanent);
        state.upsert_tab_entry(2, "build".to_string(), "📚".to_string(), Mode::Temp);

        let set_timer = state.update_tab_infos(vec![
            tab_info(1, true, "main | 📌🚀"),
            tab_info(2, false, "build | 📚"),
        ]);

        assert!(set_timer);
        assert!(state.tab_original_title(1).is_some());
        assert!(state.tab_original_title(2).is_none());

        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(2, "build".to_string())]);
    }

    #[test]
    fn tab_focus_keeps_only_pinned_segments() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(1, "main".to_string(), "📌✅".to_string(), Mode::Permanent);

        let set_timer = state.update_tab_infos(vec![tab_info(1, true, "main | 🔔 | 📌✅ | 📚")]);

        assert!(set_timer);
        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(1, "main | 📌✅".to_string())]);
    }

    #[test]
    fn pane_entry_is_cleared_after_focus_cleanup_without_pinned_segments() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(10, false, true, "bash")]);

        let set_timer = state.update_pane_manifest(PaneManifest { panes });

        assert!(!set_timer);
        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_none());
        assert_eq!(
            state.pane_effective_title(&PaneRef::Terminal(10)),
            Some("bash".to_string())
        );
    }

    #[test]
    fn tab_entry_is_cleared_after_focus_cleanup_without_pinned_segments() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(1, "main".to_string(), "📚".to_string(), Mode::Temp);

        let set_timer = state.update_tab_infos(vec![tab_info(1, true, "main")]);

        assert!(!set_timer);
        assert!(state.tab_original_title(1).is_none());
        assert_eq!(state.tab_effective_title(1), Some("main".to_string()));
    }

    #[test]
    fn pane_entry_is_preserved_when_focused_title_is_still_original() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "📌🚀".to_string(),
            Mode::Permanent,
        );

        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(10, false, true, "bash")]);

        let set_timer = state.update_pane_manifest(PaneManifest { panes });

        assert!(!set_timer);
        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_some());
        assert_eq!(
            state.pane_effective_title(&PaneRef::Terminal(10)),
            Some("bash | 📌🚀".to_string())
        );
    }

    #[test]
    fn tab_entry_is_preserved_when_focused_title_is_still_original() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(1, "main".to_string(), "📌🚀".to_string(), Mode::Permanent);

        let set_timer = state.update_tab_infos(vec![tab_info(1, true, "main")]);

        assert!(!set_timer);
        assert!(state.tab_original_title(1).is_some());
        assert_eq!(
            state.tab_effective_title(1),
            Some("main | 📌🚀".to_string())
        );
    }

    #[test]
    fn deleted_pane_entry_is_removed_on_manifest_update() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "🚀".to_string(),
            Mode::Permanent,
        );

        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(10, false, false, "bash | 🚀")]);
        state.update_pane_manifest(PaneManifest {
            panes: panes.clone(),
        });

        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_some());

        let mut panes_after_delete = HashMap::new();
        panes_after_delete.insert(0, vec![pane_info(20, false, true, "zsh")]);
        state.update_pane_manifest(PaneManifest {
            panes: panes_after_delete,
        });

        assert!(
            state.pane_original_title(&PaneRef::Terminal(10)).is_none(),
            "deleted pane entry should be removed from pane_entries"
        );
    }

    #[test]
    fn other_pane_entry_is_preserved_when_pane_is_deleted() {
        let mut state = EmotitleState::default();
        state.upsert_pane_entry(
            PaneRef::Terminal(10),
            "bash".to_string(),
            "🚀".to_string(),
            Mode::Permanent,
        );
        state.upsert_pane_entry(
            PaneRef::Terminal(20),
            "vim".to_string(),
            "📚".to_string(),
            Mode::Permanent,
        );

        let mut panes = HashMap::new();
        panes.insert(
            0,
            vec![
                pane_info(10, false, false, "bash | 🚀"),
                pane_info(20, false, true, "vim | 📚"),
            ],
        );
        state.update_pane_manifest(PaneManifest { panes });

        let mut panes_after_delete = HashMap::new();
        panes_after_delete.insert(0, vec![pane_info(20, false, true, "vim | 📚")]);
        state.update_pane_manifest(PaneManifest {
            panes: panes_after_delete,
        });

        assert!(state.pane_original_title(&PaneRef::Terminal(20)).is_some());
        assert!(
            state.pane_original_title(&PaneRef::Terminal(10)).is_none(),
            "deleted pane entry should be removed"
        );
    }
}
