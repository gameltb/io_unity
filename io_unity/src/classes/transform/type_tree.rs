use super::{Transform, TransformObject};

use crate::error::ReadResult;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObjectRef;
use glam::Mat4;

impl TransformObject for Transform<'_> {
    fn get_father(&self) -> ReadResult<TypeTreeObjectRef> {
        self.get_father()
    }

    fn get_local_mat(&self) -> ReadResult<Mat4> {
        Ok(Mat4::from_scale_rotation_translation(
            self.get_local_scale()?,
            self.get_local_rotation()?,
            self.get_local_position()?,
        ))
    }

    fn get_children(&self) -> ReadResult<Vec<TypeTreeObjectRef>> {
        self.get_children()
    }
}

impl Transform<'_> {
    fn get_father(&self) -> ReadResult<TypeTreeObjectRef> {
        TypeTreeObjectRef::try_cast_from(self.inner, "/Base/m_Father")
    }

    fn get_local_rotation(&self) -> ReadResult<glam::Quat> {
        glam::Quat::try_cast_from(self.inner, "/Base/m_LocalRotation")
    }

    fn get_local_position(&self) -> ReadResult<glam::Vec3> {
        glam::Vec3::try_cast_from(self.inner, "/Base/m_LocalPosition")
    }

    fn get_local_scale(&self) -> ReadResult<glam::Vec3> {
        glam::Vec3::try_cast_from(self.inner, "/Base/m_LocalScale")
    }

    fn get_children(&self) -> ReadResult<Vec<TypeTreeObjectRef>> {
        <Vec<TypeTreeObjectRef>>::try_cast_from(self.inner, "/Base/m_Children/Array")
    }
}
