extern crate io_unity;

use clap::{arg, Parser, Subcommand};
use std::collections::HashSet;

use io_unity::{
    classes::ClassIDType, type_tree::type_tree_json::set_info_json_tar_path,
    unity_asset_view::UnityAssetViewer,
};

use io_unity::*;

/// unity extractor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The dir contain AssetBundle files.
    #[arg(short, long)]
    bundle_dir: String,
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
    /// Extract one model, Assets under the filter path will be treated as one model.
    Extract {
        /// filter path
        #[arg(value_parser)]
        filter_path: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(path) = args.info_json_tar_path {
        set_info_json_tar_path(path);
    }

    let time = std::time::Instant::now();

    let mut unity_asset_viewer = UnityAssetViewer::new();
    unity_asset_viewer.read_data_dir(args.bundle_dir)?;

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
            for (_, sf) in &unity_asset_viewer.serialized_file_map {
                for (pathid, obj) in sf.get_object_map() {
                    if obj.class == ClassIDType::Texture2D {
                        if let Some(classes::Class::Texture2D(_tex)) =
                            sf.get_object_by_path_id(pathid.to_owned()).unwrap()
                        {
                            // println!("{:#?}", &tex);
                        }
                    } else if obj.class == ClassIDType::MonoScript {
                        let tt_o = sf.get_tt_object_by_path_id(*pathid).unwrap().unwrap();
                        println!("name\t{:?}", tt_o.get_value_by_path("/Base/m_Name"));
                        println!("\t{:?}", tt_o.get_value_by_path("/Base/m_ClassName"));
                        println!("\t{:?}", tt_o.get_value_by_path("/Base/m_Namespace"));
                        println!("\t{:?}", tt_o.get_value_by_path("/Base/m_AssemblyName"));
                    }
                    object_types.insert(obj.class.clone());
                }
            }
            println!("object_types : {:?}", object_types);
        }
        Commands::Extract { filter_path: _ } => {}
    }

    Ok(())
}
