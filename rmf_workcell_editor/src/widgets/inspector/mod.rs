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

pub mod inspect_joint;
pub use inspect_joint::*;

pub mod inspect_name;
pub use inspect_name::*;

pub mod inspect_workcell_parent;
pub use inspect_workcell_parent::*;

use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_egui::egui::{CollapsingHeader, Ui};
use librmf_site_editor::widgets::MinimalInspectorPlugin;
use librmf_site_editor::{
    interaction::Selection,
    widgets::{
        prelude::*, InspectAnchor, InspectAnchorDependents, InspectAssetSource, InspectPose,
        InspectPrimitiveShape, InspectScale,
    },
};
use rmf_workcell_format::*;
use rmf_workcell_format::{Category, SiteID};
use smallvec::SmallVec;

/// Use this to create a standard inspector plugin that covers the common use
/// cases of the site editor.
#[derive(Default)]
pub struct StandardInspectorPlugin {}

impl Plugin for StandardInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MinimalInspectorPlugin::default())
            .add_plugins((
                InspectionPlugin::<InspectName>::new(),
                InspectionPlugin::<InspectAnchor>::new(),
                InspectionPlugin::<InspectAnchorDependents>::new(),
                InspectionPlugin::<InspectPose>::new(),
                InspectionPlugin::<InspectScale>::new(),
                InspectionPlugin::<InspectAssetSource>::new(),
                InspectionPlugin::<InspectPrimitiveShape>::new(),
                InspectionPlugin::<InspectWorkcellParent>::new(),
                InspectionPlugin::<InspectJoint>::new(),
            ));
    }
}
