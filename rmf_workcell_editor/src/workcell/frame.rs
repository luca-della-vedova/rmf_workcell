/*
 * Copyright (C) 2024 Open Source Robotics Foundation
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

use crate::interaction::AnchorVisualization;
use bevy::prelude::*;
use rmf_workcell_format::FrameMarker;

// TODO(luca) We should probably have a different mesh altogether for workcell anchors, rather than
// a scaled down version of site anchors.
pub fn scale_workcell_anchors(
    new_frames: Query<&AnchorVisualization, With<FrameMarker>>,
    mut transforms: Query<&mut Transform>,
) {
    for frame in new_frames.iter() {
        if let Ok(mut tf) = transforms.get_mut(frame.body) {
            tf.scale = Vec3::splat(0.25);
        }
    }
}
