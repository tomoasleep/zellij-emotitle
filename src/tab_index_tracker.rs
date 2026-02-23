use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

const MAX_EVENT_HISTORY: usize = 200;

#[derive(Hash, Eq, PartialEq, Clone, Serialize, Debug)]
pub struct PaneKey {
    pub is_plugin: bool,
    pub id: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct TabIndexEvent {
    pub seq: u64,
    pub event_type: TabIndexEventType,
    pub pane_keys: Vec<PaneKey>,
    pub internal_index: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub enum TabIndexEventType {
    TabAdded,
    TabKeyUpdated,
    TabRemoved,
}

#[derive(Serialize)]
pub struct InternalIndexEntry {
    pub pane_keys: Vec<PaneKey>,
    pub internal_index: usize,
}

#[derive(Default)]
pub struct TabIndexTracker {
    internal_index_map: HashMap<Vec<PaneKey>, usize>,
    next_internal_index: usize,
    event_history: VecDeque<TabIndexEvent>,
    event_seq: u64,
    removed_keys: HashSet<Vec<PaneKey>>,
}

impl TabIndexTracker {
    pub fn update_for_tab_update(
        &mut self,
        tab_infos: &[zellij_tile::prelude::TabInfo],
        tab_panes: &HashMap<usize, Vec<PaneKey>>,
    ) {
        let current_tab_panes: Vec<Vec<PaneKey>> = tab_infos
            .iter()
            .filter_map(|tab| tab_panes.get(&tab.position).cloned())
            .collect();

        let newly_removed: Vec<(Vec<PaneKey>, usize)> = self
            .internal_index_map
            .iter()
            .filter(|(old_keys, _)| {
                !self.removed_keys.contains(*old_keys)
                    && !current_tab_panes
                        .iter()
                        .any(|current| current.iter().any(|key| old_keys.contains(key)))
            })
            .map(|(keys, &idx)| (keys.clone(), idx))
            .collect();

        for (pane_keys, internal_index) in newly_removed {
            self.record_event(
                TabIndexEventType::TabRemoved,
                pane_keys.clone(),
                internal_index,
            );
            self.removed_keys.insert(pane_keys);
        }

        self.update_common(tab_infos, tab_panes);
    }

    pub fn update_for_pane_update(
        &mut self,
        tab_infos: &[zellij_tile::prelude::TabInfo],
        tab_panes: &HashMap<usize, Vec<PaneKey>>,
    ) {
        self.update_common(tab_infos, tab_panes);
    }

    fn update_common(
        &mut self,
        tab_infos: &[zellij_tile::prelude::TabInfo],
        tab_panes: &HashMap<usize, Vec<PaneKey>>,
    ) {
        for tab in tab_infos {
            let Some(pane_keys) = tab_panes.get(&tab.position) else {
                continue;
            };
            if pane_keys.is_empty() {
                continue;
            }

            let existing = self
                .internal_index_map
                .iter()
                .find(|(old_keys, _)| old_keys.iter().any(|key| pane_keys.contains(key)));

            if let Some((old_keys, &internal_index)) = existing {
                if old_keys != pane_keys {
                    self.internal_index_map
                        .insert(pane_keys.clone(), internal_index);
                    self.record_event(
                        TabIndexEventType::TabKeyUpdated,
                        pane_keys.clone(),
                        internal_index,
                    );
                }
            } else if !self.internal_index_map.contains_key(pane_keys) {
                let internal_index = self.next_internal_index;
                self.next_internal_index += 1;
                self.internal_index_map
                    .insert(pane_keys.clone(), internal_index);
                self.record_event(
                    TabIndexEventType::TabAdded,
                    pane_keys.clone(),
                    internal_index,
                );
            }
        }
    }

    fn record_event(
        &mut self,
        event_type: TabIndexEventType,
        pane_keys: Vec<PaneKey>,
        internal_index: usize,
    ) {
        let event = TabIndexEvent {
            seq: self.event_seq,
            event_type,
            pane_keys,
            internal_index,
        };
        self.event_seq += 1;

        if self.event_history.len() >= MAX_EVENT_HISTORY {
            self.event_history.pop_front();
        }
        self.event_history.push_back(event);
    }

    pub fn get_event_history(&self) -> Vec<TabIndexEvent> {
        self.event_history.iter().cloned().collect()
    }

    pub fn get_rename_target(
        &self,
        tab_panes: &HashMap<usize, Vec<PaneKey>>,
        tab_index: usize,
    ) -> Option<u32> {
        let pane_keys = tab_panes.get(&tab_index)?;
        let internal_index = self
            .internal_index_map
            .iter()
            .find(|(old_keys, _)| old_keys.iter().any(|key| pane_keys.contains(key)))
            .map(|(_, &idx)| idx)?;
        Some((internal_index + 1) as u32)
    }

    pub fn get_debug_entries(&self) -> Vec<InternalIndexEntry> {
        self.internal_index_map
            .iter()
            .map(|(pane_keys, &internal_index)| InternalIndexEntry {
                pane_keys: pane_keys.clone(),
                internal_index,
            })
            .collect()
    }
}
