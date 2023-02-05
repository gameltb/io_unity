import math
import os
from mathutils import Matrix, Quaternion, Vector
from bpy_extras.io_utils import unpack_list
import bpy

try:
    import io_unity_python
except ModuleNotFoundError as e:
    import sys
    sys.path.append("target/release")
    import io_unity_python

import gc
gc.collect()


def import_SkinnedMeshRenderer(SkinnedMeshRenderer_obj):
    # obj.display_tree()

    unity_root_bone = io_unity_python.get_root_bone(uav, io_unity_python.PPtr(
        SkinnedMeshRenderer_obj.m_RootBone).get_type_tree_object_in_view(uav))
    # unity_root_bone.display_tree()
    bone_hash_name_map = io_unity_python.get_bone_path_hash_map(
        uav, unity_root_bone)

    mesh = io_unity_python.PPtr(
        SkinnedMeshRenderer_obj.m_Mesh).get_type_tree_object_in_view(uav)
    if mesh == None:
        return
    # mesh.display_tree()

    unity_mesh_BoneNameHashes = mesh.m_BoneNameHashes
    unity_mesh_BindPose = mesh.m_BindPose
    unity_mesh_name = mesh.m_Name
    unity_mesh = io_unity_python.Mesh(mesh)

    sub_meshs = []

    for i in range(unity_mesh.get_sub_mesh_count()):
        index = unity_mesh.get_index_buff(i)
        vertex = unity_mesh.get_vertex_buff(i)
        normals = unity_mesh.get_normal_buff(i)
        uv = unity_mesh.get_uv0_buff(i)
        bone_weights = unity_mesh.get_bone_weights_buff(i)

        subMeshName = unity_mesh_name + "_" + str(i)

        mesh = bpy.data.meshes.new(subMeshName)
        obj = bpy.data.objects.new(subMeshName, mesh)
        bpy.context.layer_collection.collection.objects.link(obj)
        sub_meshs.append(obj)

        mesh.vertices.add(int(len(vertex)/3))
        mesh.loops.add(len(index))
        mesh.polygons.add(int(len(index) / 3))

        mesh.vertices.foreach_set("co", vertex)
        mesh.loops.foreach_set("vertex_index", index)

        mesh.polygons.foreach_set(
            "loop_start", [3*i for i in range(int(len(index)/3))])
        mesh.polygons.foreach_set(
            "loop_total", [3 for i in range(int(len(index)/3))])
        mesh.polygons.foreach_set(
            "use_smooth", [True for i in range(int(len(index)/3))])

        mesh.uv_layers.new()
        mesh.uv_layers[0].data.foreach_set(
            "uv", unpack_list([[uv[i*2], uv[i*2 + 1]] for i in index]))

        mesh.create_normals_split()

        mesh.loops.foreach_set("normal", unpack_list(
            [[normals[i*3], normals[i*3 + 1], normals[i*3 + 2]] for i in index]))

        mesh.validate(clean_customdata=False)

        for subvertex_index, (weights, bindex) in enumerate(bone_weights):
            for i, BoneWeight in enumerate(weights):
                if BoneWeight == 0:
                    continue
                BoneName = os.path.basename(
                    bone_hash_name_map[unity_mesh_BoneNameHashes[bindex[i]]])
                if BoneName not in obj.vertex_groups:
                    vertex_group = obj.vertex_groups.new(name=BoneName)
                else:
                    vertex_group = obj.vertex_groups[BoneName]
                vertex_group.add([subvertex_index],
                                 BoneWeight, "ADD")

    unity_mesh_BindPose_BoneName = []
    for Hash in unity_mesh_BoneNameHashes:
        unity_mesh_BindPose_BoneName.append(
            os.path.basename(bone_hash_name_map[Hash]))

    Skeleton = importSkeleton(
        uav, unity_root_bone, unity_mesh_BindPose_BoneName, unity_mesh_BindPose, unity_mesh_name)

    for sub_mesh in sub_meshs:
        sub_mesh.parent = Skeleton
        SkeletonMod = sub_mesh.modifiers.new("armature", 'ARMATURE')
        SkeletonMod.object = Skeleton

    for mat in SkinnedMeshRenderer_obj.m_Materials:
        try:
            mat = io_unity_python.PPtr(
                mat).get_type_tree_object_in_view(uav)
            # mat.display_tree()
            TexEnvs = mat.m_SavedProperties.m_TexEnvs

            for tex_type in TexEnvs:
                try:
                    tex = io_unity_python.PPtr(
                        TexEnvs[tex_type].m_Texture).get_type_tree_object_in_view(uav)
                    if tex != None:
                        tex_name = tex.m_Name
                        tex = io_unity_python.Texture2D(tex)
                        os.makedirs(os.path.join(
                            mat_tex_out_dir, mat.m_Name), exist_ok=True)
                        tex.save_image(uav, os.path.join(os.path.join(
                            mat_tex_out_dir, mat.m_Name), tex_name + tex_type + ".png"))
                except Exception as e:
                    print(tex_type, e)
        except Exception as e:
            print(e)


def importSkeleton(uav, unity_root_bone, unity_mesh_BindPose_BoneName, unity_mesh_BindPose, armname="test"):
    armature = bpy.data.armatures.new(armname)
    armature.show_axes = True
    Skeleton = bpy.data.objects.new(armature.name, armature)

    bpy.context.layer_collection.collection.objects.link(Skeleton)

    Skeleton.select_set(True)
    bpy.context.view_layer.objects.active = Skeleton

    old_type = None
    if not bpy.ops.object.mode_set.poll():
        area = bpy.context.area
        old_type = area.type
        area.type = 'VIEW_3D'

    prev_mode = Skeleton.mode
    bpy.ops.object.mode_set(mode="EDIT")

    def import_bone(unity_bone, parent_bone=None):
        game_object = io_unity_python.PPtr(
            unity_bone.m_GameObject).get_type_tree_object_in_view(uav)
        if game_object.m_Name in unity_mesh_BindPose_BoneName:
            edit_bone = Skeleton.data.edit_bones.new(game_object.m_Name)
            if edit_bone != None:
                edit_bone.parent = parent_bone

            edit_bone.tail.x = 0.05

            bone_BindPose = unity_mesh_BindPose[unity_mesh_BindPose_BoneName.index(
                game_object.m_Name)]
            mat = Matrix()
            mat[0][0], mat[0][1], mat[0][2], mat[0][3] = bone_BindPose.e00, bone_BindPose.e01, bone_BindPose.e02, bone_BindPose.e03
            mat[1][0], mat[1][1], mat[1][2], mat[1][3] = bone_BindPose.e10, bone_BindPose.e11, bone_BindPose.e12, bone_BindPose.e13
            mat[2][0], mat[2][1], mat[2][2], mat[2][3] = bone_BindPose.e20, bone_BindPose.e21, bone_BindPose.e22, bone_BindPose.e23
            mat[3][0], mat[3][1], mat[3][2], mat[3][3] = bone_BindPose.e30, bone_BindPose.e31, bone_BindPose.e32, bone_BindPose.e33

            mat.invert()

            edit_bone.matrix = mat
        else:
            edit_bone = parent_bone

        for cbone in unity_bone.m_Children:
            import_bone(io_unity_python.PPtr(
                cbone).get_type_tree_object_in_view(uav), edit_bone)
        # +z =-> -x +y -> +z +x -> -y

    import_bone(unity_root_bone, parent_bone=None)

    bpy.ops.object.mode_set(mode=prev_mode)

    if old_type:
        area.type = old_type

    return Skeleton

# delete all object
# bpy.ops.object.mode_set(mode="OBJECT")
# bpy.ops.object.select_all(action='SELECT')
# bpy.ops.object.delete()


mat_tex_out_dir = ""

uav = io_unity_python.UnityAssetViewer()

uav.read_bundle_dir("Bundle Dir")

import_names = []

for objref in uav.get_objrefs():
    if objref.get_class_id() == 137:
        obj = uav.deref_object_ref(objref)
        gobj = io_unity_python.PPtr(
            obj.m_GameObject).get_type_tree_object_in_view(uav)
        print(gobj.m_Name)
        if gobj.m_Name in import_names:
            try:
                import_SkinnedMeshRenderer(obj)
            except Exception as e:
                raise e
            break



del uav
gc.collect()
