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

use librmf_site_editor::interaction::InteractionPlugin;

pub mod select;
pub use select::*;

use bevy::prelude::*;

#[derive(Default)]
pub struct WorkcellInteractionPlugin;

impl Plugin for WorkcellInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InteractionPlugin::default(),
            place_object::ObjectPlacementPlugin::default(),
            ));
    }
}