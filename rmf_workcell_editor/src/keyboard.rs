/*
 * Copyright (C) 2022 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

use crate::{interaction::Selection, CreateNewWorkspace, Delete};

pub use librmf_site_editor::keyboard::{keyboard_just_pressed_stream, KeyboardServices};

use crate::workspace::{WorkspaceLoader, WorkspaceSaver};
use bevy::{
    prelude::{Input as UserInput, *},
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;
use bevy_impulse::*;

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, handle_keyboard_input);

        let keyboard_just_pressed =
            app.spawn_continuous_service(Last, keyboard_just_pressed_stream);

        app.insert_resource(KeyboardServices {
            keyboard_just_pressed,
        });
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<UserInput<KeyCode>>,
    selection: Res<Selection>,
    mut egui_context: EguiContexts,
    mut delete: EventWriter<Delete>,
    mut new_workspace: EventWriter<CreateNewWorkspace>,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    mut workspace_loader: WorkspaceLoader,
    mut workspace_saver: WorkspaceSaver,
) {
    let Some(egui_context) = primary_windows
        .get_single()
        .ok()
        .and_then(|w| egui_context.try_ctx_for_window_mut(w))
    else {
        return;
    };
    let ui_has_focus = egui_context.wants_pointer_input()
        || egui_context.wants_keyboard_input()
        || egui_context.is_pointer_over_area();

    if ui_has_focus {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Delete) || keyboard_input.just_pressed(KeyCode::Back) {
        if let Some(selection) = selection.0 {
            delete.send(Delete::new(selection));
        } else {
            warn!("No selected entity to delete");
        }
    }

    // Ctrl keybindings
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard_input.just_pressed(KeyCode::S) {
            if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                workspace_saver.save_to_dialog();
            } else {
                workspace_saver.save_to_default_file();
            }
        }

        if keyboard_input.just_pressed(KeyCode::E) {
            workspace_saver.export_urdf_to_dialog();
        }

        // TODO(luca) pop up a confirmation prompt if the current file is not saved, or create a
        // gui to switch between open workspaces
        if keyboard_input.just_pressed(KeyCode::N) {
            new_workspace.send(CreateNewWorkspace);
        }

        if keyboard_input.just_pressed(KeyCode::O) {
            workspace_loader.load_from_dialog();
        }
    }
}
