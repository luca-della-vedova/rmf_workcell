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

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_impulse::*;
use std::path::PathBuf;

use crate::workcell::{LoadWorkcell, SaveWorkcell};
use crate::AppState;
use rmf_workcell_format::Workcell;

use crate::{
    interaction::InteractionState, ChangeCurrentWorkspace, CreateNewWorkspace, CurrentWorkspace,
    DefaultFile, FileDialogFilter, FileDialogServices, RecallWorkspace,
};

#[derive(Clone)]
pub enum WorkspaceData {
    Workcell(Vec<u8>),
    WorkcellUrdf(Vec<u8>),
}

impl WorkspaceData {
    pub fn new(path: &PathBuf, data: Vec<u8>) -> Option<Self> {
        let filename = path.file_name().and_then(|f| f.to_str())?;
        if filename.ends_with("workcell.json") {
            Some(WorkspaceData::Workcell(data))
        } else if filename.ends_with("urdf") {
            Some(WorkspaceData::WorkcellUrdf(data))
        } else {
            error!("Unrecognized file type {:?}", filename);
            None
        }
    }
}

pub struct LoadWorkspaceFile(pub Option<PathBuf>, pub WorkspaceData);

#[derive(Clone, Default, Debug)]
pub enum ExportFormat {
    #[default]
    Default,
    Urdf,
}

pub struct WorkspacePlugin;

impl Plugin for WorkspacePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangeCurrentWorkspace>()
            .add_event::<CreateNewWorkspace>()
            .add_event::<SaveWorkcell>()
            .add_event::<LoadWorkcell>()
            .init_resource::<CurrentWorkspace>()
            .init_resource::<RecallWorkspace>()
            .init_resource::<FileDialogServices>()
            .init_resource::<WorkspaceLoadingServices>()
            .init_resource::<WorkspaceSavingServices>()
            .add_systems(
                Update,
                (dispatch_new_workspace_events, sync_workspace_visibility),
            );
    }
}

pub fn dispatch_new_workspace_events(
    state: Res<State<AppState>>,
    mut new_workspace: EventReader<CreateNewWorkspace>,
    mut load_workcell: EventWriter<LoadWorkcell>,
) {
    if let Some(_cmd) = new_workspace.read().last() {
        match state.get() {
            AppState::MainMenu => {
                error!("Sent generic new workspace while in main menu");
            }
            AppState::WorkcellEditor => {
                println!("Creating new workspace");
                load_workcell.send(LoadWorkcell {
                    workcell: Workcell::default(),
                    focus: true,
                    default_file: None,
                });
            }
        }
    }
}

/// Service that takes workspace data and loads a site / workcell, as well as transition state.
pub fn process_load_workspace_files(
    In(BlockingService { request, .. }): BlockingServiceInput<LoadWorkspaceFile>,
    mut app_state: ResMut<NextState<AppState>>,
    mut interaction_state: ResMut<NextState<InteractionState>>,
    mut load_workcell: EventWriter<LoadWorkcell>,
) {
    let LoadWorkspaceFile(default_file, data) = request;
    match data {
        WorkspaceData::Workcell(data) => {
            info!("Opening workcell file");
            match Workcell::from_bytes(&data) {
                Ok(workcell) => {
                    // Switch state
                    app_state.set(AppState::WorkcellEditor);
                    load_workcell.send(LoadWorkcell {
                        workcell,
                        focus: true,
                        default_file,
                    });
                    interaction_state.set(InteractionState::Enable);
                }
                Err(err) => {
                    error!("Failed loading workcell {:?}", err);
                }
            }
        }
        WorkspaceData::WorkcellUrdf(data) => {
            info!("Importing urdf workcell");
            let Ok(utf) = std::str::from_utf8(&data) else {
                error!("Failed converting urdf bytes to string");
                return;
            };
            match urdf_rs::read_from_string(utf) {
                Ok(urdf) => {
                    match Workcell::from_urdf(&urdf) {
                        Ok(workcell) => {
                            // Switch state
                            app_state.set(AppState::WorkcellEditor);
                            load_workcell.send(LoadWorkcell {
                                workcell,
                                focus: true,
                                default_file,
                            });
                            interaction_state.set(InteractionState::Enable);
                        }
                        Err(err) => {
                            error!("Failed converting urdf to workcell {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("Failed loading urdf workcell {:?}", err);
                }
            }
        }
    }
}

#[derive(Resource)]
/// Services that deal with workspace loading
pub struct WorkspaceLoadingServices {
    /// Service that spawns an open file dialog and loads a site accordingly.
    pub load_workspace_from_dialog: Service<(), ()>,
    /// Loads the workspace at the requested path
    pub load_workspace_from_path: Service<PathBuf, ()>,
    /// Loads the workspace from the requested data
    pub load_workspace_from_data: Service<WorkspaceData, ()>,
}

impl FromWorld for WorkspaceLoadingServices {
    fn from_world(world: &mut World) -> Self {
        let process_load_files = world.spawn_service(process_load_workspace_files);
        let pick_file = world
            .resource::<FileDialogServices>()
            .pick_file_and_load
            .clone();
        // Spawn all the services
        let loading_filters = vec![
            FileDialogFilter {
                name: "Workcell".into(),
                extensions: vec!["workcell.json".into()],
            },
            FileDialogFilter {
                name: "Urdf".into(),
                extensions: vec!["urdf".into()],
            },
        ];
        let load_workspace_from_dialog = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .map_block(move |_| loading_filters.clone())
                .then(pick_file)
                .map_async(|(path, data)| async move {
                    let data = WorkspaceData::new(&path, data)?;
                    return Some(LoadWorkspaceFile(Some(path), data));
                })
                .cancel_on_none()
                .then(process_load_files)
                .connect(scope.terminate)
        });

        let load_workspace_from_path = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .map_async(|path| async move {
                    let Some(data) = std::fs::read(&path)
                        .ok()
                        .and_then(|data| WorkspaceData::new(&path, data))
                    else {
                        warn!("Unable to read file [{path:?}] so it cannot be loaded");
                        return None;
                    };
                    Some(LoadWorkspaceFile(Some(path.clone()), data))
                })
                .cancel_on_none()
                .then(process_load_files)
                .connect(scope.terminate)
        });

        let load_workspace_from_data = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .map_block(|data| LoadWorkspaceFile(None, data))
                .then(process_load_files)
                .connect(scope.terminate)
        });

        Self {
            load_workspace_from_dialog,
            load_workspace_from_path,
            load_workspace_from_data,
        }
    }
}

impl<'w, 's> WorkspaceLoader<'w, 's> {
    /// Request to spawn a dialog and load a workspace
    pub fn load_from_dialog(&mut self) {
        self.commands
            .request((), self.workspace_loading.load_workspace_from_dialog)
            .detach();
    }

    /// Request to load a workspace from a path
    pub fn load_from_path(&mut self, path: PathBuf) {
        self.commands
            .request(path, self.workspace_loading.load_workspace_from_path)
            .detach();
    }

    /// Request to load a workspace from data
    pub fn load_from_data(&mut self, data: WorkspaceData) {
        self.commands
            .request(data, self.workspace_loading.load_workspace_from_data)
            .detach();
    }
}

/// `SystemParam` used to request for workspace loading operations
#[derive(SystemParam)]
pub struct WorkspaceLoader<'w, 's> {
    workspace_loading: Res<'w, WorkspaceLoadingServices>,
    commands: Commands<'w, 's>,
}

/// Handles the file saving events
fn send_file_save(
    In(BlockingService { request, .. }): BlockingServiceInput<(PathBuf, ExportFormat)>,
    app_state: Res<State<AppState>>,
    mut save_workcell: EventWriter<SaveWorkcell>,
    current_workspace: Res<CurrentWorkspace>,
) {
    let Some(ws_root) = current_workspace.root else {
        warn!("Failed saving workspace, no current workspace found");
        return;
    };
    match app_state.get() {
        AppState::WorkcellEditor => {
            save_workcell.send(SaveWorkcell {
                root: ws_root,
                to_file: request.0,
                format: request.1,
            });
        }
        AppState::MainMenu => { /* Noop */ }
    }
}

#[derive(Resource)]
/// Services that deal with workspace loading
pub struct WorkspaceSavingServices {
    /// Service that spawns a save file dialog and saves the current site accordingly.
    pub save_workspace_to_dialog: Service<(), ()>,
    /// Saves the current workspace at the requested path.
    pub save_workspace_to_path: Service<PathBuf, ()>,
    /// Saves the current workspace in the current default file.
    pub save_workspace_to_default_file: Service<(), ()>,
    /// Opens a dialog to pick a folder and exports the requested workspace as a URDF package.
    pub export_urdf_to_dialog: Service<(), ()>,
}

impl FromWorld for WorkspaceSavingServices {
    fn from_world(world: &mut World) -> Self {
        let send_file_save = world.spawn_service(send_file_save);
        let get_default_file = |In(()): In<_>,
                                current_workspace: Res<CurrentWorkspace>,
                                default_files: Query<&DefaultFile>|
         -> Option<PathBuf> {
            let ws_root = current_workspace.root?;
            default_files.get(ws_root).ok().map(|f| f.0.clone())
        };
        let get_default_file = get_default_file.into_blocking_callback();
        let pick_file = world
            .resource::<FileDialogServices>()
            .pick_file_for_saving
            .clone();
        let pick_folder = world.resource::<FileDialogServices>().pick_folder.clone();
        let saving_filters = vec![FileDialogFilter {
            name: "Workcell".into(),
            extensions: vec!["workcell.json".into()],
        }];
        // Spawn all the services
        let save_workspace_to_dialog = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .map_block(move |_| saving_filters.clone())
                .then(pick_file)
                .map_block(|path| (path, ExportFormat::default()))
                .then(send_file_save)
                .connect(scope.terminate)
        });
        let save_workspace_to_path = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .map_block(|path| (path, ExportFormat::default()))
                .then(send_file_save)
                .connect(scope.terminate)
        });
        let save_workspace_to_default_file = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .then(get_default_file)
                .branch_for_none(|chain: Chain<()>| {
                    chain
                        .then(save_workspace_to_dialog)
                        .connect(scope.terminate)
                })
                .then(save_workspace_to_path)
                .connect(scope.terminate)
        });
        let export_urdf_to_dialog = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .then(pick_folder)
                .map_block(|path| (path, ExportFormat::Urdf))
                .then(send_file_save)
                .connect(scope.terminate)
        });

        Self {
            save_workspace_to_dialog,
            save_workspace_to_path,
            save_workspace_to_default_file,
            export_urdf_to_dialog,
        }
    }
}

// TODO(luca) implement saving in wasm, it's supported since rfd version 0.12, however it requires
// calling .write on the `FileHandle` object returned by the AsyncFileDialog. Such FileHandle is
// not Send in wasm so it can't be sent to another thread through an event. We would need to
// refactor saving to be fully done in the async task rather than send an event to have wasm saving.
impl<'w, 's> WorkspaceSaver<'w, 's> {
    /// Request to spawn a dialog and save the workspace
    pub fn save_to_dialog(&mut self) {
        self.commands
            .request((), self.workspace_saving.save_workspace_to_dialog)
            .detach();
    }

    /// Request to save the workspace to the default file (or a dialog if no default file is
    /// available).
    pub fn save_to_default_file(&mut self) {
        self.commands
            .request((), self.workspace_saving.save_workspace_to_default_file)
            .detach();
    }

    /// Request to save the workspace to the requested path
    pub fn save_to_path(&mut self, path: PathBuf) {
        self.commands
            .request(path, self.workspace_saving.save_workspace_to_path)
            .detach();
    }

    /// Request to export the workspace as a urdf to a folder selected from a dialog
    pub fn export_urdf_to_dialog(&mut self) {
        self.commands
            .request((), self.workspace_saving.export_urdf_to_dialog)
            .detach();
    }
}

/// `SystemParam` used to request for workspace loading operations
#[derive(SystemParam)]
pub struct WorkspaceSaver<'w, 's> {
    workspace_saving: Res<'w, WorkspaceSavingServices>,
    commands: Commands<'w, 's>,
}

pub fn sync_workspace_visibility(
    current_workspace: Res<CurrentWorkspace>,
    mut recall: ResMut<RecallWorkspace>,
    mut visibility: Query<&mut Visibility>,
) {
    if !current_workspace.is_changed() {
        return;
    }

    if recall.0 != current_workspace.root {
        // Set visibility of current to target
        if let Some(current_workspace_entity) = current_workspace.root {
            if let Ok(mut v) = visibility.get_mut(current_workspace_entity) {
                *v = if current_workspace.display {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
        // Disable visibility in recall
        if let Some(recall) = recall.0 {
            if let Ok(mut v) = visibility.get_mut(recall) {
                *v = Visibility::Hidden;
            }
        }
        recall.0 = current_workspace.root;
    }
}
