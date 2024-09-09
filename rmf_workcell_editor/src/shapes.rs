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

use bevy::math::Affine3A;
use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridSettings};
use bevy_polyline::{material::PolylineMaterial, polyline::Polyline};
use std::collections::{BTreeMap, HashMap};

const X_AXIS_COLOR: Color = Color::rgb(1.0, 0.2, 0.2);
const Y_AXIS_COLOR: Color = Color::rgb(0.2, 1.0, 0.2);

pub(crate) fn make_infinite_grid(
    scale: f32,
    fadeout_distance: f32,
    shadow_color: Option<Color>,
) -> InfiniteGridBundle {
    let mut settings = InfiniteGridSettings::default();
    // The upstream bevy_infinite_grid developers use an x-z plane grid but we
    // use an x-y plane grid, so we need to make some tweaks.
    settings.x_axis_color = X_AXIS_COLOR;
    settings.z_axis_color = Y_AXIS_COLOR;
    settings.fadeout_distance = fadeout_distance;
    settings.shadow_color = shadow_color;
    let transform = Transform::from_rotation(Quat::from_rotation_x(90_f32.to_radians()))
        .with_scale(Vec3::splat(scale));

    InfiniteGridBundle {
        settings,
        transform,
        ..Default::default()
    }
}
