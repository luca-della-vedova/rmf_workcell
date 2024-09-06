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

pub mod geometry;
pub use geometry::*;

pub mod inertial;
pub use inertial::*;

pub mod joint;
pub use joint::*;

pub mod workcell;
pub use workcell::*;

// TODO(luca) move away from SiteID?
pub use rmf_site_format::{Pending, PrimitiveShape, NameInSite, AssetSource, Pose, Anchor, Category, Model, ModelMarker, SiteID, Scale};

mod is_default;
pub(crate) use is_default::*;

pub const CURRENT_MAJOR_VERSION: u32 = 0;
pub const CURRENT_MINOR_VERSION: u32 = 1;
