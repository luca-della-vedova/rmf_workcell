/*
 * Copyright (C) 2023 Open Source Robotics Foundation
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

use crate::{WorkspaceLoader, WorkspaceSaver};
use bevy::prelude::*;
// TODO(luca) clean up these by reexporting them
use librmf_site_editor::widgets::menu_bar::{FileMenu, MenuEvent, MenuItem, TextMenuItem};
use librmf_site_editor::workspace::CreateNewWorkspace;

/// Keeps track of which entity is associated to the export urdf button.
#[derive(Resource)]
pub struct WorkcellFileMenu {
    new: Entity,
    save: Entity,
    save_as: Entity,
    load: Entity,
    export_urdf: Entity,
}

impl FromWorld for WorkcellFileMenu {
    fn from_world(world: &mut World) -> Self {
        let file_header = world.resource::<FileMenu>().get();
        let new = world
            .spawn(MenuItem::Text(TextMenuItem::new("New").shortcut("Ctrl-N")))
            .set_parent(file_header)
            .id();
        let save = world
            .spawn(MenuItem::Text(TextMenuItem::new("Save").shortcut("Ctrl-S")))
            .set_parent(file_header)
            .id();
        let save_as = world
            .spawn(MenuItem::Text(
                TextMenuItem::new("Save As").shortcut("Ctrl-Shift-S"),
            ))
            .set_parent(file_header)
            .id();
        let load = world
            .spawn(MenuItem::Text(TextMenuItem::new("Open").shortcut("Ctrl-O")))
            .set_parent(file_header)
            .id();
        let export_urdf = world
            .spawn(MenuItem::Text(
                TextMenuItem::new("Export Urdf").shortcut("Ctrl-E"),
            ))
            .set_parent(file_header)
            .id();

        WorkcellFileMenu {
            new,
            save,
            save_as,
            load,
            export_urdf,
        }
    }
}

pub fn handle_export_urdf_menu_events(
    mut menu_events: EventReader<MenuEvent>,
    file_menu: Res<WorkcellFileMenu>,
    mut workspace_saver: WorkspaceSaver,
    mut workspace_loader: WorkspaceLoader,
    mut new_workspace: EventWriter<CreateNewWorkspace>,
) {
    for event in menu_events.read() {
        if event.clicked() && event.source() == file_menu.new {
            new_workspace.send(CreateNewWorkspace);
        } else if event.clicked() && event.source() == file_menu.save {
            workspace_saver.save_to_default_file();
        } else if event.clicked() && event.source() == file_menu.save_as {
            workspace_saver.save_to_dialog();
        } else if event.clicked() && event.source() == file_menu.load {
            workspace_loader.load_from_dialog();
        } else if event.clicked() && event.source() == file_menu.export_urdf {
            workspace_saver.export_urdf_to_dialog();
        }
    }
}
