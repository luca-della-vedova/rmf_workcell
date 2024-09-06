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

//! The site editor allows you to insert your own egui widgets into the UI.
//! Simple examples of custom widgets can be found in the docs for
//! [`PropertiesTilePlugin`] and [`InspectionPlugin`].
//!
//! There are three categories of widgets that the site editor provides
//! out-of-the-box support for inserting, but the widget system itself is
//! highly extensible, allowing you to define your own categories of widgets.
//!
//! The three categories provided out of the box include:
//! - [Panel widget][1]: Add a new panel to the UI.
//! - Tile widget: Add a tile into a [panel of tiles][2] such as the [`PropertiesPanel`]. Use [`PropertiesTilePlugin`] to make a new tile widget that goes inside of the standard `PropertiesPanel`.
//! - [`InspectionPlugin`]: Add a widget to the [`MainInspector`] to display more information about the currently selected entity.
//!
//! In our terminology, there are two kinds of panels:
//! - Side panels: A vertical column widget on the left or right side of the screen.
//!   - [`PropertiesPanel`] is usually a side panel placed on the right side of the screen.
//!   - [`FuelAssetBrowser`] is a side panel typically placed on the left side of the screen.
//!   - [`Diagnostics`] is a side panel that interactively flags issues that have been found in the site.
//! - Top / Bottom Panels:
//!   - The [`MenuBarPlugin`] provides a menu bar at the top of the screen.
//!     - Create an entity with a [`Menu`] component to create a new menu inside the menu bar.
//!     - Add an entity with a [`MenuItem`] component as a child to a menu entity to add a new item into a menu.
//!     - The [`FileMenu`], [`ToolMenu`], and [`ViewMenu`] are resources that provide access to various standard menus.
//!   - The [`ConsoleWidgetPlugin`] provides a console at the bottom of the screen to display information, warning, and error messages.
//!
//! [1]: crate::widgets::PanelWidget
//! [2]: crate::widgets::show_panel_of_tiles

use librmf_site_editor::interaction::{Hover, PickingBlockers};
use crate::AppState;
use bevy::{
    ecs::{
        system::{SystemParam, SystemState},
        world::EntityWorldMut,
    },
    prelude::*,
};
use bevy_egui::{
    egui::{self, Ui},
    EguiContexts,
};

/*
pub mod creation;
use creation::*;

pub mod fuel_asset_browser;
pub use fuel_asset_browser::*;

pub mod inspector;
pub use inspector::*;

pub mod menu_bar;
pub use menu_bar::*;

use librmf_site_editor::widgets::prelude::*;

/// This plugin provides the standard UI layout that was designed for the common
/// use cases of the site editor.
#[derive(Default)]
pub struct StandardUiPlugin {}

impl Plugin for StandardUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasTooltips>()
            .add_plugins((
                IconsPlugin::default(),
                MenuBarPlugin::default(),
                StandardPropertiesPanelPlugin::default(),
                //FuelAssetBrowserPlugin::default(),
                ConsoleWidgetPlugin::default(),
            ))
            .add_systems(Startup, init_ui_style)
            .add_systems(
                Update,
                workcell_ui_layout
                    .in_set(RenderUiSet)
                    .run_if(in_state(AppState::WorkcellEditor)),
            );
    }
}

/// This set is for systems that impact rendering the UI using egui. The
/// [`UserCameraDisplay`] resource waits until after this set is finished before
/// computing the user camera area.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct RenderUiSet;

/// This system renders all UI panels in the application and makes sure that the
/// UI rendering works correctly with the picking system, and any other systems
/// as needed.
pub fn workcell_ui_layout(
    world: &mut World,
    panel_widgets: &mut QueryState<(Entity, &mut PanelWidget)>,
    egui_context_state: &mut SystemState<EguiContexts>,
) {
    render_panels(world, panel_widgets, egui_context_state);

    let mut egui_context = egui_context_state.get_mut(world);
    let mut ctx = egui_context.ctx_mut().clone();
    let ui_has_focus =
        ctx.wants_pointer_input() || ctx.wants_keyboard_input() || ctx.is_pointer_over_area();

    if let Some(mut picking_blocker) = world.get_resource_mut::<PickingBlockers>() {
        picking_blocker.ui = ui_has_focus;
    }

    if ui_has_focus {
        // If the UI has focus and there were no hover events emitted by the UI,
        // then we should emit a None hover event
        let mut hover = world.resource_mut::<Events<Hover>>();
        if hover.is_empty() {
            hover.send(Hover(None));
        }
    } else {
        // If the UI does not have focus then render the CanvasTooltips.
        world.resource_mut::<CanvasTooltips>().render(&mut ctx);
    }
}

fn init_ui_style(mut egui_context: EguiContexts) {
    // I think the default egui dark mode text color is too dim, so this changes
    // it to a brighter white.
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(250, 250, 250));
    egui_context.ctx_mut().set_visuals(visuals);
}
*/
