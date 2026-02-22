use std::collections::{HashMap, HashSet};

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
struct PendingTabRestore {
    title: String,
    anchor_pane_id: Option<u32>,
}

#[derive(Default)]
pub struct EmotitleState {
    pub pane_manifest: Option<PaneManifest>,
    pub tab_infos: Vec<TabInfo>,
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

        self.pending_pane_restores
            .retain(|pane_ref, _| current_panes.contains(pane_ref));

        self.remap_tab_state_with_manifest(&pane_manifest);

        self.pane_manifest = Some(pane_manifest.clone());
        self.clean_focused_panes_on_focus(&pane_manifest)
    }

    pub fn update_tab_infos(&mut self, tab_infos: Vec<TabInfo>) -> bool {
        let current_tabs: HashSet<usize> = tab_infos.iter().map(|tab| tab.position).collect();

        self.pending_tab_restores
            .retain(|tab_index, _| current_tabs.contains(tab_index));

        self.tab_infos = tab_infos.clone();
        self.clean_focused_tabs_on_focus(&tab_infos)
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
        self.pane_title(pane_ref)
    }

    pub fn tab_title(&self, tab_index: usize) -> Option<String> {
        self.tab_infos
            .iter()
            .find(|tab| tab.position == tab_index)
            .map(|tab| tab.name.clone())
    }

    pub fn tab_effective_title(&self, tab_index: usize) -> Option<String> {
        self.tab_title(tab_index)
    }

    pub fn tab_rename_target(&self, tab_index: usize) -> Option<u32> {
        let mut positions: Vec<usize> = self.tab_infos.iter().map(|tab| tab.position).collect();
        positions.sort_unstable();
        let ordinal = positions
            .iter()
            .position(|position| *position == tab_index)?;
        Some((ordinal + 1) as u32)
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
            let original_title = extract_original_title(&pane.title);
            let cleaned_title = title_with_pinned_segments(&original_title, &pane.title);

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

            let original_title = extract_original_title(&tab.name);
            let cleaned_title = title_with_pinned_segments(&original_title, &tab.name);

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

fn extract_original_title(title: &str) -> String {
    title.split(" | ").next().unwrap_or(title).to_string()
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
