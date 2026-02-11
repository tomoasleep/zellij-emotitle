use std::collections::HashMap;

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
    pub mode: Mode,
}

#[derive(Default)]
pub struct EmotitleState {
    pub pane_manifest: Option<PaneManifest>,
    pub tab_infos: Vec<TabInfo>,
    pane_entries: HashMap<PaneRef, Entry>,
    tab_entries: HashMap<usize, Entry>,
}

impl EmotitleState {
    pub fn update_pane_manifest(&mut self, pane_manifest: PaneManifest) -> Vec<(PaneRef, String)> {
        self.pane_manifest = Some(pane_manifest.clone());
        let focused: Vec<PaneRef> = pane_manifest
            .panes
            .values()
            .flat_map(|panes| panes.iter())
            .filter(|pane| pane.is_focused)
            .map(pane_ref_from_pane_info)
            .collect();
        self.take_temp_panes_on_focus(&focused)
    }

    pub fn update_tab_infos(&mut self, tab_infos: Vec<TabInfo>) -> Vec<(usize, String)> {
        self.tab_infos = tab_infos.clone();
        let focused: Vec<usize> = tab_infos
            .iter()
            .filter(|tab| tab.active)
            .map(|tab| tab.position)
            .collect();
        self.take_temp_tabs_on_focus(&focused)
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

    pub fn tab_title(&self, tab_index: usize) -> Option<String> {
        self.tab_infos
            .iter()
            .find(|tab| tab.position == tab_index)
            .map(|tab| tab.name.clone())
    }

    pub fn upsert_pane_entry(
        &mut self,
        pane_ref: PaneRef,
        original_title: String,
        _emojis: String,
        mode: Mode,
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
                mode,
            },
        );
    }

    pub fn upsert_tab_entry(
        &mut self,
        tab_index: usize,
        original_title: String,
        _emojis: String,
        mode: Mode,
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
                mode,
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

    fn take_temp_panes_on_focus(&mut self, focused: &[PaneRef]) -> Vec<(PaneRef, String)> {
        let focused_set: std::collections::HashSet<PaneRef> = focused.iter().cloned().collect();
        let to_restore: Vec<PaneRef> = self
            .pane_entries
            .iter()
            .filter(|(pane_ref, entry)| entry.mode == Mode::Temp && focused_set.contains(*pane_ref))
            .map(|(pane_ref, _)| pane_ref.clone())
            .collect();

        to_restore
            .into_iter()
            .filter_map(|pane_ref| {
                self.pane_entries
                    .remove(&pane_ref)
                    .map(|entry| (pane_ref, entry.original_title))
            })
            .collect()
    }

    fn take_temp_tabs_on_focus(&mut self, focused: &[usize]) -> Vec<(usize, String)> {
        let focused_set: std::collections::HashSet<usize> = focused.iter().copied().collect();
        let to_restore: Vec<usize> = self
            .tab_entries
            .iter()
            .filter(|(tab_index, entry)| {
                entry.mode == Mode::Temp && focused_set.contains(tab_index)
            })
            .map(|(tab_index, _)| *tab_index)
            .collect();

        to_restore
            .into_iter()
            .filter_map(|tab_index| {
                self.tab_entries
                    .remove(&tab_index)
                    .map(|entry| (tab_index, entry.original_title))
            })
            .collect()
    }
}

pub fn title_with_emojis(original_title: &str, emojis: &str) -> String {
    format!("{original_title} | {emojis}")
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
        let restored = state.update_pane_manifest(PaneManifest { panes });

        assert_eq!(restored, vec![(PaneRef::Terminal(10), "bash".to_string())]);
        assert!(state.pane_original_title(&PaneRef::Terminal(10)).is_none());
    }

    #[test]
    fn temp_tab_is_restored_when_it_gets_focus() {
        let mut state = EmotitleState::default();
        state.upsert_tab_entry(2, "build".to_string(), "✅".to_string(), Mode::Temp);

        let restored = state.update_tab_infos(vec![tab_info(2, true, "build | ✅")]);

        assert_eq!(restored, vec![(2, "build".to_string())]);
        assert!(state.tab_original_title(2).is_none());
    }
}
