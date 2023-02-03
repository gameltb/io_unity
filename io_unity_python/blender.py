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

# delete all object
# bpy.ops.object.mode_set(mode="OBJECT")
# bpy.ops.object.select_all(action='SELECT')
# bpy.ops.object.delete()


uav = io_unity_python.UnityAssetViewer()

uav.add_bundle_file("BUNDLE FILE PATH")

for objref in uav:
    if objref.get_class_id() == 137:
        obj = uav.deref_object_ref(objref)
        break

# obj.display_tree()

unity_root_bone = io_unity_python.PPtr(
    obj.m_RootBone).get_type_tree_object_in_view(uav)
# unity_root_bone.display_tree()
bone_hash_name_map = io_unity_python.get_bone_path_hash_map(
    uav, unity_root_bone)

mesh = io_unity_python.PPtr(obj.m_Mesh).get_type_tree_object_in_view(uav)
# mesh.display_tree()

unity_mesh_BoneNameHashes = mesh.m_BoneNameHashes
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


def importSkeleton(uav, unity_root_bone, armname="test"):
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
        edit_bone = Skeleton.data.edit_bones.new(game_object.m_Name)
        if edit_bone != None:
            edit_bone.parent = parent_bone

        edit_bone.tail.x = 0.05

        position = unity_bone.m_LocalPosition
        position = Vector((position.x, position.y, position.z))
        scale = unity_bone.m_LocalScale
        scale = Vector((scale.x, scale.y, scale.z))
        rotation = unity_bone.m_LocalRotation
        rotation = Quaternion((rotation.w, rotation.x, rotation.y, rotation.z))

        m = Matrix.Translation(position)
        m = m @ rotation.to_matrix().to_4x4()
        m = m @ Matrix.Scale(1, 4, scale)

        if parent_bone == None:
            edit_bone.matrix = m
        else:
            edit_bone.matrix = parent_bone.matrix @ m

        for cbone in unity_bone.m_Children:
            import_bone(io_unity_python.PPtr(
                cbone).get_type_tree_object_in_view(uav), edit_bone)
        # +z =-> -x +y -> +z +x -> -y

    import_bone(unity_root_bone, parent_bone=None)

    bpy.ops.object.mode_set(mode=prev_mode)

    if old_type:
        area.type = old_type

    return Skeleton


Skeleton = importSkeleton(uav, unity_root_bone, unity_mesh_name)

for sub_mesh in sub_meshs:
    sub_mesh.parent = Skeleton
    SkeletonMod = sub_mesh.modifiers.new("armature", 'ARMATURE')
    SkeletonMod.object = Skeleton
