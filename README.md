# io_unity

io_unity is a lib for read unity assets, which supports parsing UnityFS file and serialized file.

## Additional type tree

The crate can use
tar zstd compressed file contain type tree info json files
for read file without typetree info.
see https://github.com/DaZombieKiller/TypeTreeDumper
and https://github.com/AssetRipper/TypeTreeDumps.  
File can create by InfoJson in TypeTreeDumps use

```sh
tar -caf InfoJson.tar.zst InfoJson
```

or

```sh
tar -c InfoJson | zstd --ultra -22 -o InfoJson.tar.zst
```

whitch can be less then 5MiB.
contain file path like /InfoJson/x.x.x.json.

# example

[io_unity/examples/live2dextractor.rs](io_unity/examples/live2dextractor.rs)

```sh
# build
cargo build --example live2dextractor --release --features="all"
# run
cargo run --example live2dextractor --release --features="all" -- help
```

# simple python bind

[io_unity_python](io_unity_python/README.md)

```python
import io_unity_python

uav = io_unity_python.UnityAssetViewer()
uav.add_bundle_file("BUNDLE FILE PATH")

for objref in uav:
    obj = uav.deref_object_ref(objref)
    obj.display_tree()
    try:
        print(obj.m_Name)
    except AttributeError:
        pass

print(help(obj))
```

[simple python](io_unity_python/blender.py) for blender import mesh and skeleton.

# simple gui

use [tauri](https://tauri.app/v1/guides/getting-started/prerequisites)  
require [pnpm](https://pnpm.io/installation)

run

```shell
cd io_unity_gui
pnpm tauri dev
```
