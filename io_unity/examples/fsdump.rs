extern crate io_unity;

use std::{
    fs::OpenOptions,
    io::{BufReader, Cursor},
    path::Path,
};

use io_unity::classes::ClassIDType;

use io_unity::*;

fn main() {
    let path = "/tmp/files/aa/Android/";
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

    let cabfile = oval
        .get_file_by_path(oval.get_cab_path().get(0).unwrap())
        .unwrap();
    // let mut outfile = File::create("test").unwrap();
    // outfile.write_all(&cabfile);

    let cabfile_reader = Box::new(Cursor::new(cabfile));
    let s = SerializedFile::read(cabfile_reader).unwrap();
    // println!("{:#?}", s);

    let mut fs = Box::new(oval) as Box<dyn FS>;
    let mut viewed = Vec::new();
    for (pathid, obj) in s.get_object_map() {
        // match obj.class {
        //     ClassIDType::Texture2D
        //     | ClassIDType::AudioClip
        //     | ClassIDType::TextAsset
        //     | ClassIDType::CanvasRenderer
        //     | ClassIDType::RectTransform
        //     | ClassIDType::GameObject
        //     | ClassIDType::Animation
        //     | ClassIDType::Sprite
        //     | ClassIDType::ParticleSystemRenderer
        //     | ClassIDType::ParticleSystem
        //     | ClassIDType::AnimationClip
        //     | ClassIDType::Material
        //     | ClassIDType::Shader
        //     | ClassIDType::Animator
        //     | ClassIDType::PlayableDirector
        //     | ClassIDType::Canvas
        //     | ClassIDType::SpriteAtlas
        //     | ClassIDType::CanvasGroup
        //     | ClassIDType::Transform
        //     | ClassIDType::MeshFilter
        //     | ClassIDType::MeshRenderer
        //     | ClassIDType::Font
        //     | ClassIDType::TrailRenderer
        //     | ClassIDType::Camera
        //     | ClassIDType::AudioListener
        //     | ClassIDType::AnimatorController
        //     | ClassIDType::AudioSource
        //     | ClassIDType::AudioMixerGroupController
        //     | ClassIDType::AudioMixerController
        //     | ClassIDType::AudioMixerSnapshotController
        //     | ClassIDType::MonoBehaviour
        //     | ClassIDType::AssetBundle => {
        //         continue;
        //     }
        //     _ => {
        //         // let tt_o = s.get_tt_object_by_path_id(*pathid).unwrap();
        //         // tt_o.display_tree();
        //         // println!("{:?}", tt_o.get_value_by_path("/Base/m_Script"));
        //         // panic!("")
        //     }
        // }
        if obj.class == ClassIDType::Texture2D {
            if let Some(classes::Class::Texture2D(tex)) = s.get_object_by_path_id(pathid.to_owned())
            {
                // println!("{:#?}", &tex);
                tex.get_image(&mut fs).and_then(|t| {
                    Ok(t.flipv().save(
                        "/tmp/tex/".to_string() + &tex.downcast().get_name().unwrap() + ".png",
                    ))
                });
            }
        }
        // if obj.class == ClassIDType::Transform {
        //     if let Some(classes::Class::Transform(tran)) = s.get_object_by_path_id(pathid.to_owned())
        //     {
        //         println!("{:#?}", &tran.get_local_mat());
        //     }
        // }
        if obj.class == ClassIDType::TextAsset {
            let tt_o = s.get_tt_object_by_path_id(*pathid).unwrap();
            tt_o.display_tree();
            println!("{:?}", tt_o.get_value_by_path("/Base/m_Script"));
        }
        // if obj.class == ClassIDType::MonoBehaviour {
        //     let tt_o = s.get_tt_object_by_path_id(*pathid).unwrap();
        //     tt_o.display_tree();
        //     println!("{:?}", tt_o.get_value_by_path("/Base/m_Script"));
        // }
        if obj.class == ClassIDType::MonoScript {
            let tt_o = s.get_tt_object_by_path_id(*pathid).unwrap();
            // tt_o.display_tree();
            println!("name\t{:?}", tt_o.get_value_by_path("/Base/m_Name"));
            println!("\t{:?}", tt_o.get_value_by_path("/Base/m_ClassName"));
            println!("\t{:?}", tt_o.get_value_by_path("/Base/m_Namespace"));
            println!("\t{:?}", tt_o.get_value_by_path("/Base/m_AssemblyName"));
        }
        if !viewed.contains(&obj.class) {
            let tt_o = s.get_tt_object_by_path_id(*pathid).unwrap();
            println!("class {:?}", &obj.class);
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
                // println!("{:#?}", smr);
                let mut bone_name_buff = Vec::new();
                let mut bone_father_index_buff = Vec::new();

                for bone in &*smr.get_bones().unwrap() {
                    if let Some(classes::Class::Transform(bone)) =
                        s.get_object_by_path_id(bone.get_path_id().unwrap())
                    {
                        // println!("{:#?}", bone);
                        bone_name_buff.push(
                            if let Some(classes::Class::GameObject(go)) = s.get_object_by_path_id(
                                bone.downcast()
                                    .get_game_object()
                                    .unwrap()
                                    .get_path_id()
                                    .unwrap(),
                            ) {
                                go.get_name().unwrap().to_string()
                            } else {
                                "bone".to_string()
                            },
                        );
                        let bones = smr.get_bones().unwrap();
                        let father = bones.iter().enumerate().find(|(_index, itbone)| {
                            itbone.get_path_id().unwrap()
                                == bone.get_father().unwrap().get_path_id().unwrap()
                        });
                        bone_father_index_buff
                            .push(father.and_then(|e| Some(e.0 as i32)).unwrap_or(-1));
                    }
                }
                println!("{:#?}", bone_name_buff.len());
                println!("{:#?}", bone_father_index_buff.len());

                for material in &*smr.get_materials().unwrap() {
                    let material = s.get_object_by_path_id(material.get_path_id().unwrap());
                    println!("{:#?}", material);
                }
                if let Some(classes::Class::Mesh(_mesh)) =
                    s.get_object_by_path_id(smr.get_mesh().unwrap().get_path_id().unwrap())
                {
                    // println!("{:#?}", mesh.get_bind_pose());
                }
                break;
            }
        }
    }
}
