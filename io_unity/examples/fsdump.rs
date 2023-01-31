extern crate io_unity;
#[macro_use]
extern crate anyhow;

use clap::{arg, Parser, Subcommand};
use io_unity::classes::p_ptr::{PPtr, PPtrObject};
use io_unity::classes::texture2d::{Texture2D, Texture2DObject};
use io_unity::type_tree::convert::TryCastFrom;
use io_unity::type_tree::{TypeTreeObject, TypeTreeObjectRef};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use io_unity::{
    classes::ClassIDType, type_tree::type_tree_json::set_info_json_tar_reader,
    unity_asset_view::UnityAssetViewer,
};

use io_unity::*;

/// unity extractor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The dir contain AssetBundle files.
    #[arg(short, long)]
    bundle_dir: Option<String>,
    /// The dir contain data files.
    #[arg(short, long)]
    data_dir: Option<String>,
    /// The serialized file.
    #[arg(short, long)]
    serialized_file: Option<String>,
    /// The tar zstd compressed file contain type tree info json files
    /// for read file without typetree info.
    /// see https://github.com/DaZombieKiller/TypeTreeDumper
    /// aslo https://github.com/AssetRipper/TypeTreeDumps.
    /// File create by "tar -caf InfoJson.tar.zst InfoJson"
    /// or "tar -c InfoJson | zstd --ultra -22 -o InfoJson.tar.zst"  
    /// whitch can be less then 5MiB.
    /// contain file path like /InfoJson/x.x.x.json.
    #[arg(short, long)]
    info_json_tar_path: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List container path
    List {
        /// filter path
        #[arg(value_parser)]
        filter_path: Option<String>,
    },
    /// Extract Assets.
    Extract {
        /// filter path
        #[arg(value_parser)]
        filter_path: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(path) = args.info_json_tar_path {
        let tar_file = File::open(path)?;
        set_info_json_tar_reader(Box::new(BufReader::new(tar_file)));
    }

    let time = std::time::Instant::now();

    let mut unity_asset_viewer = UnityAssetViewer::new();
    if let Some(bundle_dir) = args.bundle_dir {
        unity_asset_viewer.read_bundle_dir(bundle_dir)?;
    }
    if let Some(data_dir) = args.data_dir {
        unity_asset_viewer.read_data_dir(data_dir)?;
    }
    if let Some(serialized_file) = args.serialized_file {
        let file = OpenOptions::new().read(true).open(serialized_file).unwrap();
        unity_asset_viewer
            .add_serialized_file(Box::new(BufReader::new(file)), Some(".".to_owned()))?;
    }
    println!("Read use {:?}", time.elapsed());

    match &args.command {
        Commands::List { filter_path } => {
            for (container_path, _) in &unity_asset_viewer.container_maps {
                if let Some(filter_path) = filter_path {
                    if container_path.starts_with(filter_path) {
                        println!("{}", container_path);
                    }
                } else {
                    println!("{}", container_path);
                }
            }

            let mut object_types = HashSet::new();
            let mut mono_behaviour_calss_types = HashSet::new();
            for (_, sf) in &unity_asset_viewer.serialized_file_map {
                for (pathid, obj) in sf.get_object_map() {
                    if obj.class == ClassIDType::MonoScript as i32 {
                        // let tt_o = sf.get_tt_object_by_path_id(*pathid).unwrap().unwrap();
                        // println!("name\t{:?}", tt_o.get_value_by_path("/Base/m_Name"));
                        // println!("\t{:?}", tt_o.get_value_by_path("/Base/m_ClassName"));
                        // println!("\t{:?}", tt_o.get_value_by_path("/Base/m_Namespace"));
                        // println!("\t{:?}", tt_o.get_value_by_path("/Base/m_AssemblyName"));
                    } else if obj.class == ClassIDType::MonoBehaviour as i32 {
                        let obj = sf
                            .get_tt_object_by_path_id(*pathid)
                            .map_err(|err| {
                                let fs_path = unity_asset_viewer
                                    .get_unity_fs_by_serialized_file(&sf)
                                    .and_then(|fs| {
                                        dump_unity_fs(fs);
                                        fs.resource_search_path.clone()
                                    });

                                anyhow!(format!(
                                    "error while read. fs_path : {:?} sf_path: {:?} error : {}",
                                    fs_path, sf.resource_search_path, err
                                ))
                            })
                            .unwrap()
                            .unwrap();

                        if let Ok(pptr_o) =
                            TypeTreeObjectRef::try_cast_from(&obj.into(), "/Base/m_Script")
                        {
                            let script_pptr = PPtr::new(&pptr_o);
                            if let Some(script) =
                                script_pptr.get_type_tree_object_in_view(&unity_asset_viewer)?
                            {
                                // println!("\t{:?}", script.get_string_by_path("/Base/m_ClassName"));
                                if let Ok(class_name) =
                                    String::try_cast_from(&script, "/Base/m_ClassName")
                                {
                                    mono_behaviour_calss_types.insert(class_name);
                                }
                            }
                        }
                    }

                    object_types.insert(obj.class.clone());
                }
            }
            println!("object_types : {:?}", object_types);
            println!(
                "mono_behaviour_calss_types : {:?}",
                mono_behaviour_calss_types
            );
        }
        Commands::Extract { filter_path: _ } => {
            for (_, sf) in &unity_asset_viewer.serialized_file_map {
                for (path_id, obj_meta) in sf.get_object_map() {
                    let obj = sf
                        .get_tt_object_by_path_id(*path_id)
                        .map_err(|err| {
                            let fs_path = unity_asset_viewer
                                .get_unity_fs_by_serialized_file(&sf)
                                .and_then(|fs| {
                                    dump_unity_fs(fs);
                                    fs.resource_search_path.clone()
                                });

                            anyhow!(format!(
                                "error while read. fs_path : {:?} sf_path: {:?} error : {}",
                                fs_path, sf.resource_search_path, err
                            ))
                        })
                        .unwrap()
                        .unwrap();

                    if let Ok(name) = String::try_cast_from(&obj, "/Base/m_Name") {
                        println!("name {}", name);
                        if obj_meta.class == ClassIDType::Texture2D as i32 {
                            let obj = obj.into();
                            let tex = Texture2D::new(&obj);

                            let out_tex_path_base = "/tmp/tex/".to_string() + &name;
                            let mut out_tex_path = out_tex_path_base.clone();
                            let mut i = 0;
                            while PathBuf::from(out_tex_path.clone() + ".png").exists() {
                                out_tex_path = format!("{}.{}", out_tex_path_base, i);
                                i += 1;
                            }
                            tex.get_image(&unity_asset_viewer)
                                .and_then(|dynimg| Ok(dynimg.flipv().save(out_tex_path + ".png")));
                        } else if obj_meta.class == ClassIDType::TextAsset as i32 {
                            if let Ok(script) = String::try_cast_from(&obj, "/Base/m_Script") {
                                let mut file =
                                    File::create("/tmp/tex/".to_string() + &name + ".txt").unwrap();
                                file.write_all(script.as_bytes());
                            }
                        } else if obj_meta.class == ClassIDType::AssetBundle as i32
                            || obj_meta.class == ClassIDType::Material as i32
                            || obj_meta.class == ClassIDType::GameObject as i32
                            || obj_meta.class == ClassIDType::MonoBehaviour as i32
                            || obj_meta.class == ClassIDType::AudioClip as i32
                            || obj_meta.class == ClassIDType::Mesh as i32
                        {
                        } else {
                            // obj.display_tree();
                            // panic!()
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn dump_unity_fs(unity_fs: &UnityFS) {
    for file in unity_fs.get_file_paths() {
        if let Ok(file_buff) = unity_fs.get_file_by_path(&file) {
            let file_name = PathBuf::from(file)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let mut file = File::create(file_name).unwrap();
            file.write_all(&*file_buff);
        }
    }
}
