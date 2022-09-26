try:
    import io_unity_python
except ModuleNotFoundError as e:
    import sys
    sys.path.append("target/debug")
    import io_unity_python


file_path = "test.ab"
object_path_id = 0

fs = io_unity_python.UnityFS.readfs(file_path)


cab = fs.get_cab()
ids = []

for i in ids:
    try:
        obj = cab.get_object_by_path_id(i)
        print(i)
        print(obj.get_bone_name_and_index_buff(cab))
        cabmesh = obj.get_mesh(cab)
                
        for i in range(cabmesh.get_sub_mesh_count()):
            index = cabmesh.get_index_buff(i)
            vertex = cabmesh.get_vertex_buff(i)
            normals = cabmesh.get_normal_buff(i)
            uv = cabmesh.get_uv0_buff(i)
        print(cabmesh.get_bind_pose())
    except Exception as e:
        print(i)
        raise e

