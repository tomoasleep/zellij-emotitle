mod command;
mod state;

use std::collections::BTreeMap;

use command::{parse_args, Command, Mode, Target};
use state::{title_with_emojis, EmotitleState, PaneRef};
use zellij_tile::prelude::*;

register_plugin!(PluginState);

#[derive(Default)]
struct PluginState {
    state: EmotitleState,
}

impl ZellijPlugin for PluginState {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::PaneUpdate, EventType::TabUpdate]);
        set_selectable(false);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                let restores = self.state.update_pane_manifest(pane_manifest);
                for (pane_ref, original_title) in restores {
                    rename_pane(&pane_ref, original_title);
                }
            }
            Event::TabUpdate(tab_infos) => {
                let restores = self.state.update_tab_infos(tab_infos);
                for (tab_index, original_title) in restores {
                    rename_tab((tab_index + 1) as u32, original_title);
                }
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if pipe_message.name != "emotitle" {
            return false;
        }

        let args = pipe_message.args;
        match parse_args(&args) {
            Ok(command) => match self.handle_command(command) {
                Ok(()) => cli_pipe_output(&pipe_message.name, "ok"),
                Err(err) => cli_pipe_output(&pipe_message.name, &err),
            },
            Err(err) => {
                cli_pipe_output(&pipe_message.name, &err);
            }
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl PluginState {
    fn handle_command(&mut self, command: Command) -> Result<(), String> {
        match command.target {
            Target::Pane { pane_id } => {
                let pane_ref = match pane_id {
                    Some(id) => PaneRef::Terminal(id),
                    None => self.state.focused_pane_ref().ok_or_else(|| {
                        "could not resolve focused pane; ensure plugin received PaneUpdate"
                            .to_string()
                    })?,
                };
                self.apply_pane(pane_ref, command.emojis, command.mode)
            }
            Target::Tab { tab_index, pane_id } => {
                let tab_index = if let Some(tab_index) = tab_index {
                    tab_index
                } else if let Some(pane_id) = pane_id {
                    self.state.resolve_tab_index_from_pane_id(pane_id).ok_or_else(|| {
                        format!(
                            "could not resolve tab_index from pane_id={pane_id}; ensure plugin received PaneUpdate"
                        )
                    })?
                } else {
                    self.state.focused_tab_index().ok_or_else(|| {
                        "could not resolve focused tab; ensure plugin received TabUpdate"
                            .to_string()
                    })?
                };
                self.apply_tab(tab_index, command.emojis, command.mode)
            }
        }
    }

    fn apply_pane(&mut self, pane_ref: PaneRef, emojis: String, mode: Mode) -> Result<(), String> {
        let original_title = self
            .state
            .pane_title(&pane_ref)
            .or_else(|| self.state.pane_original_title(&pane_ref))
            .ok_or_else(|| {
                "could not find pane title; ensure plugin is loaded and received PaneUpdate"
                    .to_string()
            })?;

        let new_title = title_with_emojis(&original_title, &emojis);
        rename_pane(&pane_ref, new_title);
        self.state
            .upsert_pane_entry(pane_ref, original_title, emojis, mode);
        Ok(())
    }

    fn apply_tab(&mut self, tab_index: usize, emojis: String, mode: Mode) -> Result<(), String> {
        let original_title = self
            .state
            .tab_title(tab_index)
            .or_else(|| self.state.tab_original_title(tab_index))
            .ok_or_else(|| {
                format!(
                    "could not find tab title for tab_index={tab_index}; ensure plugin received TabUpdate"
                )
            })?;

        let new_title = title_with_emojis(&original_title, &emojis);
        rename_tab((tab_index + 1) as u32, new_title);
        self.state
            .upsert_tab_entry(tab_index, original_title, emojis, mode);
        Ok(())
    }
}

fn rename_pane(pane_ref: &PaneRef, title: String) {
    match pane_ref {
        PaneRef::Terminal(id) => rename_terminal_pane(*id, title),
        PaneRef::Plugin(id) => rename_plugin_pane(*id, title),
    }
}
