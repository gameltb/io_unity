use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::Path,
    sync::{Arc, RwLock},
};

use io_unity::type_tree::{
    convert::{FieldCastArgs, TryCast, TryCastRef},
    Field,
};
use pyo3::{exceptions::PyAttributeError, prelude::*};

pub mod python_unity_class {

    use std::sync::{Arc, RwLock};

    use pyo3::prelude::*;

    #[macro_export]
    macro_rules! def_python_unity_class {
        ($x:ident($y:path)) => {
            #[pyclass]
            pub struct $x(pub io_unity::type_tree::TypeTreeObject);
        };
        ($($xx:ident($yy:path)),+) => {

            $(def_python_unity_class!($xx($yy));)+

        };
    }

    def_python_unity_class!(
        AudioClip(audio_clip::AudioClip),
        Texture2D(texture2d::Texture2D),
        Mesh(mesh::Mesh),
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
    pub struct TypeTreeObject {
        pub inner: Arc<RwLock<Box<io_unity::type_tree::TypeTreeObject>>>,
        pub path: String,
    }
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

#[pyfunction]
fn set_info_json_tar_reader(path: String) -> PyResult<()> {
    let file = read_file(path).into_py_result()?;
    io_unity::type_tree::type_tree_json::set_info_json_tar_reader(file);
    Ok(())
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
                .map(|otto| {
                    otto.map(|tto| TypeTreeObject {
                        inner: Arc::new(RwLock::new(Box::new(tto))),
                        path: "/Base".to_owned(),
                    })
                })
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
                    class_id: obj.class,
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
        self.inner.read().unwrap().class_id
    }

    fn display_tree(&self) {
        self.inner.read().unwrap().display_tree();
    }

    fn get_data_buff(&self) -> Option<Vec<u8>> {
        let inner = self.inner.read().unwrap();
        let field = inner.get_field_by_path(&self.path)?;
        Some(
            field
                .try_cast_as(&inner.get_field_cast_args())
                .ok()?
                .to_owned(),
        )
    }

    fn __getattr__(&self, py: Python<'_>, attr: &str) -> PyResult<PyObject> {
        let inner = self.inner.read().unwrap();

        let field_cast_args = inner.get_field_cast_args();
        let path = self.path.clone() + "/" + attr;
        let field = inner
            .get_field_by_path(&path)
            .ok_or(PyAttributeError::new_err(format!(
                "field {} cannot found, Path : {}",
                attr, &path,
            )))?;

        fn cast_field(
            type_tree_object: &Arc<RwLock<Box<io_unity::type_tree::TypeTreeObject>>>,
            py: Python<'_>,
            field: &Field,
            field_cast_args: &FieldCastArgs,
            path: &str,
            should_clone_field: bool,
        ) -> PyResult<PyObject> {
            let cast_error_map = |_| {
                PyAttributeError::new_err(format!(
                    "field {} cast failed. Type: {}",
                    path,
                    field.get_type().as_str()
                ))
            };

            match field.get_type().as_str() {
                "string" => {
                    let value: String = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "bool" => {
                    let value: bool = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "SInt8" => {
                    let value: i8 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "SInt16" | "short" => {
                    let value: i16 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "SInt32" | "int" => {
                    let value: i32 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "SInt64" | "long long" => {
                    let value: i64 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "UInt8" | "char" => {
                    let value: u8 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "UInt16" | "unsigned short" => {
                    let value: u16 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "UInt32" | "unsigned int" => {
                    let value: u32 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "UInt64" | "unsigned long long" | "FileSize" => {
                    let value: u64 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "float" => {
                    let value: f32 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "double" => {
                    let value: f64 = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    return Ok(value.into_py(py));
                }
                "vector" | "staticvector" => {
                    let field = field.get_field(&["Array".to_string()]).ok_or(
                        PyAttributeError::new_err(format!(
                            "Array field {} cast failed. Type: {}",
                            path,
                            field.get_type().as_str()
                        )),
                    )?;
                    if let Some((buff_type, size)) = field.try_get_buff_type_and_type_size() {
                        match buff_type.as_str() {
                            "float" => {
                                let value: Vec<f32> = field
                                    .try_cast_to(&field_cast_args)
                                    .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            "double" => {
                                let value: Vec<f64> = field
                                    .try_cast_to(&field_cast_args)
                                    .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            &_ => (),
                        }

                        match size {
                            1 => {
                                let value: &Vec<u8> = field
                                    .try_cast_as(&field_cast_args)
                                    .map_err(cast_error_map)?;
                                return Ok(value.to_owned().into_py(py));
                            }
                            2 => {
                                let value: Vec<u16> = field
                                    .try_cast_to(&field_cast_args)
                                    .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            4 => {
                                let value: Vec<u32> = field
                                    .try_cast_to(&field_cast_args)
                                    .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            8 => {
                                let value: Vec<u64> = field
                                    .try_cast_to(&field_cast_args)
                                    .map_err(cast_error_map)?;
                                return Ok(value.into_py(py));
                            }
                            _ => (),
                        }

                        return Err(PyAttributeError::new_err(format!(
                            "Array field {} cannot cast. Type: {} Item Type : {}",
                            path,
                            field.get_type().as_str(),
                            buff_type
                        )));
                    }
                    let value: Vec<io_unity::type_tree::TypeTreeObject> = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;
                    let mut new_vec = Vec::new();
                    for obj in value {
                        let value = obj.get_field_by_name("").unwrap();
                        let value =
                            cast_field(type_tree_object, py, value, field_cast_args, path, true)?;
                        new_vec.push(value)
                    }
                    return Ok(new_vec.into_py(py));
                }
                "map" => {
                    let field = field.get_field(&["Array".to_string()]).ok_or(
                        PyAttributeError::new_err(format!(
                            "Map field {} cast failed. Type: {}",
                            path,
                            field.get_type().as_str()
                        )),
                    )?;
                    let value: HashMap<String, io_unity::type_tree::TypeTreeObject> = field
                        .try_cast_to(&field_cast_args)
                        .map_err(cast_error_map)?;

                    let mut new_map = HashMap::new();
                    for (name, obj) in value {
                        let value = obj.get_field_by_name("").unwrap();
                        let value =
                            cast_field(type_tree_object, py, value, field_cast_args, path, true)?;
                        new_map.insert(name, value);
                    }
                    return Ok(new_map.into_py(py));
                }
                &_ => {
                    let value = if should_clone_field {
                        let value: io_unity::type_tree::TypeTreeObject = field
                            .try_cast_to(&field_cast_args)
                            .map_err(cast_error_map)?;
                        TypeTreeObject {
                            inner: Arc::new(RwLock::new(Box::new(value))),
                            path: "/Base".to_owned(),
                        }
                    } else {
                        TypeTreeObject {
                            inner: type_tree_object.clone(),
                            path: path.to_string(),
                        }
                    };

                    return Ok(value.into_py(py));
                }
            }
        }

        cast_field(&self.inner, py, field, &field_cast_args, &path, false)
    }
}

#[pymethods]
impl Mesh {}

#[pymethods]
impl AudioClip {}

#[pymodule]
fn io_unity_python(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<UnityFS>()?;
    m.add_class::<SerializedFile>()?;
    m.add_class::<UnityAssetViewer>()?;
    m.add_class::<TypeTreeObject>()?;
    m.add_class::<ObjectRef>()?;
    m.add_function(wrap_pyfunction!(set_info_json_tar_reader, m)?)?;

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
        Transform(transform::Transform),
        AnimationClip(animation_clip::AnimationClip)
    );

    Ok(())
}
