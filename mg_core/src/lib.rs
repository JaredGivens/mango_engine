pub use anyhow::Result;
pub use once_cell::sync::Lazy;
pub use parking_lot::Mutex;
pub use parking_lot::MutexGuard;
pub use parry3d::na;
pub use parry3d::query::RayCast;
pub use std::f32::consts::*;
pub use std::sync::atomic;
pub use std::sync::Arc;

pub type Similarity = na::Similarity3<f32>;
pub const SQRT3: f32 = 1.732050807568877293527446341505872367_f32;
pub type Point3i = na::Point3<i16>;
pub type Point3f = na::Point3<f32>;
pub type Iso = na::Isometry3<f32>;
pub type UVec3f = na::UnitVector3<f32>;
pub type Vec3f = na::Vector3<f32>;
pub type Vec2f = na::Vector2<f32>;
pub type Quat = na::UnitQuaternion<f32>;
pub type Seg = parry3d::shape::Segment;
pub type Mat4 = parry3d::na::Matrix4<f32>;
pub type Ray = parry3d::query::Ray;
pub type Tri = parry3d::shape::Triangle;

pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>()) }
}
