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

use librmf_site_editor::interaction::{CategoryVisibility, SetCategoryVisibility};
use crate::site::{
    CollisionMeshMarker, DoorMarker, FiducialMarker, FloorMarker, LaneMarker, LiftCabin,
    LiftCabinDoorMarker, LocationTags, MeasurementMarker, VisualMeshMarker, WallMarker,
};
use crate::widgets::menu_bar::{MenuEvent, MenuItem, MenuVisualizationStates, ViewMenu};
use crate::workcell::WorkcellVisualizationMarker;
use crate::AppState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(SystemParam)]
struct VisibilityEvents<'w> {
    visuals: EventWriter<'w, SetCategoryVisibility<VisualMeshMarker>>,
    collisions: EventWriter<'w, SetCategoryVisibility<CollisionMeshMarker>>,
    origin_axis: EventWriter<'w, SetCategoryVisibility<WorkcellVisualizationMarker>>,
}

#[derive(Default)]
pub struct ViewMenuPlugin;

#[derive(Resource)]
pub struct ViewMenuItems {
    visuals: Entity,
    collisions: Entity,
    origin_axis: Entity,
}

impl FromWorld for ViewMenuItems {
    fn from_world(world: &mut World) -> Self {
        let default_visibility = world.resource::<CategoryVisibility<CollisionMeshMarker>>();
        let collisions = world
            .spawn(MenuItem::CheckBox(
                "Collision meshes".to_string(),
                default_visibility.0,
            ))
            .insert(MenuVisualizationStates(active_states.clone()))
            .set_parent(view_header)
            .id();
        let default_visibility = world.resource::<CategoryVisibility<VisualMeshMarker>>();
        let visuals = world
            .spawn(MenuItem::CheckBox(
                "Visual meshes".to_string(),
                default_visibility.0,
            ))
            .insert(MenuVisualizationStates(active_states))
            .set_parent(view_header)
            .id();
        let default_visibility =
            world.resource::<CategoryVisibility<WorkcellVisualizationMarker>>();
        let origin_axis = world
            .spawn(MenuItem::CheckBox(
                "Reference axis".to_string(),
                default_visibility.0,
            ))
            .insert(MenuVisualizationStates(workcell_states))
            .set_parent(view_header)
            .id();

        ViewMenuItems {
            collisions,
            visuals,
            origin_axis,
        }
    }
}

fn handle_view_menu_events(
    mut menu_events: EventReader<MenuEvent>,
    view_menu: Res<ViewMenuItems>,
    mut menu_items: Query<&mut MenuItem>,
    mut events: VisibilityEvents,
) {
    let mut toggle = |entity| {
        let mut menu = menu_items.get_mut(entity).unwrap();
        let value = menu.checkbox_value_mut().unwrap();
        *value = !*value;
        *value
    };
    for event in menu_events.read() {
        if event.clicked() && event.source() == view_menu.collisions {
            events.collisions.send(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.visuals {
            events.visuals.send(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.origin_axis {
            events.origin_axis.send(toggle(event.source()).into());
        }
    }
}

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewMenuItems>()
            .add_systems(Update, handle_view_menu_events);
    }
}
