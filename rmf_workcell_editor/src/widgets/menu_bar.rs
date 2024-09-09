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

use crate::{SaveWorkspace, WorkspaceLoader};
use librmf_site_editor::{
    widgets::prelude::*,
    widgets::{
        render_sub_menu, FileMenu, Menu, MenuDisabled, MenuEvent, MenuItem, ToolMenu, ViewMenu,
    },
    workspace::CreateNewWorkspace,
};

use bevy::ecs::query::Has;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui::{self, Button, Ui};

// TODO(luca) we only need this for custom workspace events, make them modular
/// Add the standard menu bar to the application.
#[derive(Default)]
pub struct MenuBarPlugin {}

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        let widget = PanelWidget::new(top_menu_bar, &mut app.world);
        app.world.spawn(widget);

        app.add_event::<MenuEvent>()
            .init_resource::<FileMenu>()
            .init_resource::<ToolMenu>()
            .init_resource::<ViewMenu>();
    }
}

#[derive(SystemParam)]
struct MenuParams<'w, 's> {
    menus: Query<'w, 's, (&'static Menu, Entity)>,
    menu_items: Query<'w, 's, (&'static mut MenuItem, Has<MenuDisabled>)>,
    extension_events: EventWriter<'w, MenuEvent>,
    view_menu: Res<'w, ViewMenu>,
}

fn top_menu_bar(
    In(input): In<PanelWidgetInput>,
    mut new_workspace: EventWriter<CreateNewWorkspace>,
    mut save: EventWriter<SaveWorkspace>,
    mut workspace_loader: WorkspaceLoader,
    file_menu: Res<FileMenu>,
    top_level_components: Query<(), Without<Parent>>,
    children: Query<&Children>,
    mut menu_params: MenuParams,
) {
    egui::TopBottomPanel::top("top_panel").show(&input.context, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.add(Button::new("New").shortcut_text("Ctrl+N")).clicked() {
                    new_workspace.send(CreateNewWorkspace);
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui
                        .add(Button::new("Save").shortcut_text("Ctrl+S"))
                        .clicked()
                    {
                        save.send(SaveWorkspace::new().to_default_file());
                    }
                    if ui
                        .add(Button::new("Save As").shortcut_text("Ctrl+Shift+S"))
                        .clicked()
                    {
                        save.send(SaveWorkspace::new().to_dialog());
                    }
                }
                if ui
                    .add(Button::new("Open").shortcut_text("Ctrl+O"))
                    .clicked()
                {
                    workspace_loader.load_from_dialog();
                }

                render_sub_menu(
                    ui,
                    &file_menu.get(),
                    &children,
                    &menu_params.menus,
                    &menu_params.menu_items,
                    &mut menu_params.extension_events,
                    true,
                );
            });
            ui.menu_button("View", |ui| {
                render_sub_menu(
                    ui,
                    &menu_params.view_menu.get(),
                    &children,
                    &menu_params.menus,
                    &menu_params.menu_items,
                    &mut menu_params.extension_events,
                    true,
                );
            });

            for (_, entity) in menu_params.menus.iter().filter(|(_, entity)| {
                top_level_components.contains(*entity)
                    && (*entity != file_menu.get() && *entity != menu_params.view_menu.get())
            }) {
                render_sub_menu(
                    ui,
                    &entity,
                    &children,
                    &menu_params.menus,
                    &menu_params.menu_items,
                    &mut menu_params.extension_events,
                    false,
                );
            }
        });
    });
}
