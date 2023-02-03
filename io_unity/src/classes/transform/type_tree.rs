use super::{Transform, TransformObject};

use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObjectRef;
use glam::Mat4;

impl TransformObject for Transform<'_> {
    fn get_father(&self) -> anyhow::Result<TypeTreeObjectRef> {
        self.get_father()
    }

    fn get_local_mat(&self) -> anyhow::Result<Mat4> {
        Ok(Mat4::from_scale_rotation_translation(
            self.get_local_scale()?,
            self.get_local_rotation()?,
            self.get_local_position()?,
        ))
    }

    fn get_children(&self) -> anyhow::Result<Vec<TypeTreeObjectRef>> {
        self.get_children()
    }
}

impl Transform<'_> {
    fn get_father(&self) -> anyhow::Result<TypeTreeObjectRef> {
        TypeTreeObjectRef::try_cast_from(self.inner, "/Base/m_Father")
    }

    fn get_local_rotation(&self) -> anyhow::Result<glam::Quat> {
        glam::Quat::try_cast_from(self.inner, "/Base/m_LocalRotation")
    }

    fn get_local_position(&self) -> anyhow::Result<glam::Vec3> {
        glam::Vec3::try_cast_from(self.inner, "/Base/m_LocalPosition")
    }

    fn get_local_scale(&self) -> anyhow::Result<glam::Vec3> {
        glam::Vec3::try_cast_from(self.inner, "/Base/m_LocalScale")
    }

    fn get_children(&self) -> anyhow::Result<Vec<TypeTreeObjectRef>> {
        <Vec<TypeTreeObjectRef>>::try_cast_from(self.inner, "/Base/m_Children/Array")
    }
}
