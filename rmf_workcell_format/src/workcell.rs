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

use std::collections::{BTreeMap, HashMap};

use std::io;

use crate::*;
#[cfg(feature = "bevy")]
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
#[cfg(feature = "bevy")]
use bevy::reflect::{TypePath, TypeUuid};
use rmf_site_format::{misc::Rotation, Anchor, Pose, RefTrait};
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

/// Helper structure to serialize / deserialize entities with parents
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Parented<P: RefTrait, T> {
    pub parent: P,
    #[serde(flatten)]
    pub bundle: T,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct FrameMarker;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct Frame {
    #[serde(flatten)]
    pub anchor: Anchor,
    pub name: NameInWorkcell,
    #[serde(skip)]
    pub marker: FrameMarker,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct NameOfWorkcell(pub String);

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct WorkcellProperties {
    pub name: NameOfWorkcell,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component, Deref, DerefMut))]
pub struct NameInWorkcell(pub String);

// TODO(luca) we might need a different bundle to denote a workcell included in site
// editor mode to deal with serde of workcells there (that would only have an asset source?)
/// Container for serialization / deserialization of workcells
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Workcell {
    /// Workcell specific properties
    #[serde(flatten)]
    pub properties: WorkcellProperties,
    /// Site ID, used for entities to set their parent to the root workcell
    pub id: u32,
    /// Frames, key is their id, used for hierarchy
    pub frames: BTreeMap<u32, Parented<u32, Frame>>,
    /// Visuals, key is their id, used for hierarchy
    pub visuals: BTreeMap<u32, Parented<u32, WorkcellModel>>,
    /// Collisions, key is their id, used for hierarchy
    pub collisions: BTreeMap<u32, Parented<u32, WorkcellModel>>,
    /// Inertias, key is their id, used for hierarchy
    pub inertias: BTreeMap<u32, Parented<u32, Inertia>>,
    /// Joints, key is their id, used for hierarchy. They must have a frame as a parent and a frame
    /// as a child
    pub joints: BTreeMap<u32, Parented<u32, Joint>>,
}

#[derive(Debug, ThisError)]
pub enum UrdfImportError {
    #[error("a joint refers to a non existing link [{0}]")]
    BrokenJointReference(String),
    // TODO(luca) Add urdf_rs::JointType to this error, it doesn't implement Display
    #[error("unsupported joint type found")]
    UnsupportedJointType,
}

#[derive(Debug, ThisError)]
pub enum WorkcellToUrdfError {
    #[error("Invalid anchor type {0:?}")]
    InvalidAnchorType(Anchor),
    #[error("Urdf error: {0}")]
    WriteToStringError(#[from] urdf_rs::UrdfError),
    #[error("Broken reference: {0}")]
    BrokenReference(u32),
}

impl Workcell {
    pub fn from_urdf(urdf: &urdf_rs::Robot) -> Result<Self, UrdfImportError> {
        let mut frame_name_to_id = HashMap::new();
        let root_id = 0_u32;
        let mut cur_id = 1u32..;
        let mut frames = BTreeMap::new();
        let mut visuals = BTreeMap::new();
        let mut collisions = BTreeMap::new();
        let mut inertias = BTreeMap::new();
        let mut joints = BTreeMap::new();
        // Populate here
        for link in &urdf.links {
            let inertia = Inertia::from(&link.inertial);
            // Add a frame with the link's name, then the inertia data as a child
            let frame_id = cur_id.next().unwrap();
            let inertia_id = cur_id.next().unwrap();
            frame_name_to_id.insert(link.name.clone(), frame_id);
            // Pose and parent will be overwritten by joints, if needed
            frames.insert(
                frame_id,
                Parented {
                    parent: root_id,
                    bundle: Frame {
                        anchor: Anchor::Pose3D(Pose::default()),
                        name: NameInWorkcell(link.name.clone()),
                        marker: Default::default(),
                    },
                },
            );
            inertias.insert(
                inertia_id,
                Parented {
                    parent: frame_id,
                    bundle: inertia,
                },
            );
            for visual in &link.visual {
                let model = WorkcellModel::from(visual);
                let visual_id = cur_id.next().unwrap();
                visuals.insert(
                    visual_id,
                    Parented {
                        parent: frame_id,
                        bundle: model,
                    },
                );
            }
            for collision in &link.collision {
                let model = WorkcellModel::from(collision);
                let collision_id = cur_id.next().unwrap();
                collisions.insert(
                    collision_id,
                    Parented {
                        parent: frame_id,
                        bundle: model,
                    },
                );
            }
        }
        for joint in &urdf.joints {
            let parent = frame_name_to_id.get(&joint.parent.link).ok_or(
                UrdfImportError::BrokenJointReference(joint.parent.link.clone()),
            )?;
            let child = frame_name_to_id.get(&joint.child.link).ok_or(
                UrdfImportError::BrokenJointReference(joint.child.link.clone()),
            )?;
            let properties = match joint.joint_type {
                urdf_rs::JointType::Revolute => JointProperties::Revolute(SingleDofJoint {
                    axis: (&joint.axis).into(),
                    limits: (&joint.limit).into(),
                }),
                urdf_rs::JointType::Prismatic => JointProperties::Prismatic(SingleDofJoint {
                    axis: (&joint.axis).into(),
                    limits: (&joint.limit).into(),
                }),
                urdf_rs::JointType::Fixed => JointProperties::Fixed,
                urdf_rs::JointType::Continuous => JointProperties::Continuous(SingleDofJoint {
                    axis: (&joint.axis).into(),
                    limits: (&joint.limit).into(),
                }),
                _ => {
                    return Err(UrdfImportError::UnsupportedJointType);
                }
            };
            let joint_id = cur_id.next().unwrap();
            // Reassign the child parenthood and pose to the joint
            // If the frame didn't exist we would have returned an error when populating child
            // hence this is safe.
            let child_frame = frames.get_mut(child).unwrap();
            child_frame.parent = joint_id;
            // In urdf, joint origin represents the coordinates of the joint in the
            // parent frame. The child is always in the origin of the joint
            child_frame.bundle.anchor = Anchor::Pose3D((&joint.origin).into());
            joints.insert(
                joint_id,
                Parented {
                    parent: *parent,
                    bundle: Joint {
                        name: NameInWorkcell(joint.name.clone()),
                        properties,
                    },
                },
            );
        }

        Ok(Workcell {
            properties: WorkcellProperties {
                name: NameOfWorkcell(urdf.name.clone()),
            },
            id: root_id,
            frames,
            visuals,
            collisions,
            inertias,
            joints,
        })
    }
    pub fn to_writer<W: io::Write>(&self, writer: W) -> serde_json::Result<()> {
        serde_json::ser::to_writer_pretty(writer, self)
    }

    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::ser::to_string_pretty(self)
    }

    pub fn to_urdf(&self) -> Result<urdf_rs::Robot, WorkcellToUrdfError> {
        let mut parent_to_visuals = HashMap::new();
        for (_, visual) in self.visuals.iter() {
            let parent = visual.parent;
            let visual = &visual.bundle;
            let visual = urdf_rs::Visual {
                name: Some(visual.name.clone()),
                origin: visual.pose.into(),
                geometry: visual.geometry.clone().into(),
                material: None,
            };
            parent_to_visuals
                .entry(parent)
                .or_insert_with(Vec::new)
                .push(visual);
        }

        let mut parent_to_collisions = HashMap::new();
        for (_, collision) in self.collisions.iter() {
            let parent = collision.parent;
            let collision = &collision.bundle;
            let collision = urdf_rs::Collision {
                name: Some(collision.name.clone()),
                origin: collision.pose.into(),
                geometry: collision.geometry.clone().into(),
            };
            parent_to_collisions
                .entry(parent)
                .or_insert_with(Vec::new)
                .push(collision);
        }

        // If the workcell has a single frame child we can use the child as the base link.
        // Otherwise, we will need to spawn a new base link to contain all the workcell children
        let workcell_child_frames = self
            .frames
            .iter()
            .filter(|(_, frame)| frame.parent == self.id);
        let num_children = workcell_child_frames.clone().count();
        let frames = if num_children != 1 {
            // TODO(luca) remove hardcoding of base link name, it might in some cases create
            // duplicates
            let mut frames = self.frames.clone();
            let dummy_frame = Frame {
                anchor: Anchor::Pose3D(Pose {
                    rot: Rotation::Quat([0.0, 0.0, 0.0, 0.0]),
                    trans: [0.0, 0.0, 0.0],
                }),
                // As per Industrial Workcell Coordinate Conventions, the name of the workcell
                // datum link shall be "<workcell_name>_workcell_link".
                name: NameInWorkcell(String::from(
                    self.properties.name.0.clone() + "_workcell_link",
                )),
                marker: FrameMarker,
            };
            frames.insert(
                self.id,
                Parented {
                    // Root has no parent, use placeholder of max u32
                    parent: u32::MAX,
                    bundle: dummy_frame,
                },
            );
            frames
        } else {
            // Flatten the hierarchy by making the only child the new workcell base link
            self.frames.clone()
        };

        let mut parent_to_inertials = HashMap::new();
        for (_, inertia) in self.inertias.iter() {
            let parent = inertia.parent;
            let inertia = &inertia.bundle;
            let inertial = urdf_rs::Inertial::from(inertia);
            parent_to_inertials.insert(parent, inertial);
        }

        // TODO(luca) combine multiple frames without a joint inbetween into a single link.
        // For now as soon as a joint is missing the hierarchy will be broken
        let links = frames
            .iter()
            .map(|(frame_id, parented_frame)| {
                let name = parented_frame.bundle.name.0.clone();
                let inertial = parent_to_inertials.remove(&frame_id).unwrap_or_default();
                let collision = parent_to_collisions.remove(&frame_id).unwrap_or_default();
                let visual = parent_to_visuals.remove(&frame_id).unwrap_or_default();

                urdf_rs::Link {
                    name,
                    inertial,
                    collision,
                    visual,
                }
            })
            .collect::<Vec<_>>();

        let joints = self
            .joints
            .iter()
            .map(|(joint_id, parented_joint)| {
                let joint_parent = parented_joint.parent;
                let joint = &parented_joint.bundle;
                // The pose of the joint is the pose of the frame that has it as its parent
                let parent_frame = self
                    .frames
                    .get(&joint_parent)
                    .ok_or(WorkcellToUrdfError::BrokenReference(joint_parent))?;
                let child_frame = self
                    .frames
                    .values()
                    .find(|frame| frame.parent == *joint_id)
                    .ok_or(WorkcellToUrdfError::BrokenReference(*joint_id))?;
                let parent_name = parent_frame.bundle.name.clone();
                let child_name = child_frame.bundle.name.clone();
                let Anchor::Pose3D(pose) = child_frame.bundle.anchor else {
                    return Err(WorkcellToUrdfError::InvalidAnchorType(
                        child_frame.bundle.anchor.clone(),
                    ));
                };
                let (joint_type, axis, limit) = match &joint.properties {
                    JointProperties::Fixed => (
                        urdf_rs::JointType::Fixed,
                        urdf_rs::Axis::default(),
                        urdf_rs::JointLimit::default(),
                    ),
                    JointProperties::Revolute(joint) => (
                        urdf_rs::JointType::Revolute,
                        (&joint.axis).into(),
                        (&joint.limits).into(),
                    ),
                    JointProperties::Prismatic(joint) => (
                        urdf_rs::JointType::Prismatic,
                        (&joint.axis).into(),
                        (&joint.limits).into(),
                    ),
                    JointProperties::Continuous(joint) => (
                        urdf_rs::JointType::Continuous,
                        (&joint.axis).into(),
                        (&joint.limits).into(),
                    ),
                };
                Ok(urdf_rs::Joint {
                    name: joint.name.0.clone(),
                    joint_type,
                    origin: pose.into(),
                    parent: urdf_rs::LinkName {
                        link: parent_name.0,
                    },
                    child: urdf_rs::LinkName { link: child_name.0 },
                    axis,
                    limit,
                    dynamics: None,
                    mimic: None,
                    safety_controller: None,
                })
            })
            .collect::<Result<Vec<_>, WorkcellToUrdfError>>()?;

        // TODO(luca) implement materials
        let robot = urdf_rs::Robot {
            name: self.properties.name.0.clone(),
            links,
            joints,
            materials: vec![],
        };
        Ok(robot)
    }

    pub fn to_urdf_string(&self) -> Result<String, WorkcellToUrdfError> {
        let urdf = self.to_urdf()?;
        urdf_rs::write_to_string(&urdf).map_err(|e| WorkcellToUrdfError::WriteToStringError(e))
    }

    pub fn to_urdf_writer(&self, mut writer: impl io::Write) -> Result<(), std::io::Error> {
        let urdf = self
            .to_urdf_string()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writer.write_all(urdf.as_bytes())
    }

    pub fn from_reader<R: io::Read>(reader: R) -> serde_json::Result<Self> {
        serde_json::de::from_reader(reader)
    }

    pub fn from_str<'a>(s: &'a str) -> serde_json::Result<Self> {
        serde_json::de::from_str(s)
    }

    pub fn from_bytes<'a>(s: &'a [u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(s)
    }
}

#[cfg_attr(
    feature = "bevy",
    derive(Component, Clone, Debug, Deref, DerefMut, TypeUuid, TypePath)
)]
#[cfg_attr(feature = "bevy", uuid = "fe707f9e-c6f3-11ed-afa1-0242ac120002")]
pub struct UrdfRoot(pub urdf_rs::Robot);

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::{assert_float_eq, float_eq};
    use rmf_site_format::{Angle, PrimitiveShape};

    fn frame_by_name(
        frames: &BTreeMap<u32, Parented<u32, Frame>>,
        name: &str,
    ) -> Option<(u32, Parented<u32, Frame>)> {
        frames
            .iter()
            .find(|(_, parented_frame)| {
                parented_frame.bundle.name == Some(NameInWorkcell(name.to_string()))
            })
            .map(|(id, f)| (*id, f.clone()))
    }

    fn element_by_parent<T: Clone>(
        models: &BTreeMap<u32, Parented<u32, T>>,
        parent: u32,
    ) -> Option<(u32, Parented<u32, T>)> {
        models
            .iter()
            .find(|(_, parented_element)| parented_element.parent == parent)
            .map(|(id, e)| (*id, e.clone()))
    }

    fn is_pose_eq(p1: &Pose, p2: &Pose) -> bool {
        if !p1
            .trans
            .iter()
            .zip(p2.trans.iter())
            .map(|(t1, t2)| float_eq!(t1, t2, abs <= 1e-6))
            .all(|eq| eq)
        {
            return false;
        }
        match (
            p1.rot.as_euler_extrinsic_xyz(),
            p2.rot.as_euler_extrinsic_xyz(),
        ) {
            (Rotation::EulerExtrinsicXYZ(r1), Rotation::EulerExtrinsicXYZ(r2)) => r1
                .iter()
                .zip(r2.iter())
                .map(|(a1, a2)| float_eq!(a1.radians(), a2.radians(), abs <= 1e-6))
                .all(|eq| eq),
            _ => false,
        }
    }

    fn is_inertia_eq(i1: &Inertia, i2: &Inertia) -> bool {
        is_pose_eq(&i1.center, &i2.center)
            && float_eq!(i1.mass.0, i2.mass.0, abs <= 1e6)
            && float_eq!(i1.moment.ixx, i2.moment.ixx, abs <= 1e6)
            && float_eq!(i1.moment.ixy, i2.moment.ixy, abs <= 1e6)
            && float_eq!(i1.moment.ixz, i2.moment.ixz, abs <= 1e6)
            && float_eq!(i1.moment.iyy, i2.moment.iyy, abs <= 1e6)
            && float_eq!(i1.moment.iyz, i2.moment.iyz, abs <= 1e6)
            && float_eq!(i1.moment.izz, i2.moment.izz, abs <= 1e6)
    }

    #[test]
    fn urdf_roundtrip() {
        let urdf = urdf_rs::read_file("test/07-physics.urdf").unwrap();
        let workcell = Workcell::from_urdf(&urdf).unwrap();
        assert_eq!(workcell.visuals.len(), 16);
        assert_eq!(workcell.collisions.len(), 16);
        assert_eq!(workcell.frames.len(), 16);
        assert_eq!(workcell.joints.len(), 15);
        assert_eq!(workcell.properties.name.0, "physics");
        // Test that we convert poses from joints to frames
        let (right_leg_id, right_leg) = frame_by_name(&workcell.frames, "right_leg").unwrap();
        let target_right_leg_pose = Pose {
            trans: [0.0, -0.22, 0.25],
            rot: Default::default(),
        };
        assert!(right_leg
            .bundle
            .anchor
            .is_close(&Anchor::Pose3D(target_right_leg_pose), 1e-6));
        // Test that we can parse parenthood and properties of visuals and collisions correctly
        let (_, right_leg_visual) = element_by_parent(&workcell.visuals, right_leg_id).unwrap();
        let target_right_leg_model_pose = Pose {
            trans: [0.0, 0.0, -0.3],
            rot: Rotation::EulerExtrinsicXYZ([
                Angle::Rad(0.0),
                Angle::Rad(1.57075),
                Angle::Rad(0.0),
            ]),
        };
        assert!(is_pose_eq(
            &right_leg_visual.bundle.pose,
            &target_right_leg_model_pose
        ));
        assert!(matches!(
            right_leg_visual.bundle.geometry,
            Geometry::Primitive(PrimitiveShape::Box { .. })
        ));
        let (_, right_leg_collision) =
            element_by_parent(&workcell.collisions, right_leg_id).unwrap();
        assert!(is_pose_eq(
            &right_leg_collision.bundle.pose,
            &target_right_leg_model_pose
        ));
        assert!(matches!(
            right_leg_collision.bundle.geometry,
            Geometry::Primitive(PrimitiveShape::Box { .. })
        ));
        // Test inertia parenthood and parsing
        let (_, right_leg_inertia) = element_by_parent(&workcell.inertias, right_leg_id).unwrap();
        assert_float_eq!(right_leg_inertia.bundle.mass.0, 10.0, abs <= 1e6);
        let target_right_leg_inertia = Inertia {
            center: Pose::default(),
            mass: Mass(10.0),
            moment: Moment {
                ixx: 1.0,
                ixy: 0.0,
                ixz: 0.0,
                iyy: 1.0,
                iyz: 0.0,
                izz: 1.0,
            },
        };
        assert!(is_inertia_eq(
            &right_leg_inertia.bundle,
            &target_right_leg_inertia
        ));
        // Test joint parenthood and parsing
        let (_, right_leg_joint) = element_by_parent(&workcell.joints, right_leg_id).unwrap();
        assert!(matches!(
            right_leg_joint.bundle.properties,
            JointProperties::Fixed
        ));
        assert_eq!(
            right_leg_joint.bundle.name,
            NameInWorkcell("right_base_joint".to_string())
        );
        // Test that the new urdf contains the same data
        let new_urdf = workcell.to_urdf().unwrap();
        assert_eq!(new_urdf.name, "physics");
        assert_eq!(new_urdf.links.len(), 16);
        assert_eq!(new_urdf.joints.len(), 15);
        // Check that link information is preserved
        let right_leg_link = new_urdf
            .links
            .iter()
            .find(|l| l.name == "right_leg")
            .unwrap();
        assert!(is_inertia_eq(
            &(&right_leg_link.inertial).into(),
            &target_right_leg_inertia
        ));
        assert_eq!(right_leg_link.visual.len(), 1);
        assert_eq!(right_leg_link.collision.len(), 1);
        let right_leg_visual = right_leg_link.visual.get(0).unwrap();
        let right_leg_collision = right_leg_link.collision.get(0).unwrap();
        assert!(is_pose_eq(
            &(&right_leg_visual.origin).into(),
            &target_right_leg_model_pose
        ));
        assert!(is_pose_eq(
            &(&right_leg_collision.origin).into(),
            &target_right_leg_model_pose
        ));
        assert!(matches!(
            right_leg_visual.geometry,
            urdf_rs::Geometry::Box { .. }
        ));
        assert!(matches!(
            right_leg_collision.geometry,
            urdf_rs::Geometry::Box { .. }
        ));
        // Check that joint origin is preserved
        let right_leg_joint = new_urdf
            .joints
            .iter()
            .find(|l| l.name == "base_to_right_leg")
            .unwrap();
        assert!(is_pose_eq(
            &(&right_leg_joint.origin).into(),
            &target_right_leg_pose
        ));
        assert!(matches!(
            right_leg_joint.joint_type,
            urdf_rs::JointType::Fixed
        ));
    }
}
