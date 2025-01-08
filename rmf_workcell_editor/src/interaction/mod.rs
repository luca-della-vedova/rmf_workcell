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

pub use librmf_site_editor::interaction::InteractionPlugin as SiteInteractionPlugin;
// Reexported types used for widgets / plugins
pub use librmf_site_editor::interaction::{
    aligned_z_axis, extract_selector_input, hover_service, print_if_err, set_visibility,
    AnchorVisualization, CategoryVisibility, CategoryVisibilityPlugin, CommonNodeErrors, Cursor,
    DragPlaneBundle, GizmoBlockers, HighlightAnchors, Hover, Hovering, InspectorFilter,
    InspectorService, InteractionAssets, InteractionState, IntersectGroundPlaneParams,
    PickingBlockers, Preview, RunSelector, Select, Selectable, Selection, SelectionFilter,
    SelectionNodeResult, SelectionServiceStages, SelectorInput, SetCategoryVisibility,
    SiteRaycastSet, VisualCue,
};

use crate::WorkcellVisualizationMarker;

pub mod select;
pub use select::*;

use bevy::prelude::*;

#[derive(Default)]
pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SiteInteractionPlugin::default(),
            CategoryVisibilityPlugin::<WorkcellVisualizationMarker>::visible(true),
            place_object::ObjectPlacementPlugin::default(),
        ));
    }
}
