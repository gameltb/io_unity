from mathutils import Matrix, Quaternion, Vector
from bpy_extras.io_utils import unpack_list
import bpy

try:
    import io_unity_python
except ModuleNotFoundError as e:
    import sys
    sys.path.append("target/debug")
    import io_unity_python

import importlib
importlib.reload(io_unity_python)


file_path = "test.ab"
object_path_id = 0


fs = io_unity_python.UnityFS.readfs(file_path)

cab = fs.get_cab()

cabobj = cab.get_object_by_path_id(object_path_id)

cab_mesh_bone_name, cab_mesh_bone_index, cab_mesh_bone_local_mat = cabobj.get_bone_name_index_local_mat_buff(
    cab)
cabmesh = cabobj.get_mesh(cab)
cab_pose = cabmesh.get_bind_pose()

sub_meshs = []


def importSkeleton(pose, bonename, boneindex, armname="test"):
    import bpy
    from mathutils import Vector
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

    def fixmat(mat):
        fix = Matrix(
            [[1, 0, 0, 0],
             [0, 0, 1, 0],
             [0, 1, 0, 0],
             [0, 0, 0, 1]])
        return fix @ mat @ fix

    def getWorldCoordinate(bi, mat):
        ii = bi
        while boneindex[ii] != -1:
            mat = fixmat(Matrix(pose[ii]).transposed()) @ mat
            ii = boneindex[ii]
        print("getWorldCoordinate", mat, mat.decompose())
        return mat

    for i, bone in enumerate(pose):
        edit_bone = Skeleton.data.edit_bones.new(bonename[i])

        edit_bone.tail.y = 0.05
        print("raw", bone, Matrix(bone).transposed(),
              Matrix(bone).transposed().decompose())

        edit_bone.matrix = getWorldCoordinate(
            i, fixmat(Matrix(bone).transposed()))
        # edit_bone.matrix = fixmat(Matrix(bone))#.transposed()

    for i, bone in enumerate(pose):
        edit_bone = Skeleton.data.edit_bones[bonename[i]]

        if boneindex[i] != -1:
            edit_bone.parent = Skeleton.data.edit_bones[bonename[boneindex[i]]]

    bpy.ops.object.mode_set(mode=prev_mode)

    if old_type:
        area.type = old_type

    return Skeleton


for i in range(cabmesh.get_sub_mesh_count()):
    index = cabmesh.get_index_buff(i)
    vertex = cabmesh.get_vertex_buff(i)
    normals = cabmesh.get_normal_buff(i)
    uv = cabmesh.get_uv0_buff(i)
    bone_weights = cabmesh.get_bone_weights_buff(i)

    subMeshName = 'test' + str(i)

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
            BoneName = cab_mesh_bone_name[bindex[i]]
            if BoneName not in obj.vertex_groups:
                vertex_group = obj.vertex_groups.new(name=BoneName)
            vertex_group = obj.vertex_groups[BoneName]
            vertex_group.add([subvertex_index],
                             BoneWeight, "ADD")

Skeleton = importSkeleton(
    cab_mesh_bone_local_mat, cab_mesh_bone_name, cab_mesh_bone_index)

for sub_mesh in sub_meshs:
    sub_mesh.parent = Skeleton
    SkeletonMod = sub_mesh.modifiers.new("armature", 'ARMATURE')
    SkeletonMod.object = Skeleton
