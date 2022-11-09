use super::TransformObject;
use crate::classes::component::{self, ComponentObject};
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
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
}

impl Transform<'_> {
    fn get_father(&self) -> Option<PPtr> {
        self.inner
            .get_object_by_path("/Base/m_Father")
            .and_then(|f| Some(PPtr::new(f)))
    }

    fn get_local_rotation(&self) -> Option<glam::Quat> {
        self.inner.get_quat_by_path("/Base/m_LocalRotation")
    }

    fn get_local_position(&self) -> Option<glam::Vec3> {
        self.inner.get_vec3f_by_path("/Base/m_LocalPosition")
    }

    fn get_local_scale(&self) -> Option<glam::Vec3> {
        self.inner.get_vec3f_by_path("/Base/m_LocalScale")
    }
}
