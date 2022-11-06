use std::{
    fs::OpenOptions,
    io::{BufReader, Cursor},
    path::PathBuf,
};

use pyo3::{prelude::*, types::PyBytes};

pub mod python_unity_class {
    use io_unity::classes::*;
    use pyo3::prelude::*;

    #[macro_export]
    macro_rules! def_python_unity_class {
        ($x:ident($y:path)) => {
            #[pyclass]
            pub struct $x(pub $y);
        };
        ($($xx:ident($yy:path)),+) => {

            $(def_python_unity_class!($xx($yy));)+

        };
    }

    def_python_unity_class!(
        AssetBundle(asset_bundle::AssetBundle),
        AudioClip(audio_clip::AudioClip),
        Texture2D(texture2d::Texture2D),
        Mesh(mesh::Mesh),
        Transform(transform::Transform),
        GameObject(game_object::GameObject),
        AnimationClip(animation_clip::AnimationClip),
        SkinnedMeshRenderer(skinned_mesh_renderer::SkinnedMeshRenderer),
        MeshRenderer(mesh_renderer::MeshRenderer),
        Material(material::Material),
        MeshFilter(mesh_filter::MeshFilter),
        MonoBehaviour(mono_behaviour::MonoBehaviour),
        MonoScript(mono_script::MonoScript),
        Animator(animator::Animator),
        Avatar(avatar::Avatar)
    );

    #[pyclass]
    pub struct UnityFS(pub io_unity::UnityFS);
    #[pyclass]
    pub struct SerializedFile(pub io_unity::SerializedFile);
}

use python_unity_class::*;

#[pymethods]
impl UnityFS {
    #[staticmethod]
    pub fn readfs(path: String) -> PyResult<Self> {
        let file_path = PathBuf::from(&path)
            .parent()
            .and_then(|p| Some(p.to_string_lossy().into_owned()));
        let file = OpenOptions::new().read(true).open(path)?;
        let file = BufReader::new(file);
        Ok(UnityFS(
            io_unity::UnityFS::read(Box::new(file), file_path)
                .or(Err(pyo3::exceptions::PyException::new_err("")))?,
        ))
    }

    pub fn get_cab(&self) -> PyResult<SerializedFile> {
        Ok(SerializedFile::read(
            self.0.get_file_by_path(
                self.0
                    .get_cab_path()
                    .get(0)
                    .ok_or(pyo3::exceptions::PyException::new_err(""))?,
            )?,
        )?)
    }
}

#[pymethods]
impl SerializedFile {
    #[staticmethod]
    pub fn read(cabfile: Vec<u8>) -> PyResult<Self> {
        let cabfile_reader = Box::new(Cursor::new(cabfile));
        Ok(SerializedFile(
            io_unity::SerializedFile::read(cabfile_reader)
                .or(Err(pyo3::exceptions::PyException::new_err("")))?,
        ))
    }

    pub fn get_object_count(&self) -> i32 {
        self.0.get_object_count()
    }

    pub fn get_raw_object_by_index(&self, py: Python, index: u32) -> PyResult<PyObject> {
        let obj = match self
            .0
            .get_object_by_index(index)
            .ok_or(pyo3::exceptions::PyException::new_err(""))?
        {
            io_unity::classes::Class::AssetBundle(ab) => AssetBundle(ab).into_py(py),
            io_unity::classes::Class::AudioClip(ac) => AudioClip(ac).into_py(py),
            io_unity::classes::Class::Texture2D(tex) => Texture2D(tex).into_py(py),
            io_unity::classes::Class::Mesh(mesh) => Mesh(mesh).into_py(py),
            _ => return Err(pyo3::exceptions::PyException::new_err("")),
        };
        Ok(obj)
    }

    pub fn get_object_by_path_id(&self, py: Python, path_id: i64) -> PyResult<PyObject> {
        #[macro_export]
        macro_rules! cov_python_unity_class {
            ($($x:ident($y:path)),+) => {
                match self
                    .0
                    .get_object_by_path_id(path_id)
                    .ok_or(pyo3::exceptions::PyException::new_err(""))?
                {
                    $(io_unity::classes::Class::$x(o) => $x(o).into_py(py),)+
                }
            };
        }

        let obj = cov_python_unity_class!(
            AssetBundle(asset_bundle::AssetBundle),
            AudioClip(audio_clip::AudioClip),
            Texture2D(texture_2d::Texture2D),
            Mesh(mesh::Mesh),
            Transform(transform::Transform),
            GameObject(game_object::GameObject),
            AnimationClip(animation_clip::AnimationClip),
            SkinnedMeshRenderer(skinned_mesh_renderer::SkinnedMeshRenderer),
            MeshRenderer(mesh_renderer::MeshRenderer),
            Material(material::Material),
            MeshFilter(mesh_filter::MeshFilter),
            MonoBehaviour(mono_behaviour::MonoBehaviour),
            MonoScript(mono_script::MonoScript),
            Animator(animator::Animator),
            Avatar(avatar::Avatar)
        );

        Ok(obj)
    }
}

#[pymethods]
impl Mesh {
    fn get_index_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<u32>> {
        Ok(self.0.get_index_buff(sub_mesh_id))
    }

    fn get_vertex_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<f32>> {
        Ok(self.0.get_vertex_buff(sub_mesh_id))
    }

    fn get_normal_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<f32>> {
        Ok(self.0.get_normal_buff(sub_mesh_id))
    }

    fn get_uv0_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<f32>> {
        Ok(self.0.get_uv0_buff(sub_mesh_id))
    }

    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<(Vec<f32>, Vec<u32>)>> {
        Ok(self
            .0
            .get_bone_weights_buff(sub_mesh_id)
            .into_iter()
            .map(|w| (w.weight, w.bone_index))
            .collect())
    }

    fn get_sub_mesh_count(&self) -> PyResult<usize> {
        Ok(self.0.get_sub_mesh_count())
    }

    fn get_bind_pose(&self) -> PyResult<Vec<[[f32; 4]; 4]>> {
        Ok(self
            .0
            .get_bind_pose()
            .into_iter()
            .map(|m| m.to_cols_array_2d())
            .collect())
    }
}

#[pymethods]
impl AudioClip {
    fn get_audio_data(&self, py: Python, fs: &PyCell<UnityFS>) -> PyResult<PyObject> {
        let mut fs = Box::new(fs.try_borrow_mut()?.0.clone()) as Box<dyn io_unity::FS>;
        let data = self.0.get_audio_data(&mut fs)?;
        Ok(PyBytes::new(py, &data).into())
    }

    fn get_name(&self) -> String {
        self.0.get_name()
    }
}

#[pymethods]
impl SkinnedMeshRenderer {
    fn get_mesh(&self, py: Python, sf: &PyCell<SerializedFile>) -> PyResult<PyObject> {
        if let Some(io_unity::classes::Class::Mesh(mesh)) = sf
            .try_borrow()?
            .0
            .get_object_by_path_id(self.0.get_mesh().get_path_id())
        {
            Ok(Mesh(mesh).into_py(py))
        } else {
            Err(pyo3::exceptions::PyException::new_err(""))
        }
    }

    fn get_bone_name_index_local_mat_buff(
        &self,
        sf: &PyCell<SerializedFile>,
    ) -> PyResult<(Vec<String>, Vec<i32>, Vec<[[f32; 4]; 4]>)> {
        let mut bone_name_buff = Vec::new();
        let mut bone_father_index_buff = Vec::new();
        let mut bone_local_mat_buff = Vec::new();

        let bones = self.0.get_bones();

        for bone in &*bones {
            if let Some(io_unity::classes::Class::Transform(bone)) =
                sf.try_borrow()?.0.get_object_by_path_id(bone.get_path_id())
            {
                bone_name_buff.push(
                    if let Some(io_unity::classes::Class::GameObject(go)) = sf
                        .try_borrow()?
                        .0
                        .get_object_by_path_id(bone.get_game_object().get_path_id())
                    {
                        go.get_name().to_string()
                    } else {
                        "bone".to_string()
                    },
                );
                let father = bones.iter().enumerate().find(|(_index, itbone)| {
                    itbone.get_path_id() == bone.get_father().get_path_id()
                });
                bone_father_index_buff.push(father.and_then(|e| Some(e.0 as i32)).unwrap_or(-1));
                bone_local_mat_buff.push(bone.get_local_mat().to_cols_array_2d());
            }
        }
        Ok((bone_name_buff, bone_father_index_buff, bone_local_mat_buff))

        // for material in self.0.get_materials() {
        //     let material = s.get_object_by_path_id(material.get_path_id());
        //     println!("{:#?}", material);
        // }
    }
}

#[pymodule]
fn io_unity_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UnityFS>()?;
    m.add_class::<SerializedFile>()?;

    #[macro_export]
    macro_rules! add_python_unity_class {
        ($x:ident($y:path)) => {
            m.add_class::<$x>()?;
        };
        ($($xx:ident($yy:path)),+) => {
            $(add_python_unity_class!($xx($yy));)+
        };
    }

    add_python_unity_class!(
        AssetBundle(asset_bundle::AssetBundle),
        AudioClip(audio_clip::AudioClip),
        Texture2D(texture_2d::Texture2D),
        Mesh(mesh::Mesh),
        Transform(transform::Transform),
        GameObject(game_object::GameObject),
        AnimationClip(animation_clip::AnimationClip),
        SkinnedMeshRenderer(skinned_mesh_renderer::SkinnedMeshRenderer),
        MeshRenderer(mesh_renderer::MeshRenderer),
        Material(material::Material),
        MeshFilter(mesh_filter::MeshFilter),
        MonoBehaviour(mono_behaviour::MonoBehaviour),
        MonoScript(mono_script::MonoScript),
        Animator(animator::Animator),
        Avatar(avatar::Avatar)
    );

    Ok(())
}
