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

use crate::{
    bevy_egui::egui::Ui,
    widgets::{prelude::*, Inspect, SelectorWidget},
    CreateJoint, Dependents,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use rmf_workcell_format::{FrameMarker, JointProperties};

#[derive(SystemParam)]
pub struct InspectJoint<'w, 's> {
    joints: Query<
        'w,
        's,
        (
            &'static Parent,
            &'static Dependents,
            &'static JointProperties,
        ),
    >,
    frames: Query<'w, 's, (), With<FrameMarker>>,
    selector: SelectorWidget<'w, 's>,
}

impl<'w, 's> WidgetSystem<Inspect> for InspectJoint<'w, 's> {
    fn show(
        Inspect { selection, .. }: Inspect,
        ui: &mut Ui,
        state: &mut SystemState<Self>,
        world: &mut World,
    ) {
        let mut params = state.get_mut(world);
        params.show_widget(selection, ui);
    }
}

impl<'w, 's> InspectJoint<'w, 's> {
    pub fn show_widget(&mut self, id: Entity, ui: &mut Ui) {
        let Ok((parent, deps, joint_properties)) = self.joints.get(id) else {
            return;
        };

        ui.label("Parent frame");
        self.selector.show_widget(**parent, ui);

        if let Some(frame_dep) = deps.iter().find(|d| self.frames.get(**d).is_ok()) {
            ui.label("Child frame");
            self.selector.show_widget(*frame_dep, ui);
        }

        ui.horizontal(|ui| {
            ui.label("Joint Type");
            // TODO(luca) Make this a ComboBox to edit joint value data
            ui.label(joint_properties.label());
        });
        // TODO(luca) add joint limit and joint axis inspectors
    }
}

#[derive(SystemParam)]
pub struct InspectJointCreator<'w, 's> {
    frame_parents: Query<'w, 's, &'static Parent, With<FrameMarker>>,
    joints: Query<'w, 's, (), With<JointProperties>>,
    create_joint: EventWriter<'w, CreateJoint>,
}

impl<'w, 's> WidgetSystem<Inspect> for InspectJointCreator<'w, 's> {
    fn show(
        Inspect { selection, .. }: Inspect,
        ui: &mut Ui,
        state: &mut SystemState<Self>,
        world: &mut World,
    ) {
        let mut params = state.get_mut(world);
        params.show_widget(selection, ui);
    }
}

impl<'w, 's> InspectJointCreator<'w, 's> {
    pub fn show_widget(&mut self, id: Entity, ui: &mut Ui) {
        let Ok(parent) = self.frame_parents.get(id) else {
            return;
        };
        if self.frame_parents.get(parent.get()).is_ok() {
            if ui
                .button("Create joint")
                .on_hover_text(
                    "Create a fixed joint and place it between the parent frame and this frame",
                )
                .clicked()
            {
                self.create_joint.send(CreateJoint {
                    parent: parent.get(),
                    child: id,
                });
            }
        }
    }
}
