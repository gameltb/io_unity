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