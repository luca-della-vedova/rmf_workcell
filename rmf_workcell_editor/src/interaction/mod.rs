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

use librmf_site_editor::site::{
    update_anchor_transforms, CollisionMeshMarker,
    SiteUpdateSet, VisualMeshMarker, 
};
use crate::workcell::WorkcellVisualizationMarker;

/*
pub mod select;
pub use select::*;

use bevy::prelude::*;
use bevy_mod_outline::OutlinePlugin;
use bevy_mod_raycast::deferred::DeferredRaycastingPlugin;
use bevy_polyline::PolylinePlugin;

#[derive(Reflect)]
pub struct SiteRaycastSet;

#[derive(Default)]
pub struct InteractionPlugin { }

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<InteractionState>()
            .init_resource::<GizmoBlockers>()
            .configure_sets(
                PostUpdate,
                (
                    SiteUpdateSet::AssignOrphansFlush,
                    InteractionUpdateSet::AddVisuals,
                    InteractionUpdateSet::CommandFlush,
                    InteractionUpdateSet::ProcessVisuals,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                apply_deferred.in_set(InteractionUpdateSet::CommandFlush),
            )
            .add_plugins(PolylinePlugin)
            .add_plugins(DeferredRaycastingPlugin::<SiteRaycastSet>::default())
            .init_resource::<InteractionAssets>()
            .init_resource::<Cursor>()
            .init_resource::<CameraControls>()
            .init_resource::<Picked>()
            .init_resource::<PickingBlockers>()
            .init_resource::<GizmoState>()
            .insert_resource(HighlightAnchors(false))
            .add_event::<ChangePick>()
            .add_event::<MoveTo>()
            .add_event::<GizmoClicked>()
            .add_event::<SpawnPreview>()
            .add_plugins((
                OutlinePlugin,
                CategoryVisibilityPlugin::<VisualMeshMarker>::visible(true),
                CategoryVisibilityPlugin::<CollisionMeshMarker>::visible(false),
                CategoryVisibilityPlugin::<WorkcellVisualizationMarker>::visible(true),
            ))
            .add_plugins((CameraControlsPlugin, ModelPreviewPlugin));

            app.add_plugins(SelectionPlugin::default())
                .add_systems(
                    Update,
                    (
                        make_lift_doormat_gizmo,
                        update_doormats_for_level_change,
                        update_picking_cam,
                        update_physical_light_visual_cues,
                        make_selectable_entities_pickable,
                        update_anchor_visual_cues.after(SelectionServiceStages::Select),
                        update_popups.after(SelectionServiceStages::Select),
                        update_unassigned_anchor_cues,
                        update_anchor_proximity_xray.after(SelectionServiceStages::PickFlush),
                        remove_deleted_supports_from_visual_cues,
                        on_highlight_anchors_change,
                    )
                        .run_if(in_state(InteractionState::Enable)),
                )
                // Split the above because of a compile error when the tuple is too large
                .add_systems(
                    Update,
                    (
                        update_lane_visual_cues.after(SelectionServiceStages::Select),
                        update_edge_visual_cues.after(SelectionServiceStages::Select),
                        update_point_visual_cues.after(SelectionServiceStages::Select),
                        update_path_visual_cues.after(SelectionServiceStages::Select),
                        update_outline_visualization.after(SelectionServiceStages::Select),
                        update_highlight_visualization.after(SelectionServiceStages::Select),
                        update_cursor_hover_visualization.after(SelectionServiceStages::Select),
                        update_gizmo_click_start.after(SelectionServiceStages::Select),
                        update_gizmo_release,
                        update_drag_motions
                            .after(update_gizmo_click_start)
                            .after(update_gizmo_release),
                        handle_lift_doormat_clicks.after(update_gizmo_click_start),
                        manage_previews,
                        update_physical_camera_preview,
                        dirty_changed_lifts,
                        handle_preview_window_close,
                    )
                        .run_if(in_state(InteractionState::Enable)),
                )
                .add_systems(
                    PostUpdate,
                    (
                        add_anchor_visual_cues,
                        remove_interaction_for_subordinate_anchors,
                        add_lane_visual_cues,
                        add_edge_visual_cues,
                        add_point_visual_cues,
                        add_path_visual_cues,
                        add_outline_visualization,
                        add_highlight_visualization,
                        add_cursor_hover_visualization,
                        add_physical_light_visual_cues,
                        add_popups,
                    )
                        .run_if(in_state(InteractionState::Enable))
                        .in_set(InteractionUpdateSet::AddVisuals),
                )
                .add_systems(
                    Update,
                    propagate_visual_cues
                        .run_if(in_state(InteractionState::Enable))
                        .in_set(InteractionUpdateSet::ProcessVisuals),
                )
                .add_systems(OnExit(InteractionState::Enable), hide_cursor)
                .add_systems(
                    PostUpdate,
                    (
                        move_anchor.before(update_anchor_transforms),
                        move_pose,
                        make_gizmos_pickable,
                    )
                        .run_if(in_state(InteractionState::Enable)),
                )
                .add_systems(First, update_picked);
    }
}

pub fn set_visibility(entity: Entity, q_visibility: &mut Query<&mut Visibility>, visible: bool) {
    if let Some(mut visibility) = q_visibility.get_mut(entity).ok() {
        let v = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        // Avoid a mutable access if nothing actually needs to change
        if *visibility != v {
            *visibility = v;
        }
    }
}
*/
