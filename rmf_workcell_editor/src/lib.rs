use bevy::{log::LogPlugin, pbr::DirectionalLightShadowMap, prelude::*};
#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
use main_menu::MainMenuPlugin;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod keyboard;
use keyboard::*;

pub mod widgets;
use widgets::*;

pub mod demo_world;

pub mod main_menu;
pub mod workcell;
use workcell::*;
pub mod interaction;

pub mod workspace;
use workspace::*;

mod shapes;

pub mod view_menu;
use view_menu::*;

// Parts of the site editor that we reexport since they are needed to build workcell editor apps
pub use librmf_site_editor::{
    aabb::AabbUpdatePlugin,
    animate::AnimationPlugin,
    asset_loaders::AssetLoadersPlugin,
    bevy_egui,
    bevy_mod_raycast,
    log::LogHistoryPlugin,
    // Misc components
    site::{
        AnchorBundle, CollisionMeshMarker, DefaultFile, Delete, Dependents, ModelLoader,
        ModelLoadingResult, PreventDeletion, VisualMeshMarker,
    },
    site::{
        Change, ChangePlugin, Recall, RecallAssetSource, RecallPlugin, RecallPrimitiveShape,
        SiteAssets,
    },
    site::{DeletionPlugin, FuelPlugin, ModelLoadingPlugin},
    site_asset_io,
    wireframe::SiteWireframePlugin,
    // Workspace related objects that are shared with site editor, the rest are reimplemented since
    // the functionality differs significantly
    workspace::{
        ChangeCurrentWorkspace, CreateNewWorkspace, CurrentWorkspace, FileDialogFilter,
        FileDialogServices, RecallWorkspace,
    },
    Autoload,
};

use crate::interaction::InteractionPlugin;

use bevy::render::{
    render_resource::{AddressMode, SamplerDescriptor},
    settings::{WgpuFeatures, WgpuSettings},
    RenderPlugin,
};

use rmf_workcell_format::{
    AssetSource, NameInWorkcell, NameOfWorkcell, Pose, PrimitiveShape, Scale,
};

#[cfg_attr(not(target_arch = "wasm32"), derive(Parser))]
pub struct CommandLineArgs {
    /// Filename of a Site (.site.ron) or Building (.building.yaml) file to load.
    /// Exclude this argument to get the main menu.
    pub filename: Option<String>,
    /// Name of a Site (.site.ron) file to import on top of the base FILENAME.
    #[cfg_attr(not(target_arch = "wasm32"), arg(short, long))]
    pub import: Option<String>,
}

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    WorkcellEditor,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_js() {
    extern crate console_error_panic_hook;
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(vec!["web".to_owned()]);
}

pub fn run(command_line_args: Vec<String>) {
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let command_line_args = CommandLineArgs::parse_from(command_line_args);
        if let Some(path) = command_line_args.filename {
            app.insert_resource(Autoload::file(
                path.into(),
                command_line_args.import.map(Into::into),
            ));
        }
    }

    app.add_plugins(WorkcellEditor::default());
    app.run();
}

#[derive(Default)]
pub struct WorkcellEditor {}

impl Plugin for WorkcellEditor {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            site_asset_io::SiteAssetIoPlugin,
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RMF Workcell Editor".to_owned(),
                        #[cfg(not(target_arch = "wasm32"))]
                        resolution: (1600., 900.).into(),
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some(String::from("#rmf_site_editor_canvas")),
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        address_mode_u: AddressMode::Repeat,
                        address_mode_v: AddressMode::Repeat,
                        address_mode_w: AddressMode::Repeat,
                        ..Default::default()
                    }
                    .into(),
                })
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        #[cfg(not(target_arch = "wasm32"))]
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }
                    .into(),
                }),
        ));

        app.insert_resource(DirectionalLightShadowMap { size: 2048 })
            .init_resource::<SiteAssets>()
            .add_plugins((
                ChangePlugin::<NameInWorkcell>::default(),
                ChangePlugin::<NameOfWorkcell>::default(),
                ChangePlugin::<Pose>::default(),
                ChangePlugin::<Scale>::default(),
                ChangePlugin::<AssetSource>::default(),
                RecallPlugin::<RecallAssetSource>::default(),
                ChangePlugin::<PrimitiveShape>::default(),
                RecallPlugin::<RecallPrimitiveShape>::default(),
            ))
            .add_state::<AppState>()
            .add_plugins((
                ModelLoadingPlugin::default(),
                FuelPlugin::default(),
                DeletionPlugin,
            ))
            .add_plugins((
                AssetLoadersPlugin,
                LogHistoryPlugin,
                AabbUpdatePlugin,
                bevy_egui::EguiPlugin,
                KeyboardInputPlugin,
                InteractionPlugin,
                AnimationPlugin,
                WorkspacePlugin,
                bevy_impulse::ImpulsePlugin::default(),
                StandardUiPlugin::default(),
                MainMenuPlugin,
                WorkcellEditorPlugin,
            ))
            // Note order matters, plugins that edit the menus must be initialized after the UI
            .add_plugins((ViewMenuPlugin, SiteWireframePlugin));

        // Ref https://github.com/bevyengine/bevy/issues/10877. The default behavior causes issues
        // with events being accumulated when not read (i.e. scrolling mouse wheel on a UI widget).
        app.world
            .remove_resource::<bevy::ecs::event::EventUpdateSignal>();
    }
}
