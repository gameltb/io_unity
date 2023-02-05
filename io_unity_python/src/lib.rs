use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufReader,
    path::Path,
};

use io_unity::{
    classes::{
        audio_clip::AudioClipObject, mesh::MeshObject, p_ptr::PPtrObject,
        texture2d::Texture2DObject,
    },
    type_tree::convert::TryCastFrom,
};

use pyo3::{exceptions::PyAttributeError, prelude::*, types::PyBytes};

pub mod python_unity_class {

    use pyo3::prelude::*;

    #[macro_export]
    macro_rules! def_python_unity_class {
        ($x:ident($y:path)) => {
            #[pyclass]
            pub struct $x(pub io_unity::type_tree::TypeTreeObjectRef);
        };
        ($($xx:ident($yy:path)),+) => {

            $(def_python_unity_class!($xx($yy));)+

        };
    }

    def_python_unity_class!(
        AudioClip(audio_clip::AudioClip),
        Texture2D(texture2d::Texture2D),
        Mesh(mesh::Mesh),
        PPtr(p_ptr::PPtr),
        Transform(transform::Transform),
        AnimationClip(animation_clip::AnimationClip)
    );

    #[pyclass]
    pub struct UnityFS(pub io_unity::UnityFS);
    #[pyclass]
    pub struct SerializedFile(pub io_unity::SerializedFile);
    #[pyclass]
    pub struct UnityAssetViewer(pub io_unity::unity_asset_view::UnityAssetViewer);
    #[pyclass]
    pub struct TypeTreeObjectRef(pub io_unity::type_tree::TypeTreeObjectRef);
}

use python_unity_class::*;

trait IntoPyResult<T> {
    fn into_py_result(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for Result<T, anyhow::Error> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e| {
            pyo3::exceptions::PyException::new_err(format!("{}\n{}", e, e.backtrace()))
        })
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
    class_id: i32,
}

#[pymethods]
impl ObjectRef {
    #[new]
    fn new(serialized_file_id: i64, path_id: i64) -> Self {
        ObjectRef {
            serialized_file_id,
            path_id,
            class_id: 0,
        }
    }

    fn get_class_id(&self) -> i32 {
        self.class_id
    }
}

#[pyclass]
pub struct ObjectRefIter {
    inner: std::vec::IntoIter<ObjectRef>,
}

#[pymethods]
impl ObjectRefIter {
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

#[pyfunction]
fn set_info_json_tar_reader(path: String) -> PyResult<()> {
    let file = read_file(path).into_py_result()?;
    io_unity::type_tree::type_tree_json::set_info_json_tar_reader(file);
    Ok(())
}

#[pyfunction]
fn get_bone_path_hash_map(
    viewer: &UnityAssetViewer,
    transform: &TypeTreeObjectRef,
) -> PyResult<BTreeMap<u32, String>> {
    let transform = io_unity::classes::transform::Transform::new(&transform.0);
    io_unity::classes::transform::get_bone_path_hash_map(&viewer.0, &transform).into_py_result()
}

#[pyfunction]
fn get_root_bone(
    viewer: &UnityAssetViewer,
    transform: &TypeTreeObjectRef,
) -> PyResult<TypeTreeObjectRef> {
    let transform = io_unity::classes::transform::Transform::new(&transform.0);
    Ok(TypeTreeObjectRef(io_unity::classes::transform::get_root_bone(&viewer.0, &transform).into_py_result()?))
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

    pub fn deref_object_ref(&self, object_ref: &ObjectRef) -> PyResult<Option<TypeTreeObjectRef>> {
        if let Some(serialized_file) = self
            .0
            .serialized_file_map
            .get(&object_ref.serialized_file_id)
        {
            return serialized_file
                .get_tt_object_by_path_id(object_ref.path_id)
                .map(|otto| otto.map(|tto| TypeTreeObjectRef(tto.into())))
                .into_py_result();
        }
        Ok(None)
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<ObjectRefIter>> {
        let mut obj_vec = Vec::new();

        for (serialized_file_id, sf) in &slf.0.serialized_file_map {
            for (path_id, obj) in sf.get_object_map() {
                obj_vec.push(ObjectRef {
                    serialized_file_id: *serialized_file_id,
                    path_id: *path_id,
                    class_id: obj.class,
                })
            }
        }
        let iter = ObjectRefIter {
            inner: obj_vec.into_iter(),
        };
        Py::new(slf.py(), iter)
    }

    fn get_container_name_by_object_ref(&self, object_ref: &ObjectRef) -> Option<String> {
        self.0
            .get_container_name_by_serialized_file_id_and_path_id(
                object_ref.serialized_file_id,
                object_ref.path_id,
            )
            .map(|s| s.to_owned())
    }
}

#[pymethods]
impl TypeTreeObjectRef {
    fn get_class_id(&self) -> i32 {
        self.0.get_class_id()
    }

    fn display_tree(&self) {
        self.0.display_tree();
    }

    fn get_data_buff(&self) -> Option<Vec<u8>> {
        let path_to_self: Vec<String> = Vec::new();
        <Vec<u8>>::try_cast_from(&self.0, path_to_self.as_slice()).ok()
    }

    fn get_container_name(&self, viewer: &UnityAssetViewer) -> Option<String> {
        viewer
            .0
            .get_container_name_by_serialized_file_id_and_path_id(
                self.0.get_serialized_file_id(),
                self.0.get_path_id(),
            )
            .map(|s| s.to_owned())
    }

    fn __getattr__(&self, py: Python<'_>, attr: &str) -> PyResult<PyObject> {
        let path = vec![attr.to_owned()];
        let field = io_unity::type_tree::TypeTreeObjectRef::try_cast_from(&self.0, path.as_slice())
            .map_err(|_| {
                PyAttributeError::new_err(format!(
                    "field {} cannot found, Path : {:?}",
                    attr, &path,
                ))
            })?;

        fn cast_field(
            field: io_unity::type_tree::TypeTreeObjectRef,
            py: Python<'_>,
        ) -> PyResult<PyObject> {
            let cast_error_map = |_| {
                PyAttributeError::new_err(format!(
                    "field {:?} cast failed. Type: {:?}",
                    field.path,
                    field.get_type()
                ))
            };
            let path_to_self: Vec<String> = Vec::new();

            match field
                .get_type()
                .ok_or(PyAttributeError::new_err(format!(
                    "field {:?} cast failed. Type: {:?}",
                    field.path,
                    field.get_type()
                )))?
                .as_str()
            {
                "string" => {
                    let value = <String>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "bool" => {
                    let value = <bool>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "SInt8" => {
                    let value = <i8>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "SInt16" | "short" => {
                    let value = <i16>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "SInt32" | "int" => {
                    let value = <i32>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "SInt64" | "long long" => {
                    let value = <i64>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "UInt8" | "char" => {
                    let value = <u8>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "UInt16" | "unsigned short" => {
                    let value = <u16>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "UInt32" | "unsigned int" => {
                    let value = <u32>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "UInt64" | "unsigned long long" => {
                    let value = <u64>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "FileSize" => {
                    let value = <usize>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "float" => {
                    let value = <f32>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "double" => {
                    let value = <f64>::try_cast_from(&field, path_to_self.as_slice())
                        .map_err(cast_error_map)?;
                    Ok(value.into_py(py))
                }
                "vector" | "staticvector" => {
                    let field = io_unity::type_tree::TypeTreeObjectRef::try_cast_from(
                        &field,
                        ["Array".to_owned()].as_slice(),
                    )
                    .map_err(|_| {
                        PyAttributeError::new_err(format!(
                            "Array field {:?} cast failed. Type: {:?}",
                            field.path,
                            field.get_type()
                        ))
                    })?;
                    if let Some((buff_type, size)) = field.try_get_buff_type_and_type_size() {
                        match buff_type.as_str() {
                            "float" => {
                                let value =
                                    <Vec<f32>>::try_cast_from(&field, path_to_self.as_slice())
                                        .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            "double" => {
                                let value =
                                    <Vec<f64>>::try_cast_from(&field, path_to_self.as_slice())
                                        .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            &_ => (),
                        }

                        match size {
                            1 => {
                                let value =
                                    <Vec<u8>>::try_cast_from(&field, path_to_self.as_slice())
                                        .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            2 => {
                                let value =
                                    <Vec<u16>>::try_cast_from(&field, path_to_self.as_slice())
                                        .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            4 => {
                                let value =
                                    <Vec<u32>>::try_cast_from(&field, path_to_self.as_slice())
                                        .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            8 => {
                                let value =
                                    <Vec<u64>>::try_cast_from(&field, path_to_self.as_slice())
                                        .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            _ => (),
                        }

                        return Err(PyAttributeError::new_err(format!(
                            "Array field {:?} cannot cast. Type: {:?} Item Type : {}",
                            field.path,
                            field.get_type(),
                            buff_type
                        )));
                    }
                    let value = <Vec<io_unity::type_tree::TypeTreeObjectRef>>::try_cast_from(
                        &field,
                        path_to_self.as_slice(),
                    )
                    .map_err(cast_error_map)?;
                    let mut new_vec = Vec::new();
                    for obj in value {
                        let value = cast_field(obj, py)?;
                        new_vec.push(value)
                    }
                    Ok(new_vec.into_py(py))
                }
                "map" => {
                    let field = io_unity::type_tree::TypeTreeObjectRef::try_cast_from(
                        &field,
                        ["Array".to_owned()].as_slice(),
                    )
                    .map_err(|_| {
                        PyAttributeError::new_err(format!(
                            "Map field {:?} cast failed. Type: {:?}",
                            field.path,
                            field.get_type()
                        ))
                    })?;
                    let value =
                        <HashMap<String, io_unity::type_tree::TypeTreeObjectRef>>::try_cast_from(
                            &field,
                            path_to_self.as_slice(),
                        )
                        .map_err(cast_error_map)?;

                    let mut new_map = HashMap::new();
                    for (name, obj) in value {
                        let value = cast_field(obj, py)?;
                        new_map.insert(name, value);
                    }
                    Ok(new_map.into_py(py))
                }
                &_ => {
                    let value = TypeTreeObjectRef(field);
                    Ok(value.into_py(py))
                }
            }
        }

        cast_field(field, py)
    }
}

#[pymethods]
impl PPtr {
    #[new]
    fn new(obj: &TypeTreeObjectRef) -> Self {
        PPtr(obj.0.clone())
    }

    fn get_type_tree_object_in_view(
        &self,
        viewer: &UnityAssetViewer,
    ) -> PyResult<Option<TypeTreeObjectRef>> {
        let pptr = io_unity::classes::p_ptr::PPtr::new(&self.0);
        Ok(pptr
            .get_type_tree_object_in_view(&viewer.0)
            .into_py_result()?
            .map(|obj| TypeTreeObjectRef(obj.into())))
    }
}

#[pymethods]
impl Mesh {
    #[new]
    fn new(obj: &TypeTreeObjectRef) -> Self {
        Mesh(obj.0.clone())
    }

    fn get_sub_mesh_count(&self) -> PyResult<usize> {
        let mesh = io_unity::classes::mesh::Mesh::new(&self.0);
        mesh.get_sub_mesh_count().into_py_result()
    }

    fn get_index_buff(&self, sub_mesh_index: usize) -> PyResult<Vec<u32>> {
        let mesh = io_unity::classes::mesh::Mesh::new(&self.0);
        mesh.get_index_buff(sub_mesh_index).into_py_result()
    }

    fn get_vertex_buff(&self, sub_mesh_index: usize) -> PyResult<Vec<f32>> {
        let mesh = io_unity::classes::mesh::Mesh::new(&self.0);
        mesh.get_vertex_buff(sub_mesh_index).into_py_result()
    }

    fn get_normal_buff(&self, sub_mesh_index: usize) -> PyResult<Vec<f32>> {
        let mesh = io_unity::classes::mesh::Mesh::new(&self.0);
        mesh.get_normal_buff(sub_mesh_index).into_py_result()
    }

    fn get_uv0_buff(&self, sub_mesh_index: usize) -> PyResult<Vec<f32>> {
        let mesh = io_unity::classes::mesh::Mesh::new(&self.0);
        mesh.get_uv0_buff(sub_mesh_index).into_py_result()
    }

    fn get_bone_weights_buff(&self, sub_mesh_index: usize) -> PyResult<Vec<(Vec<f32>, Vec<i64>)>> {
        let mesh = io_unity::classes::mesh::Mesh::new(&self.0);
        Ok(mesh
            .get_bone_weights_buff(sub_mesh_index)
            .into_py_result()?
            .into_iter()
            .map(|w| (w.weight, w.bone_index))
            .collect())
    }
}

#[pymethods]
impl AudioClip {
    #[new]
    fn new(obj: &TypeTreeObjectRef) -> Self {
        AudioClip(obj.0.clone())
    }

    fn get_audio_data(&self, py: Python<'_>, viewer: &UnityAssetViewer) -> PyResult<PyObject> {
        let audio_clip = io_unity::classes::audio_clip::AudioClip::new(&self.0);
        audio_clip
            .get_audio_data(&viewer.0)
            .map(|data| PyBytes::new(py, &data).into())
            .into_py_result()
    }
}

#[pymethods]
impl Texture2D {
    #[new]
    fn new(obj: &TypeTreeObjectRef) -> Self {
        Texture2D(obj.0.clone())
    }

    fn save_image(&self, viewer: &UnityAssetViewer, file_path: String) -> PyResult<()> {
        let texture = io_unity::classes::texture2d::Texture2D::new(&self.0);
        texture
            .get_image(&viewer.0)
            .map(|dynimg| dynimg.flipv().save(file_path))
            .into_py_result()?
            .map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))
    }
}

#[pymodule]
fn io_unity_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UnityFS>()?;
    m.add_class::<SerializedFile>()?;
    m.add_class::<UnityAssetViewer>()?;
    m.add_class::<TypeTreeObjectRef>()?;
    m.add_class::<ObjectRef>()?;
    m.add_function(wrap_pyfunction!(set_info_json_tar_reader, m)?)?;
    m.add_function(wrap_pyfunction!(get_bone_path_hash_map, m)?)?;
    m.add_function(wrap_pyfunction!(get_root_bone, m)?)?;

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
        AudioClip(audio_clip::AudioClip),
        Texture2D(texture2d::Texture2D),
        Mesh(mesh::Mesh),
        PPtr(p_ptr::PPtr),
        Transform(transform::Transform),
        AnimationClip(animation_clip::AnimationClip)
    );

    Ok(())
}
