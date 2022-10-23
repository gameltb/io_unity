extern crate io_unity;

use std::{
    fs::OpenOptions,
    io::{BufReader, Cursor},
    path::Path,
};

use io_unity::classes::ClassIDType;

use io_unity::*;

fn main() {
    let path = "/tmp/files/AssetBundle/";
    let dirs = std::fs::read_dir(path).unwrap();
    for entry in dirs {
        if let Ok(entry) = entry {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file()
                    && entry
                        .path()
                        .to_string_lossy()
                        .to_lowercase()
                        .ends_with(".bundle")
                {
                    println!("{}", entry.path().display());
                    handle(entry.path());
                }
            } else {
                println!("Couldn't get file type for {:?}", entry.path());
            }
        }
    }
}

fn handle<P: AsRef<Path>>(filepath: P) {
    let file = OpenOptions::new().read(true).open(filepath).unwrap();
    let file = BufReader::new(file);

    let oval = UnityFS::read(Box::new(file), None).unwrap();
    for p in oval.get_files() {
        println!("{}", p.path());
    }

    let cabfile = oval.get_cab().unwrap();
    // let mut outfile = File::create("test").unwrap();
    // outfile.write_all(&cabfile);

    let cabfile_reader = Box::new(Cursor::new(cabfile));
    let s = SerializedFile::read(cabfile_reader).unwrap();
    // println!("{:#?}", s);

    let mut fs = Box::new(oval) as Box<dyn FS>;
    let mut viewed = Vec::new();
    for (pathid, obj) in s.get_object_map() {
        if obj.class == ClassIDType::Texture2D {
            if let Some(classes::Class::Texture2D(tex)) = s.get_object_by_path_id(pathid.to_owned())
            {
                println!("{:#?}", &tex);
                tex.get_image(&mut fs).and_then(|t| {
                    Some(
                        t.flipv()
                            .save("/tmp/tex/".to_string() + &tex.get_image_name() + ".png"),
                    )
                });
            }
        }
        if !viewed.contains(&obj.class) {
            let tt_o = s.get_tt_object_by_path_id(*pathid).unwrap();
            tt_o.display_tree();
            println!("{:?}", tt_o.get_value_by_path("/Base/m_Name"));
            // println!("{:#?}", s.get_tt_object_by_path_id(*pathid));
            viewed.push(obj.class.clone());
        }
        continue;
        if obj.class == ClassIDType::SkinnedMeshRenderer {
            println!("{:#?}", pathid);
            // continue;
            if let Some(classes::Class::SkinnedMeshRenderer(smr)) =
                s.get_object_by_path_id(pathid.to_owned())
            {
                println!("{:#?}", smr);
                let mut bone_name_buff = Vec::new();
                let mut bone_father_index_buff = Vec::new();

                for bone in smr.get_bones() {
                    if let Some(classes::Class::Transform(bone)) =
                        s.get_object_by_path_id(bone.get_path_id())
                    {
                        println!("{:#?}", bone);
                        bone_name_buff.push(
                            if let Some(classes::Class::GameObject(go)) = s.get_object_by_path_id(
                                bone.get_component().get_game_object().get_path_id(),
                            ) {
                                go.get_name().to_string()
                            } else {
                                "bone".to_string()
                            },
                        );
                        let father =
                            smr.get_bones()
                                .into_iter()
                                .enumerate()
                                .find(|(_index, itbone)| {
                                    itbone.get_path_id() == bone.get_father().get_path_id()
                                });
                        bone_father_index_buff
                            .push(father.and_then(|e| Some(e.0 as i32)).unwrap_or(-1));
                    }
                }
                println!("{:#?}", bone_name_buff.len());
                println!("{:#?}", bone_father_index_buff.len());

                for material in smr.get_materials() {
                    let material = s.get_object_by_path_id(material.get_path_id());
                    println!("{:#?}", material);
                }
                if let Some(classes::Class::Mesh(mesh)) =
                    s.get_object_by_path_id(smr.get_mesh().get_path_id())
                {
                    println!("{:#?}", mesh.get_bind_pose());
                }
                break;
            }
        }
    }
}
