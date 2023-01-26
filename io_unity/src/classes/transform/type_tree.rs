use super::TransformObject;
use crate::classes::component::{self, ComponentObject};
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObject;
use glam::Mat4;
use supercow::Supercow;

def_type_tree_class!(Transform);

impl component::DownCast for Transform<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn ComponentObject + Send + 'a>> {
        Supercow::owned(Box::new(component::type_tree::Component::new(&*self.inner)))
    }
}

impl TransformObject for Transform<'_> {
    fn get_father(&self) -> Option<Supercow<PPtr>> {
        Some(Supercow::owned(self.get_father()?))
    }

    fn get_local_mat(&self) -> Option<Mat4> {
        Some(Mat4::from_scale_rotation_translation(
            self.get_local_scale()?,
            self.get_local_rotation()?,
            self.get_local_position()?,
        ))
    }

    fn get_children(&self) -> Option<Supercow<Vec<PPtr>>> {
        Some(Supercow::owned(self.get_children()?))
    }
}

impl Transform<'_> {
    fn get_father(&self) -> Option<PPtr> {
        TypeTreeObject::try_cast_from(&self.inner, "/Base/m_Father")
            .ok()
            .and_then(|po| Some(PPtr::new(po)))
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

    fn get_children(&self) -> Option<Vec<PPtr>> {
        <Vec<TypeTreeObject>>::try_cast_from(&self.inner, "/Base/m_Children/Array")
            .ok()
            .and_then(|ov| Some(ov.into_iter().map(|po| PPtr::new(po)).collect()))
    }
}
