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

use rmf_site_format::Pose;

#[cfg(feature = "bevy")]
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "bevy", derive(Component, Deref, DerefMut))]
pub struct Mass(pub f32);

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct Moment {
    pub ixx: f32,
    pub ixy: f32,
    pub ixz: f32,
    pub iyy: f32,
    pub iyz: f32,
    pub izz: f32,
}

impl From<&urdf_rs::Inertia> for Moment {
    fn from(inertia: &urdf_rs::Inertia) -> Self {
        Self {
            ixx: inertia.ixx as f32,
            ixy: inertia.ixy as f32,
            ixz: inertia.ixz as f32,
            iyy: inertia.iyy as f32,
            iyz: inertia.iyz as f32,
            izz: inertia.izz as f32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct Inertia {
    pub center: Pose,
    pub mass: Mass,
    pub moment: Moment,
}

impl From<&urdf_rs::Inertial> for Inertia {
    fn from(inertial: &urdf_rs::Inertial) -> Self {
        Self {
            center: (&inertial.origin).into(),
            mass: Mass(inertial.mass.value as f32),
            moment: (&inertial.inertia).into(),
        }
    }
}

impl From<&Inertia> for urdf_rs::Inertial {
    fn from(inertia: &Inertia) -> Self {
        Self {
            origin: inertia.center.into(),
            mass: urdf_rs::Mass {
                value: inertia.mass.0 as f64,
            },
            inertia: urdf_rs::Inertia {
                ixx: inertia.moment.ixx as f64,
                ixy: inertia.moment.ixy as f64,
                ixz: inertia.moment.ixz as f64,
                iyy: inertia.moment.iyy as f64,
                iyz: inertia.moment.iyz as f64,
                izz: inertia.moment.izz as f64,
            },
        }
    }
}
