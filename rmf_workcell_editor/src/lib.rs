use bevy::{log::LogPlugin, pbr::DirectionalLightShadowMap, prelude::*};
use bevy_egui::EguiPlugin;
use main_menu::MainMenuPlugin;
// use warehouse_generator::WarehouseGeneratorPlugin;
#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod keyboard;
use keyboard::*;

pub mod widgets;
use widgets::*;

pub mod demo_world;

pub mod main_menu;
pub mod workcell;
use workcell::WorkcellEditorPlugin;
pub mod interaction;

pub mod workspace;
use workspace::*;

mod shapes;

// TODO(luca) consider using children for this rather than rewriting
pub mod view_menu;
use view_menu::*;

use librmf_site_editor::{
    aabb::AabbUpdatePlugin,
    animate::AnimationPlugin,
    asset_loaders::AssetLoadersPlugin,
    interaction::CategoryVisibilityPlugin,
    log::LogHistoryPlugin,
    site::{ChangePlugin, RecallAssetSource, RecallPlugin, RecallPrimitiveShape, SiteAssets},
    site::{CurrentEditDrawing, CurrentLevel, ToggleLiftDoorAvailability},
    site::{DeletionPlugin, FuelPlugin, ModelLoadingPlugin},
    site_asset_io::SiteAssetIoPlugin,
    widgets::UserCameraDisplayPlugin,
    wireframe::SiteWireframePlugin,
    Autoload,
};

use crate::workcell::WorkcellVisualizationMarker;

use crate::interaction::WorkcellInteractionPlugin;

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
        // TODO(luca) clean this
        let mut plugins = DefaultPlugins.build();
        plugins = plugins.set(WindowPlugin {
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
        });
        app.add_plugins((
            SiteAssetIoPlugin,
            plugins
                .disable::<LogPlugin>()
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
                    ..default()
                }),
        ));

        app.insert_resource(DirectionalLightShadowMap { size: 2048 })
            .init_resource::<SiteAssets>()
            // TODO(luca) remove the need to add all of these for interaction plugin to work
            .add_plugins(UserCameraDisplayPlugin::default())
            .init_resource::<CurrentEditDrawing>()
            .init_resource::<CurrentLevel>()
            .add_event::<ToggleLiftDoorAvailability>()
            .add_plugins((
                ChangePlugin::<NameInWorkcell>::default(),
                ChangePlugin::<NameOfWorkcell>::default(),
                ChangePlugin::<Pose>::default(),
                ChangePlugin::<Scale>::default(),
                ChangePlugin::<AssetSource>::default(),
                RecallPlugin::<RecallAssetSource>::default(),
                ChangePlugin::<PrimitiveShape>::default(),
                RecallPlugin::<RecallPrimitiveShape>::default(),
                CategoryVisibilityPlugin::<WorkcellVisualizationMarker>::visible(true),
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
                EguiPlugin,
                KeyboardInputPlugin,
                WorkcellInteractionPlugin,
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
