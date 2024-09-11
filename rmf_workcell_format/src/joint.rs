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

#[cfg(feature = "bevy")]
use bevy::ecs::system::EntityCommands;
#[cfg(feature = "bevy")]
use bevy::prelude::{Bundle, Component, Entity, Event, SpatialBundle};

use crate::{Category, NameInWorkcell};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JointAxis([f32; 3]);

impl From<&urdf_rs::Axis> for JointAxis {
    fn from(axis: &urdf_rs::Axis) -> Self {
        Self(axis.xyz.map(|t| t as f32))
    }
}

impl From<&JointAxis> for urdf_rs::Axis {
    fn from(axis: &JointAxis) -> Self {
        Self {
            xyz: urdf_rs::Vec3(axis.0.map(|v| v as f64)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum RangeLimits {
    None,
    Symmetric(f32),
    Asymmetric {
        lower: Option<f32>,
        upper: Option<f32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JointLimits {
    position: RangeLimits,
    effort: RangeLimits,
    velocity: RangeLimits,
}

impl From<&urdf_rs::JointLimit> for JointLimits {
    fn from(limit: &urdf_rs::JointLimit) -> Self {
        Self {
            position: RangeLimits::Asymmetric {
                lower: Some(limit.lower as f32),
                upper: Some(limit.upper as f32),
            },
            effort: RangeLimits::Symmetric(limit.effort as f32),
            velocity: RangeLimits::Symmetric(limit.velocity as f32),
        }
    }
}

impl From<&JointLimits> for urdf_rs::JointLimit {
    fn from(limits: &JointLimits) -> Self {
        const DEFAULT_EFFORT_LIMIT: f64 = 1e3;
        const DEFAULT_VELOCITY_LIMIT: f64 = 10.0;
        fn min_or_default(slice: [Option<f32>; 2], default: f64) -> f64 {
            let mut vec = slice
                .iter()
                .filter_map(|v| v.map(|m| m as f64))
                .collect::<Vec<_>>();
            vec.sort_by(|a, b| a.total_cmp(b));
            vec.first().cloned().unwrap_or(default)
        }
        // 0.0 is a valid default in urdf for lower and upper limits
        let (lower, upper) = match limits.position {
            RangeLimits::None => (0.0, 0.0),
            RangeLimits::Symmetric(l) => (l as f64, l as f64),
            RangeLimits::Asymmetric { lower, upper } => (
                lower.map(|v| v as f64).unwrap_or_default(),
                upper.map(|v| v as f64).unwrap_or_default(),
            ),
        };
        let effort = match limits.effort {
            RangeLimits::None => {
                println!(
                    "No effort limit found when exporting to urdf, setting to {}",
                    DEFAULT_EFFORT_LIMIT
                );
                DEFAULT_EFFORT_LIMIT
            }
            RangeLimits::Symmetric(l) => l as f64,
            RangeLimits::Asymmetric { lower, upper } => {
                let limit = min_or_default([lower, upper], DEFAULT_EFFORT_LIMIT);
                println!(
                    "Asymmetric effort limit found when exporting to urdf, setting to {}",
                    limit
                );
                limit
            }
        };
        let velocity = match limits.velocity {
            RangeLimits::None => {
                println!(
                    "No velocity limit found when exporting to urdf, setting to {}",
                    DEFAULT_VELOCITY_LIMIT
                );
                DEFAULT_VELOCITY_LIMIT
            }
            RangeLimits::Symmetric(l) => l as f64,
            RangeLimits::Asymmetric { lower, upper } => {
                let limit = min_or_default([lower, upper], DEFAULT_VELOCITY_LIMIT);
                println!(
                    "Asymmetric velocity limit found when exporting to urdf, setting to {}",
                    limit
                );
                limit
            }
        };
        Self {
            lower,
            upper,
            effort,
            velocity,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct Joint {
    pub name: NameInWorkcell,
    pub properties: JointProperties,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub enum JointProperties {
    Fixed,
    Prismatic(SingleDofJoint),
    Revolute(SingleDofJoint),
    Continuous(SingleDofJoint),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SingleDofJoint {
    pub limits: JointLimits,
    pub axis: JointAxis,
}

impl JointProperties {
    pub fn label(&self) -> String {
        match &self {
            JointProperties::Fixed => "Fixed",
            JointProperties::Revolute(_) => "Revolute",
            JointProperties::Prismatic(_) => "Prismatic",
            JointProperties::Continuous(_) => "Continuous",
        }
        .to_string()
    }
}

// TODO(luca) should commands implementation be in rmf_workcell_editor instead of rmf_workcell_format?
/// Custom spawning implementation since bundles don't allow options
#[cfg(feature = "bevy")]
impl Joint {
    pub fn add_bevy_components(&self, commands: &mut EntityCommands) {
        commands.insert((
            SpatialBundle::INHERITED_IDENTITY,
            Category::Joint,
            self.name.clone(),
            self.properties.clone(),
        ));
    }
}

/// Event used  to request the creation of a joint between a parent and a child frame
#[cfg(feature = "bevy")]
#[derive(Event)]
pub struct CreateJoint {
    pub parent: Entity,
    pub child: Entity,
    // TODO(luca) Add different properties here such as JointType
}
