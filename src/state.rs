use std::collections::{HashMap, HashSet};

use crate::command::Mode;
use serde::Serialize;
use zellij_tile::prelude::{PaneInfo, PaneManifest, TabInfo};

#[derive(Serialize)]
pub struct PaneDebugInfo {
    pub id: u32,
    pub is_plugin: bool,
    pub is_focused: bool,
    pub title: String,
}

#[derive(Serialize)]
pub struct TabDebugInfo {
    pub position: usize,
    pub name: String,
    pub active: bool,
    pub panes: Vec<PaneDebugInfo>,
}

#[derive(Serialize)]
pub struct InfoDebug {
    pub tabs: Vec<TabDebugInfo>,
    pub focused_tab_index: Option<usize>,
    pub focused_pane: Option<String>,
}

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

#[derive(Debug, Clone)]
struct TabEntry {
    entry: Entry,
    anchor_pane_id: Option<u32>,
}

#[derive(Debug, Clone)]
struct PendingTabRestore {
    title: String,
    anchor_pane_id: Option<u32>,
}

#[derive(Default)]
pub struct EmotitleState {
    pub pane_manifest: Option<PaneManifest>,
    pub tab_infos: Vec<TabInfo>,
    pane_entries: HashMap<PaneRef, Entry>,
    tab_entries: HashMap<usize, TabEntry>,
    pending_pane_restores: HashMap<PaneRef, String>,
    pending_tab_restores: HashMap<usize, PendingTabRestore>,
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

        self.remap_tab_state_with_manifest(&pane_manifest);

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
        let manifest = self.pane_manifest.as_ref()?;
        let candidates: Vec<usize> = manifest
            .panes
            .iter()
            .filter_map(|(tab_position, panes)| {
                panes
                    .iter()
                    .any(|pane| !pane.is_plugin && pane.id == pane_id)
                    .then_some(*tab_position)
            })
            .filter_map(|manifest_tab_position| {
                self.tab_position_for_manifest_position(manifest_tab_position)
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return candidates.first().copied();
        }

        if let Some(focused_tab_position) = self.focused_tab_index() {
            if candidates.contains(&focused_tab_position) {
                return Some(focused_tab_position);
            }
        }

        if let Some(focused_tab_position) = self.focused_tab_index_from_manifest() {
            if candidates.contains(&focused_tab_position) {
                return Some(focused_tab_position);
            }
        }

        manifest
            .panes
            .iter()
            .find_map(|(tab_position, panes)| {
                panes
                    .iter()
                    .any(|pane| !pane.is_plugin && pane.id == pane_id && pane.is_focused)
                    .then_some(*tab_position)
            })
            .and_then(|manifest_tab_position| {
                self.tab_position_for_manifest_position(manifest_tab_position)
            })
            .or_else(|| candidates.first().copied())
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

    pub fn focused_tab_index_from_manifest(&self) -> Option<usize> {
        let manifest = self.pane_manifest.as_ref()?;
        manifest
            .panes
            .iter()
            .find_map(|(tab_position, panes)| {
                panes
                    .iter()
                    .any(|pane| pane.is_focused && !pane.is_plugin)
                    .then_some(*tab_position)
            })
            .and_then(|manifest_tab_position| {
                self.tab_position_for_manifest_position(manifest_tab_position)
            })
            .or_else(|| {
                manifest
                    .panes
                    .iter()
                    .find_map(|(tab_position, panes)| {
                        panes
                            .iter()
                            .any(|pane| pane.is_focused)
                            .then_some(*tab_position)
                    })
                    .and_then(|manifest_tab_position| {
                        self.tab_position_for_manifest_position(manifest_tab_position)
                    })
            })
    }

    pub fn tab_resolution_debug(&self) -> String {
        let tab_positions = self
            .tab_infos
            .iter()
            .map(|tab| format!("{}:{}", tab.position, tab.active))
            .collect::<Vec<_>>()
            .join(",");
        let manifest_positions = self
            .pane_manifest
            .as_ref()
            .map(|manifest| {
                manifest
                    .panes
                    .keys()
                    .copied()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|position| position.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| "none".to_string());
        let manifest_panes = self
            .pane_manifest
            .as_ref()
            .map(|manifest| {
                let mut positions: Vec<usize> = manifest.panes.keys().copied().collect();
                positions.sort_unstable();
                positions
                    .iter()
                    .filter_map(|position| {
                        manifest.panes.get(position).map(|panes| {
                            let pane_desc = panes
                                .iter()
                                .map(|p| format!("{}:{}:{}", p.id, p.is_plugin, p.is_focused))
                                .collect::<Vec<_>>()
                                .join(";");
                            format!("{position}=[{pane_desc}]")
                        })
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_else(|| "none".to_string());
        format!(
            "tab_infos=[{tab_positions}] manifest_positions=[{manifest_positions}] manifest_panes=[{manifest_panes}] focused_tab={:?} focused_tab_manifest={:?} focused_pane={:?}",
            self.focused_tab_index(),
            self.focused_tab_index_from_manifest(),
            self.focused_pane_ref()
        )
    }

    pub fn info_debug(&self) -> String {
        let manifest = self.pane_manifest.as_ref();

        let mut tab_debug_infos: Vec<TabDebugInfo> = self
            .tab_infos
            .iter()
            .map(|tab| {
                let panes = manifest
                    .and_then(|m| {
                        let manifest_position = self
                            .manifest_tab_position_for_tab_position(tab.position)
                            .unwrap_or(tab.position);
                        m.panes.get(&manifest_position)
                    })
                    .map(|panes| {
                        panes
                            .iter()
                            .map(|p| PaneDebugInfo {
                                id: p.id,
                                is_plugin: p.is_plugin,
                                is_focused: p.is_focused,
                                title: p.title.clone(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                TabDebugInfo {
                    position: tab.position,
                    name: tab.name.clone(),
                    active: tab.active,
                    panes,
                }
            })
            .collect();

        tab_debug_infos.sort_by_key(|t| t.position);

        let info = InfoDebug {
            tabs: tab_debug_infos,
            focused_tab_index: self.focused_tab_index(),
            focused_pane: self.focused_pane_ref().map(|p| format!("{:?}", p)),
        };

        serde_json::to_string(&info).unwrap_or_else(|_| "{}".to_string())
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
        let entry = self
            .tab_entries
            .get(&tab_index)
            .map(|tab_entry| &tab_entry.entry);

        match (tab_title, entry) {
            (Some(title), Some(entry)) if title == entry.original_title => {
                Some(title_with_emojis(&entry.original_title, &entry.emojis))
            }
            (Some(title), _) => Some(title),
            (None, Some(entry)) => Some(title_with_emojis(&entry.original_title, &entry.emojis)),
            (None, None) => None,
        }
    }

    pub fn tab_rename_target(&self, tab_index: usize) -> Option<u32> {
        let mut positions: Vec<usize> = self.tab_infos.iter().map(|tab| tab.position).collect();
        positions.sort_unstable();
        let ordinal = positions
            .iter()
            .position(|position| *position == tab_index)?;
        Some((ordinal + 1) as u32)
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
        anchor_pane_id: Option<u32>,
        original_title: String,
        emojis: String,
        _mode: Mode,
    ) {
        let original_title = self
            .tab_entries
            .get(&tab_index)
            .filter(|e| e.anchor_pane_id == anchor_pane_id)
            .map(|e| e.entry.original_title.clone())
            .unwrap_or(original_title);
        self.tab_entries.insert(
            tab_index,
            TabEntry {
                entry: Entry {
                    original_title,
                    emojis,
                },
                anchor_pane_id,
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
            .map(|entry| entry.entry.original_title.clone())
    }

    pub fn tab_anchor_pane_id(&self, tab_index: usize) -> Option<u32> {
        let manifest = self.pane_manifest.as_ref()?;
        let manifest_tab_position = self.manifest_tab_position_for_tab_position(tab_index)?;
        let panes = manifest.panes.get(&manifest_tab_position)?;
        panes
            .iter()
            .find(|pane| !pane.is_plugin && pane.is_focused)
            .or_else(|| panes.iter().find(|pane| !pane.is_plugin))
            .map(|pane| pane.id)
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
            let tab_index = tab.position;

            let anchor_pane_id = self.tab_anchor_pane_id(tab_index);

            let original_title = self
                .tab_entries
                .get(&tab_index)
                .map(|entry| entry.entry.original_title.as_str())
                .unwrap_or_else(|| tab.name.split(" | ").next().unwrap_or(&tab.name));
            let cleaned_title = title_with_pinned_segments(original_title, &tab.name);

            if tab.name != original_title {
                let mut remove_entry = false;
                if let Some(entry) = self.tab_entries.get_mut(&tab_index) {
                    let suffix =
                        emojis_suffix_from_title(&entry.entry.original_title, &cleaned_title);
                    if suffix.is_empty() {
                        remove_entry = true;
                    } else {
                        entry.entry.emojis = suffix;
                    }
                }
                if remove_entry {
                    self.tab_entries.remove(&tab_index);
                }
            }

            if cleaned_title != tab.name {
                self.pending_tab_restores.insert(
                    tab_index,
                    PendingTabRestore {
                        title: cleaned_title,
                        anchor_pane_id,
                    },
                );
                set_timer = true;
            }
        }

        set_timer
    }

    pub fn take_pending_pane_restores(&mut self) -> Vec<(PaneRef, String)> {
        self.pending_pane_restores.drain().collect()
    }

    pub fn take_pending_tab_restores(&mut self) -> Vec<(usize, String)> {
        let pending = std::mem::take(&mut self.pending_tab_restores);
        let mut resolved = Vec::new();
        let current_tabs: HashSet<usize> = self.tab_infos.iter().map(|tab| tab.position).collect();

        for (previous_index, restore) in pending {
            if let Some(anchor_pane_id) = restore.anchor_pane_id {
                if let Some(tab_index) = self.resolve_tab_index_from_pane_id(anchor_pane_id) {
                    resolved.push((tab_index, restore.title));
                } else if current_tabs.contains(&previous_index) {
                    resolved.push((previous_index, restore.title));
                } else {
                    self.pending_tab_restores.insert(previous_index, restore);
                }
            } else if current_tabs.contains(&previous_index) {
                resolved.push((previous_index, restore.title));
            } else {
                self.pending_tab_restores.insert(previous_index, restore);
            }
        }

        resolved
    }

    pub fn has_pending_tab_restores(&self) -> bool {
        !self.pending_tab_restores.is_empty()
    }

    pub fn clear_pending_tab_restore(&mut self, tab_index: usize) {
        let pending = std::mem::take(&mut self.pending_tab_restores);
        let mut retained = HashMap::new();

        for (pending_index, restore) in pending {
            let resolved_index = restore
                .anchor_pane_id
                .and_then(|anchor| self.resolve_tab_index_from_pane_id(anchor))
                .unwrap_or(pending_index);
            if resolved_index != tab_index {
                retained.insert(pending_index, restore);
            }
        }

        self.pending_tab_restores = retained;
    }

    fn remap_tab_state_with_manifest(&mut self, pane_manifest: &PaneManifest) {
        let mut manifest_positions: Vec<usize> = pane_manifest.panes.keys().copied().collect();
        manifest_positions.sort_unstable();
        let mut tab_positions: Vec<usize> = self.tab_infos.iter().map(|tab| tab.position).collect();
        tab_positions.sort_unstable();

        let pane_id_to_tab_index: HashMap<u32, usize> = pane_manifest
            .panes
            .iter()
            .flat_map(|(tab_position, panes)| {
                let resolved_tab_position = if tab_positions.contains(tab_position) {
                    Some(*tab_position)
                } else {
                    manifest_positions
                        .iter()
                        .position(|position| position == tab_position)
                        .and_then(|ordinal| tab_positions.get(ordinal).copied())
                };

                resolved_tab_position
                    .map(|tab_position| panes.iter().map(move |pane| (pane.id, tab_position)))
                    .into_iter()
                    .flatten()
            })
            .collect();

        let mut remapped_entries = HashMap::new();
        for (previous_index, tab_entry) in self.tab_entries.drain() {
            if let Some(anchor_pane_id) = tab_entry.anchor_pane_id {
                if let Some(new_index) = pane_id_to_tab_index.get(&anchor_pane_id) {
                    remapped_entries.insert(*new_index, tab_entry);
                } else {
                    remapped_entries.insert(previous_index, tab_entry);
                }
            } else {
                remapped_entries.insert(previous_index, tab_entry);
            }
        }
        self.tab_entries = remapped_entries;

        let mut remapped_restores = HashMap::new();
        for (previous_index, restore) in self.pending_tab_restores.drain() {
            if let Some(anchor_pane_id) = restore.anchor_pane_id {
                if let Some(new_index) = pane_id_to_tab_index.get(&anchor_pane_id) {
                    remapped_restores.insert(*new_index, restore);
                } else {
                    remapped_restores.insert(previous_index, restore);
                }
            } else {
                remapped_restores.insert(previous_index, restore);
            }
        }
        self.pending_tab_restores = remapped_restores;
    }

    fn manifest_tab_position_for_tab_position(&self, tab_position: usize) -> Option<usize> {
        let manifest = self.pane_manifest.as_ref()?;
        if manifest.panes.contains_key(&tab_position) {
            return Some(tab_position);
        }

        let mut tab_positions: Vec<usize> = self.tab_infos.iter().map(|tab| tab.position).collect();
        tab_positions.sort_unstable();
        let ordinal = tab_positions
            .iter()
            .position(|position| *position == tab_position)?;

        let mut manifest_positions: Vec<usize> = manifest.panes.keys().copied().collect();
        manifest_positions.sort_unstable();
        manifest_positions.get(ordinal).copied()
    }

    fn tab_position_for_manifest_position(&self, manifest_tab_position: usize) -> Option<usize> {
        if self
            .tab_infos
            .iter()
            .any(|tab| tab.position == manifest_tab_position)
        {
            return Some(manifest_tab_position);
        }

        let mut manifest_positions: Vec<usize> =
            self.pane_manifest.as_ref()?.panes.keys().copied().collect();
        manifest_positions.sort_unstable();
        let ordinal = manifest_positions
            .iter()
            .position(|position| *position == manifest_tab_position)?;

        let mut tab_positions: Vec<usize> = self.tab_infos.iter().map(|tab| tab.position).collect();
        tab_positions.sort_unstable();
        tab_positions.get(ordinal).copied()
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

    fn update_tab_pane_manifest(state: &mut EmotitleState, tabs: &[(usize, u32, bool)]) {
        let mut panes = HashMap::new();
        for (tab_index, pane_id, focused) in tabs {
            panes.insert(
                *tab_index,
                vec![pane_info(*pane_id, false, *focused, "pane")],
            );
        }
        state.update_pane_manifest(PaneManifest { panes });
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
    fn resolve_tab_index_from_pane_id_ignores_plugin_panes() {
        let mut state = EmotitleState::default();
        let mut panes = HashMap::new();
        panes.insert(
            0,
            vec![
                pane_info(2, false, false, "shell"),
                pane_info(2, true, false, "plugin"),
            ],
        );
        panes.insert(1, vec![pane_info(2, true, true, "plugin-focused")]);
        state.update_pane_manifest(PaneManifest { panes });

        assert_eq!(state.resolve_tab_index_from_pane_id(2), Some(0));
    }

    #[test]
    fn resolve_tab_index_from_pane_id_prefers_focused_when_ids_overlap() {
        let mut state = EmotitleState::default();
        state.update_tab_infos(vec![tab_info(0, false, "A"), tab_info(1, true, "B")]);

        let mut panes = HashMap::new();
        panes.insert(0, vec![pane_info(0, false, false, "a")]);
        panes.insert(1, vec![pane_info(0, false, true, "b")]);
        state.update_pane_manifest(PaneManifest { panes });

        assert_eq!(state.resolve_tab_index_from_pane_id(0), Some(1));
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
        update_tab_pane_manifest(&mut state, &[(2, 20, true)]);
        state.upsert_tab_entry(
            2,
            Some(20),
            "build".to_string(),
            "✅".to_string(),
            Mode::Temp,
        );

        let set_timer = state.update_tab_infos(vec![tab_info(2, true, "build | ✅")]);

        assert!(set_timer);
        assert!(state.tab_original_title(2).is_none());

        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(2, "build".to_string())]);
    }

    #[test]
    fn all_temp_tabs_are_restored_on_focus() {
        let mut state = EmotitleState::default();
        update_tab_pane_manifest(&mut state, &[(1, 11, true), (2, 22, false), (3, 33, false)]);
        state.upsert_tab_entry(
            1,
            Some(11),
            "main".to_string(),
            "🚀".to_string(),
            Mode::Temp,
        );
        state.upsert_tab_entry(
            2,
            Some(22),
            "build".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );
        state.upsert_tab_entry(
            3,
            Some(33),
            "test".to_string(),
            "📌✅".to_string(),
            Mode::Permanent,
        );

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
        update_tab_pane_manifest(&mut state, &[(1, 11, true), (2, 22, false)]);
        state.upsert_tab_entry(
            1,
            Some(11),
            "main".to_string(),
            "📌🚀".to_string(),
            Mode::Permanent,
        );
        state.upsert_tab_entry(
            2,
            Some(22),
            "build".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

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
        update_tab_pane_manifest(&mut state, &[(1, 11, true)]);
        state.upsert_tab_entry(
            1,
            Some(11),
            "main".to_string(),
            "📌✅".to_string(),
            Mode::Permanent,
        );

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
        update_tab_pane_manifest(&mut state, &[(1, 11, true)]);
        state.upsert_tab_entry(
            1,
            Some(11),
            "main".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

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
        update_tab_pane_manifest(&mut state, &[(1, 11, true)]);
        state.upsert_tab_entry(
            1,
            Some(11),
            "main".to_string(),
            "📌🚀".to_string(),
            Mode::Permanent,
        );

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

    #[test]
    fn tab_entry_tracks_anchor_when_tab_is_inserted_before_it() {
        let mut state = EmotitleState::default();
        update_tab_pane_manifest(&mut state, &[(0, 10, false), (1, 20, true)]);
        state.upsert_tab_entry(
            1,
            Some(20),
            "work".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

        update_tab_pane_manifest(&mut state, &[(0, 99, false), (1, 10, false), (2, 20, true)]);

        let set_timer = state.update_tab_infos(vec![
            tab_info(0, false, "new"),
            tab_info(1, false, "main"),
            tab_info(2, true, "work | 📚"),
        ]);

        assert!(set_timer);
        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(2, "work".to_string())]);
    }

    #[test]
    fn tab_entry_tracks_anchor_when_tab_before_it_is_deleted() {
        let mut state = EmotitleState::default();
        update_tab_pane_manifest(&mut state, &[(0, 99, false), (1, 20, true)]);
        state.upsert_tab_entry(
            1,
            Some(20),
            "work".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

        update_tab_pane_manifest(&mut state, &[(0, 20, true)]);

        let set_timer = state.update_tab_infos(vec![tab_info(0, true, "work | 📚")]);

        assert!(set_timer);
        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(0, "work".to_string())]);
    }

    #[test]
    fn pending_tab_restore_falls_back_to_previous_index_when_anchor_is_stale() {
        let mut state = EmotitleState::default();
        update_tab_pane_manifest(&mut state, &[(0, 10, false), (1, 20, true)]);
        state.upsert_tab_entry(
            0,
            Some(10),
            "work".to_string(),
            "📚".to_string(),
            Mode::Temp,
        );

        let set_timer = state.update_tab_infos(vec![tab_info(0, true, "work | 📚")]);
        assert!(set_timer);

        update_tab_pane_manifest(&mut state, &[(0, 20, true)]);

        let restored = state.take_pending_tab_restores();
        assert_eq!(restored, vec![(0, "work".to_string())]);
    }

    #[test]
    fn tab_entry_original_title_is_replaced_when_anchor_changes() {
        let mut state = EmotitleState::default();

        state.upsert_tab_entry(0, Some(10), "old".to_string(), "📚".to_string(), Mode::Temp);
        state.upsert_tab_entry(0, Some(20), "new".to_string(), "✅".to_string(), Mode::Temp);

        assert_eq!(state.tab_original_title(0), Some("new".to_string()));
    }

    #[test]
    fn clear_pending_tab_restore_removes_matching_resolved_tab() {
        let mut state = EmotitleState::default();
        let mut panes = HashMap::new();
        panes.insert(2, vec![pane_info(10, false, false, "work")]);
        panes.insert(1, vec![pane_info(20, false, false, "other")]);
        state.update_pane_manifest(PaneManifest { panes });
        state.pending_tab_restores.insert(
            0,
            PendingTabRestore {
                title: "work".to_string(),
                anchor_pane_id: Some(10),
            },
        );
        state.pending_tab_restores.insert(
            1,
            PendingTabRestore {
                title: "other".to_string(),
                anchor_pane_id: Some(20),
            },
        );

        state.clear_pending_tab_restore(2);
        assert!(!state.pending_tab_restores.contains_key(&0));
        assert!(state.pending_tab_restores.contains_key(&1));

        state.clear_pending_tab_restore(1);
        assert!(state.pending_tab_restores.is_empty());
    }
}
