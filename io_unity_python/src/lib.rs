use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use io_unity::type_tree::convert::{TryCast, TryCastFrom};
use pyo3::{exceptions::PyAttributeError, prelude::*};

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
    #[pyclass]
    pub struct UnityAssetViewer(pub io_unity::unity_asset_view::UnityAssetViewer);
    #[pyclass]
    pub struct TypeTreeObject(pub io_unity::type_tree::TypeTreeObject);
}

use python_unity_class::*;

trait IntoPyResult<T> {
    fn into_py_result(self) -> PyResult<T>;
}

impl<T, E> IntoPyResult<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))
    }
}

impl<T> IntoPyResult<T> for Option<T> {
    fn into_py_result(self) -> PyResult<T> {
        self.ok_or(pyo3::exceptions::PyException::new_err("Value is None"))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ObjectRef {
    serialized_file_id: i64,
    path_id: i64,
    class_id_type: io_unity::classes::ClassIDType,
}

#[pyclass]
pub struct Iter {
    inner: std::vec::IntoIter<ObjectRef>,
}

#[pymethods]
impl Iter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<ObjectRef> {
        slf.inner.next()
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Box<BufReader<File>>> {
    let file = File::open(path)?;
    Ok(Box::new(BufReader::new(file)))
}

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
            io_unity::UnityFS::read(Box::new(file), file_path).into_py_result()?,
        ))
    }

    pub fn get_cab(&self) -> PyResult<SerializedFile> {
        Ok(SerializedFile::read(self.0.get_file_by_path(
            self.0.get_cab_path().get(0).into_py_result()?,
        )?)?)
    }
}

#[pymethods]
impl SerializedFile {
    #[staticmethod]
    pub fn read(cabfile: Vec<u8>) -> PyResult<Self> {
        let cabfile_reader = Box::new(Cursor::new(cabfile));
        Ok(SerializedFile(
            io_unity::SerializedFile::read(cabfile_reader, 0, None).into_py_result()?,
        ))
    }

    pub fn get_object_count(&self) -> i32 {
        self.0.get_object_count()
    }

    pub fn get_object_by_path_id(&self, py: Python, path_id: i64) -> PyResult<PyObject> {
        #[macro_export]
        macro_rules! cov_python_unity_class {
            ($($x:ident($y:path)),+) => {
                match self
                    .0
                    .get_object_by_path_id(path_id)
                    .into_py_result()?
                    .into_py_result()?
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
impl UnityAssetViewer {
    #[new]
    fn new() -> Self {
        UnityAssetViewer(io_unity::unity_asset_view::UnityAssetViewer::new())
    }

    pub fn read_bundle_dir(&mut self, path: String) -> PyResult<()> {
        self.0.read_bundle_dir(path).into_py_result()
    }

    pub fn read_data_dir(&mut self, path: String) -> PyResult<()> {
        self.0.read_data_dir(path).into_py_result()
    }

    pub fn add_bundle_file(
        &mut self,
        path: String,
        resource_search_path: Option<String>,
    ) -> PyResult<i64> {
        self.0
            .add_bundle_file(read_file(path).into_py_result()?, resource_search_path)
            .into_py_result()
    }

    pub fn add_serialized_file(
        &mut self,
        path: String,
        resource_search_path: Option<String>,
    ) -> PyResult<i64> {
        self.0
            .add_serialized_file(read_file(path).into_py_result()?, resource_search_path)
            .into_py_result()
    }

    pub fn deref_object_ref(&self, object_ref: ObjectRef) -> PyResult<Option<TypeTreeObject>> {
        if let Some(serialized_file) = self
            .0
            .serialized_file_map
            .get(&object_ref.serialized_file_id)
        {
            return serialized_file
                .get_tt_object_by_path_id(object_ref.path_id)
                .map(|otto| otto.map(|tto| TypeTreeObject(tto)))
                .into_py_result();
        }
        Ok(None)
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
        let mut obj_vec = Vec::new();

        for (serialized_file_id, sf) in &slf.0.serialized_file_map {
            for (path_id, obj) in sf.get_object_map() {
                obj_vec.push(ObjectRef {
                    serialized_file_id: *serialized_file_id,
                    path_id: *path_id,
                    class_id_type: obj.class.clone(),
                })
            }
        }
        let iter = Iter {
            inner: obj_vec.into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

#[pymethods]
impl TypeTreeObject {
    fn get_class_id(&self) -> i32 {
        self.0.class_id
    }

    fn display_tree(&self) {
        self.0.display_tree();
    }

    fn __getattr__(&self, py: Python<'_>, attr: &str) -> PyResult<PyObject> {
        let field_cast_args = self.0.get_field_cast_args();
        let field = self
            .0
            .get_field_by_name(attr)
            .ok_or(PyAttributeError::new_err(format!(
                "field {} cannot found",
                attr,
            )))?;

        match field.get_type().as_str() {
            "string" => {
                let value: String = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "bool" => {
                let value: bool = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "SInt8" => {
                let value: i8 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "SInt16" | "short" => {
                let value: i16 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "SInt32" | "int" => {
                let value: i32 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "SInt64" | "long long" => {
                let value: i64 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "UInt8" | "char" => {
                let value: u8 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "UInt16" | "unsigned short" => {
                let value: u16 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "UInt32" | "unsigned int" => {
                let value: u32 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "UInt64" | "unsigned long long" | "FileSize" => {
                let value: u64 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "float" => {
                let value: f32 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            "double" => {
                let value: f64 = field.try_cast_to(&field_cast_args).map_err(|_| {
                    PyAttributeError::new_err(format!(
                        "field {} cast failed. Type: {}",
                        attr,
                        field.get_type().as_str()
                    ))
                })?;
                return Ok(value.into_py(py));
            }
            &_ => (),
        }

        Err(PyAttributeError::new_err(format!(
            "field {} cannot cast. Type: {}",
            attr,
            field.get_type().as_str()
        )))
    }
}

#[pymethods]
impl Mesh {
    fn get_index_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<u32>> {
        self.0.get_index_buff(sub_mesh_id).into_py_result()
    }

    fn get_vertex_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<f32>> {
        self.0.get_vertex_buff(sub_mesh_id).into_py_result()
    }

    fn get_normal_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<f32>> {
        self.0.get_normal_buff(sub_mesh_id).into_py_result()
    }

    fn get_uv0_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<f32>> {
        self.0.get_uv0_buff(sub_mesh_id).into_py_result()
    }

    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> PyResult<Vec<(Vec<f32>, Vec<u32>)>> {
        Ok(self
            .0
            .get_bone_weights_buff(sub_mesh_id)
            .into_py_result()?
            .into_iter()
            .map(|w| (w.weight, w.bone_index))
            .collect())
    }

    fn get_sub_mesh_count(&self) -> PyResult<usize> {
        self.0.get_sub_mesh_count().into_py_result()
    }

    fn get_bind_pose(&self) -> PyResult<Vec<[[f32; 4]; 4]>> {
        Ok(self
            .0
            .get_bind_pose()
            .into_py_result()?
            .iter()
            .map(|m| m.to_cols_array_2d())
            .collect())
    }
}

#[pymethods]
impl AudioClip {
    fn get_name(&self) -> String {
        self.0.downcast().get_name().unwrap()
    }
}

#[pymethods]
impl SkinnedMeshRenderer {
    fn get_mesh(&self, py: Python, sf: &PyCell<SerializedFile>) -> PyResult<PyObject> {
        if let Some(io_unity::classes::Class::Mesh(mesh)) = sf
            .try_borrow()?
            .0
            .get_object_by_path_id(
                self.0
                    .get_mesh()
                    .and_then(|m| m.get_path_id())
                    .into_py_result()?,
            )
            .into_py_result()?
        {
            Ok(Mesh(mesh).into_py(py))
        } else {
            Err(pyo3::exceptions::PyException::new_err(
                "Can't find object in the SerializedFile",
            ))
        }
    }

    fn get_bone_name_index_local_mat_buff(
        &self,
        sf: &PyCell<SerializedFile>,
    ) -> PyResult<(Vec<String>, Vec<i32>, Vec<[[f32; 4]; 4]>)> {
        let mut bone_name_buff = Vec::new();
        let mut bone_father_index_buff = Vec::new();
        let mut bone_local_mat_buff = Vec::new();

        let bones = self.0.get_bones().into_py_result()?;

        for bone in &*bones {
            if let Some(io_unity::classes::Class::Transform(bone)) = sf
                .try_borrow()?
                .0
                .get_object_by_path_id(bone.get_path_id().into_py_result()?)
                .into_py_result()?
            {
                bone_name_buff.push(
                    if let Some(io_unity::classes::Class::GameObject(go)) = sf
                        .try_borrow()?
                        .0
                        .get_object_by_path_id(
                            bone.downcast()
                                .get_game_object()
                                .and_then(|go| go.get_path_id())
                                .into_py_result()?,
                        )
                        .into_py_result()?
                    {
                        go.get_name().into_py_result()?.to_string()
                    } else {
                        "bone".to_string()
                    },
                );
                let father = bones.iter().enumerate().find(|(_index, itbone)| {
                    if let (Some(itboneid), Some(boneid)) = (
                        itbone.get_path_id(),
                        bone.get_father().and_then(|f| f.get_path_id()),
                    ) {
                        itboneid == boneid
                    } else {
                        false
                    }
                });
                bone_father_index_buff.push(father.and_then(|e| Some(e.0 as i32)).unwrap_or(-1));
                bone_local_mat_buff.push(bone.get_local_mat().into_py_result()?.to_cols_array_2d());
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
    m.add_class::<UnityAssetViewer>()?;
    m.add_class::<TypeTreeObject>()?;

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
