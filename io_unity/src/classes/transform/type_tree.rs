use super::{Transform, TransformObject};

use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObject;
use glam::Mat4;

impl TransformObject for Transform<'_> {
    fn get_father(&self) -> Option<TypeTreeObject> {
        Some(self.get_father()?)
    }

    fn get_local_mat(&self) -> Option<Mat4> {
        Some(Mat4::from_scale_rotation_translation(
            self.get_local_scale()?,
            self.get_local_rotation()?,
            self.get_local_position()?,
        ))
    }

    fn get_children(&self) -> Option<Vec<TypeTreeObject>> {
        Some(self.get_children()?)
    }
}

impl Transform<'_> {
    fn get_father(&self) -> Option<TypeTreeObject> {
        TypeTreeObject::try_cast_from(&self.inner, "/Base/m_Father").ok()
    }

    fn get_local_rotation(&self) -> Option<glam::Quat> {
        glam::Quat::try_cast_from(&self.inner, "/Base/m_LocalRotation").ok()
    }

    fn get_local_position(&self) -> Option<glam::Vec3> {
        glam::Vec3::try_cast_from(&self.inner, "/Base/m_LocalPosition").ok()
    }

    fn get_local_scale(&self) -> Option<glam::Vec3> {
        glam::Vec3::try_cast_from(&self.inner, "/Base/m_LocalScale").ok()
    }

    fn get_children(&self) -> Option<Vec<TypeTreeObject>> {
        <Vec<TypeTreeObject>>::try_cast_from(&self.inner, "/Base/m_Children/Array").ok()
    }
}
