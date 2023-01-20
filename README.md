# io_unity

io_unity is a lib for read unity assets, which supports parsing UnityFS file now.(WIP)


## example

[io_unity/examples/live2dextractor.rs](io_unity/examples/live2dextractor.rs)

## simple python bind

[io_unity_python](io_unity_python/README.md)

[simple python](io_unity_python/blender.py) for blender import mesh (skeleton not work)

## simple gui

use [tauri](https://tauri.app/v1/guides/getting-started/prerequisites)  
require [pnpm](https://pnpm.io/installation)

run 
```shell
cd io_unity_gui
pnpm tauri dev
```
