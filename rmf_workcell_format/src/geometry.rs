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

use crate::is_default;

use rmf_site_format::{AssetSource, Pose, PrimitiveShape};

use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Geometry {
    //#[serde(flatten)]
    Primitive(PrimitiveShape),
    Mesh {
        source: AssetSource,
        #[serde(default, skip_serializing_if = "is_default")]
        scale: Option<Vec3>,
    },
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry::Primitive(PrimitiveShape::Box { size: [0.0; 3] })
    }
}

impl From<Geometry> for urdf_rs::Geometry {
    fn from(geometry: Geometry) -> Self {
        match geometry {
            Geometry::Mesh { source, scale } => urdf_rs::Geometry::Mesh {
                // SAFETY: We don't need to validate the syntax of the asset
                // path because that will be done later when we attempt to load
                // this as an asset.
                filename: unsafe { (&source).as_unvalidated_asset_path() },
                scale: scale.map(|v| urdf_rs::Vec3([v.x as f64, v.y as f64, v.z as f64])),
            },
            Geometry::Primitive(PrimitiveShape::Box { size: [x, y, z] }) => {
                urdf_rs::Geometry::Box {
                    size: urdf_rs::Vec3([x as f64, y as f64, z as f64]),
                }
            }
            Geometry::Primitive(PrimitiveShape::Cylinder { radius, length }) => {
                urdf_rs::Geometry::Cylinder {
                    radius: radius as f64,
                    length: length as f64,
                }
            }
            Geometry::Primitive(PrimitiveShape::Capsule { radius, length }) => {
                urdf_rs::Geometry::Capsule {
                    radius: radius as f64,
                    length: length as f64,
                }
            }
            Geometry::Primitive(PrimitiveShape::Sphere { radius }) => urdf_rs::Geometry::Sphere {
                radius: radius as f64,
            },
        }
    }
}

// TODO(luca) feature gate urdf support
impl From<&urdf_rs::Geometry> for Geometry {
    fn from(geom: &urdf_rs::Geometry) -> Self {
        match geom {
            urdf_rs::Geometry::Box { size } => Geometry::Primitive(PrimitiveShape::Box {
                size: (**size).map(|f| f as f32),
            }),
            urdf_rs::Geometry::Cylinder { radius, length } => {
                Geometry::Primitive(PrimitiveShape::Cylinder {
                    radius: *radius as f32,
                    length: *length as f32,
                })
            }
            urdf_rs::Geometry::Capsule { radius, length } => {
                Geometry::Primitive(PrimitiveShape::Capsule {
                    radius: *radius as f32,
                    length: *length as f32,
                })
            }
            urdf_rs::Geometry::Sphere { radius } => Geometry::Primitive(PrimitiveShape::Sphere {
                radius: *radius as f32,
            }),
            urdf_rs::Geometry::Mesh { filename, scale } => {
                let scale = scale
                    .clone()
                    .and_then(|s| Some(Vec3::from_array(s.map(|v| v as f32))));
                // Most (all?) Urdf files use package references, we fallback to local if that is
                // not the case
                let source = if let Some(path) = filename.strip_prefix("package://") {
                    AssetSource::Package(path.to_owned())
                } else {
                    AssetSource::Local(filename.clone())
                };
                Geometry::Mesh { source, scale }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct WorkcellModel {
    pub name: String,
    pub geometry: Geometry,
    pub pose: Pose,
}

impl WorkcellModel {
    fn from_urdf_data(
        pose: &urdf_rs::Pose,
        name: &Option<String>,
        geometry: &urdf_rs::Geometry,
    ) -> Self {
        WorkcellModel {
            name: name.clone().unwrap_or_default(),
            geometry: geometry.into(),
            pose: pose.into(),
        }
    }
}

impl From<&urdf_rs::Visual> for WorkcellModel {
    fn from(visual: &urdf_rs::Visual) -> Self {
        WorkcellModel::from_urdf_data(&visual.origin, &visual.name, &visual.geometry)
    }
}

impl From<&urdf_rs::Collision> for WorkcellModel {
    fn from(collision: &urdf_rs::Collision) -> Self {
        WorkcellModel::from_urdf_data(&collision.origin, &collision.name, &collision.geometry)
    }
}
