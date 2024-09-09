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

use bevy::{ecs::system::SystemParam, prelude::*, tasks::AsyncComputeTaskPool};
use bevy_impulse::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;

use crate::workcell::{LoadWorkcell, SaveWorkcell};
use crate::AppState;
use librmf_site_editor::interaction::InteractionState;
use librmf_site_editor::site::DefaultFile;
use rmf_workcell_format::Workcell;

use librmf_site_editor::workspace::{
    ChangeCurrentWorkspace, CreateNewWorkspace, CurrentWorkspace, FileDialogServices,
    WorkspaceMarker,
};

use crossbeam_channel::{Receiver, Sender};

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

#[derive(Event)]
pub struct SaveWorkspace {
    /// If specified workspace will be saved to requested file, otherwise the default file
    pub destination: SaveWorkspaceDestination,
    /// If specified the workspace will be exported to a specific format
    pub format: ExportFormat,
}

impl SaveWorkspace {
    pub fn new() -> Self {
        Self {
            destination: SaveWorkspaceDestination::default(),
            format: ExportFormat::default(),
        }
    }

    pub fn to_default_file(mut self) -> Self {
        self.destination = SaveWorkspaceDestination::DefaultFile;
        self
    }

    pub fn to_dialog(mut self) -> Self {
        self.destination = SaveWorkspaceDestination::Dialog;
        self
    }

    pub fn to_path(mut self, path: &PathBuf) -> Self {
        self.destination = SaveWorkspaceDestination::Path(path.clone());
        self
    }

    pub fn to_urdf(mut self) -> Self {
        self.format = ExportFormat::Urdf;
        self
    }
}

#[derive(Default, Debug, Clone)]
pub enum SaveWorkspaceDestination {
    #[default]
    DefaultFile,
    Dialog,
    Path(PathBuf),
}

#[derive(Clone, Default, Debug)]
pub enum ExportFormat {
    #[default]
    Default,
    Urdf,
}

/// Event used in channels to communicate the file handle that was chosen by the user.
#[derive(Debug)]
pub struct SaveWorkspaceFile {
    path: PathBuf,
    format: ExportFormat,
    root: Entity,
}

/// Use a channel since file dialogs are async and channel senders can be cloned and passed into an
/// async block.
#[derive(Debug, Resource)]
pub struct SaveWorkspaceChannels {
    pub sender: Sender<SaveWorkspaceFile>,
    pub receiver: Receiver<SaveWorkspaceFile>,
}

impl Default for SaveWorkspaceChannels {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Self { sender, receiver }
    }
}

/// Used to keep track of visibility when switching workspace
#[derive(Debug, Default, Resource)]
pub struct RecallWorkspace(Option<Entity>);

pub struct WorkspacePlugin;

impl Plugin for WorkspacePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChangeCurrentWorkspace>()
            .add_event::<CreateNewWorkspace>()
            .add_event::<SaveWorkspace>()
            .add_event::<SaveWorkcell>()
            .add_event::<LoadWorkcell>()
            .init_resource::<CurrentWorkspace>()
            .init_resource::<RecallWorkspace>()
            .init_resource::<SaveWorkspaceChannels>()
            .init_resource::<FileDialogServices>()
            .init_resource::<WorkspaceLoadingServices>()
            .add_systems(
                Update,
                (
                    dispatch_new_workspace_events,
                    sync_workspace_visibility,
                    workspace_file_save_complete,
                ),
            );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, dispatch_save_workspace_events);
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
                    println!("Setting state to workcell editor");
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
                    // TODO(luca) make this function return a result and this a match statement
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
    /// Service that spawns a save file dialog then creates a site with an empty level.
    // pub create_empty_workspace_from_dialog: Service<(), ()>,
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
            .pick_file_for_loading
            .clone();
        // Spawn all the services
        let load_workspace_from_dialog = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .then(pick_file)
                .map_async(|(path, data)| async move {
                    let data = WorkspaceData::new(&path, data)?;
                    return Some(LoadWorkspaceFile(Some(path), data));
                })
                .cancel_on_none()
                .then(process_load_files)
                .connect(scope.terminate)
        });

        // TODO(luca) consider offering this for workcell editor
        /*
        let create_empty_workspace_from_dialog = world.spawn_workflow(|scope, builder| {
            scope
                .input
                .chain(builder)
                .map_async(|_| async move {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Some(file) = AsyncFileDialog::new().save_file().await {
                            let file = file.path().to_path_buf();
                            let name = file
                                .file_stem()
                                .map(|s| s.to_str().map(|s| s.to_owned()))
                                .flatten()
                                .unwrap_or_else(|| "blank".to_owned());
                            let data = WorkspaceData::LoadSite(LoadSite::blank_L1(
                                name,
                                Some(file.clone()),
                            ));
                            return Some(LoadWorkspaceFile(Some(file), data));
                        }
                        None
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        let data =
                            WorkspaceData::LoadSite(LoadSite::blank_L1("blank".to_owned(), None));
                        Some(LoadWorkspaceFile(None, data))
                    }
                })
                .cancel_on_none()
                .then(process_load_files)
                .connect(scope.terminate)
        });
        */

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
            // create_empty_workspace_from_dialog,
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

    /// Request to spawn a dialog to select a file and create a new site with a blank level
    pub fn create_empty_from_dialog(&mut self) {
        /*
        self.commands
            .request(
                (),
                self.workspace_loading.create_empty_workspace_from_dialog,
            )
            .detach();
        */
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

// TODO(luca) implement this in wasm, it's supported since rfd version 0.12, however it requires
// calling .write on the `FileHandle` object returned by the AsyncFileDialog. Such FileHandle is
// not Send in wasm so it can't be sent to another thread through an event. We would need to
// refactor saving to be fully done in the async task rather than send an event to have wasm saving.
#[cfg(not(target_arch = "wasm32"))]
fn dispatch_save_workspace_events(
    mut save_events: EventReader<SaveWorkspace>,
    save_channels: Res<SaveWorkspaceChannels>,
    workspace: Res<CurrentWorkspace>,
    default_files: Query<&DefaultFile>,
) {
    let spawn_dialog = |format: &ExportFormat, root| {
        let sender = save_channels.sender.clone();
        let format = format.clone();
        AsyncComputeTaskPool::get()
            .spawn(async move {
                if let Some(file) = AsyncFileDialog::new().save_file().await {
                    let path = file.path().to_path_buf();
                    sender
                        .send(SaveWorkspaceFile { path, format, root })
                        .expect("Failed sending save event");
                }
            })
            .detach();
    };
    for event in save_events.read() {
        if let Some(ws_root) = workspace.root {
            match &event.destination {
                SaveWorkspaceDestination::DefaultFile => {
                    if let Some(file) = default_files.get(ws_root).ok().map(|f| f.0.clone()) {
                        save_channels
                            .sender
                            .send(SaveWorkspaceFile {
                                path: file,
                                format: event.format.clone(),
                                root: ws_root,
                            })
                            .expect("Failed sending save request");
                    } else {
                        spawn_dialog(&event.format, ws_root);
                    }
                }
                SaveWorkspaceDestination::Dialog => spawn_dialog(&event.format, ws_root),
                SaveWorkspaceDestination::Path(path) => {
                    save_channels
                        .sender
                        .send(SaveWorkspaceFile {
                            path: path.clone(),
                            format: event.format.clone(),
                            root: ws_root,
                        })
                        .expect("Failed sending save request");
                }
            }
        } else {
            warn!("Unable to save, no workspace loaded");
            return;
        }
    }
}

/// Handles the file saving events
fn workspace_file_save_complete(
    app_state: Res<State<AppState>>,
    mut save_workcell: EventWriter<SaveWorkcell>,
    save_channels: Res<SaveWorkspaceChannels>,
) {
    if let Ok(result) = save_channels.receiver.try_recv() {
        match app_state.get() {
            AppState::WorkcellEditor => {
                save_workcell.send(SaveWorkcell {
                    root: result.root,
                    to_file: result.path,
                    format: result.format,
                });
            }
            AppState::MainMenu => { /* Noop */ }
        }
    }
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
