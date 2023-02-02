#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, Cursor},
    sync::{Arc, Mutex},
};

use io_unity::type_tree::convert::TryCastFrom;
use tauri::{api::dialog, AboutMetadata, CustomMenuItem, Menu, MenuItem, Submenu};

struct IOUnityContext {
    fs: Mutex<HashMap<String, io_unity::UnityFS>>,
    cabs: Mutex<HashMap<String, io_unity::SerializedFile>>,
    objects: Mutex<HashMap<String, io_unity::type_tree::TypeTreeObject>>,
}

impl IOUnityContext {
    fn new(
        fs: HashMap<String, io_unity::UnityFS>,
        cabs: HashMap<String, io_unity::SerializedFile>,
        objects: HashMap<String, io_unity::type_tree::TypeTreeObject>,
    ) -> Self {
        Self {
            fs: Mutex::new(fs),
            cabs: Mutex::new(cabs),
            objects: Mutex::new(objects),
        }
    }
}

#[derive(serde::Serialize)]
struct Object {
    path_id: String,
    tp: String,
    name: String,
}

#[tauri::command]
fn list_fs(state: tauri::State<IOUnityContext>) -> Result<Vec<String>, ()> {
    if let Ok(op) = state.fs.lock() {
        let keys = op.keys().into_iter().map(|s| s.to_owned()).collect();
        return Ok(keys);
    }
    Err(())
}

#[tauri::command]
fn open_fs(state: tauri::State<IOUnityContext>, fs_path: &str) -> Result<String, ()> {
    if let Ok(mut op) = state.fs.try_lock() {
        let file = OpenOptions::new().read(true).open(fs_path).unwrap();
        let file = BufReader::new(file);
        let fs = io_unity::UnityFS::read(Box::new(file), None).unwrap();
        op.insert(fs_path.to_string(), fs);
        return Ok(fs_path.to_string());
    }
    Err(())
}

#[tauri::command]
fn list_fs_path(state: tauri::State<IOUnityContext>, fs_path: &str) -> Result<Vec<String>, ()> {
    if let Ok(op) = state.fs.try_lock() {
        if let Some(fs) = op.get(fs_path) {
            return Ok(fs.get_file_paths());
        }
    }
    Err(())
}

#[tauri::command]
fn list_fs_cab(state: tauri::State<IOUnityContext>, fs_path: &str) -> Result<Vec<Object>, ()> {
    if let Ok(cabs) = state.cabs.try_lock() {
        if let Some(cab) = cabs.get(fs_path) {
            let mut objects = vec![];
            for (pathid, obj) in cab.get_object_map() {
                let tt_o = cab.get_tt_object_by_path_id(*pathid).unwrap();
                let name =
                    String::try_cast_from(&tt_o.unwrap(), "/Base/m_Name").unwrap_or("".to_owned());
                objects.push(Object {
                    path_id: pathid.to_string(),
                    tp: format!("{:?}", &obj.class),
                    name,
                });
            }
            return Ok(objects);
        }
    }
    if let Ok(op) = state.fs.try_lock() {
        if let Some(fs) = op.get(fs_path) {
            if let Some(cabfile_path) = fs.get_cab_path().get(0) {
                let cabfile = fs.get_file_data_by_path(cabfile_path).unwrap();
                let cabfile_reader = Box::new(Cursor::new(cabfile));
                let cab = io_unity::SerializedFile::read(cabfile_reader, 0, None).unwrap();
                let mut objects = vec![];
                for (pathid, obj) in cab.get_object_map() {
                    let tt_o = cab.get_tt_object_by_path_id(*pathid).unwrap();
                    let name = String::try_cast_from(&tt_o.unwrap(), "/Base/m_Name")
                        .unwrap_or("".to_owned());
                    objects.push(Object {
                        path_id: pathid.to_string(),
                        tp: format!("{:?}", &obj.class),
                        name,
                    });
                }
                state.cabs.lock().unwrap().insert(fs_path.to_owned(), cab);
                return Ok(objects);
            }
        }
    }
    Err(())
}

fn main() {
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let file_menu = Submenu::new(
        "File",
        Menu::new().add_item(open).add_native_item(MenuItem::Quit),
    );
    let help_menu = Submenu::new(
        "Help",
        Menu::new().add_native_item(MenuItem::About(
            "io_unity".to_owned(),
            AboutMetadata::new()
                .version("0.1.0")
                .website("https://github.com/gameltb/io_unity")
                .authors(vec!["gamegccltb".to_owned()]),
        )),
    );

    let menu = Menu::new().add_submenu(file_menu).add_submenu(help_menu);

    let io_unity_context = IOUnityContext::new(HashMap::new(), HashMap::new(), HashMap::new());

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "open" => {
                dialog::FileDialogBuilder::default().pick_file(move |path_buf| {
                    if let Some(path) = &path_buf {
                        let _ = event
                            .window()
                            .emit("menu-open-event", path.to_string_lossy())
                            .unwrap();
                    }
                });
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            list_fs,
            open_fs,
            list_fs_path,
            list_fs_cab
        ])
        .manage(io_unity_context)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
