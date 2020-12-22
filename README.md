# IO DIF

Blender plugin to import and export Torque DIF interiors.

## Features

### Import DIF

- Powered by [hxDIF](https://github.com/RandomityGuy/hxDIF)
- Supports PathedInteriors and its path
- Supports embedded GameEntities
- Supports loading textures

### Export DIF

- Powered by [DifBuilder](https://github.com/RandomityGuy/DifBuilder)
- Export support for PathedInteriors and Markers
- Export support for GameEntities
- Additional export parameters provided by [obj2difPlus](https://github.com/RandomityGuy/obj2difPlus)

## Installation

Download the plugin from Releases and install it how you normally install blender plugins.

## How to

### Import

File > Import > Torque (.dif).  
It can't be any more simpler than that

### Export

File > Export > Torque (.dif)

#### Additional export options

Flip Faces: Flip the normals of the dif, incase the resultant dif is inside out.  
Double Faces: Make all the faces double sided, may increase lag during collision detection.

### DIF Properties Panel

Located in the object properties panel

- Interior Entity Type:
  - InteriorResource: normal static interior type
  - PathedInterior: moving platform type interior
    - Marker Path: a curve object that describes the path of the moving platform
      - initialPathPosition: set using the "Evaluation Time" parameter located in Curve > Object Data Properties > Path Animation
      - totalPathTime: time it takes for the moving platform to complete the path, set using the "Frames" parameter located in Curve > Object Data Properties > Path Animation.
  - Game Entity: represents an entity in the dif such as items
    - Game Class: the class of the entity such as "Item", "StaticShape",etc
    - Datablock: the datablock of the item.

## Build

Checkout the repository correctly

```
git checkout https://github.com/RandomityGuy/io_dif.git
git submodule init
git submodule update
git submodule foreach git submodule init
git submodule foreach git submodule update
```

Then build DifBuilderLib.dll using CMake.  
Copy resultant DifBuilderLib.dll to blender_plugin/io_dif folder.  
Copy blender_plugin/io_dif to your blender plugins folder.

## Credits

Thanks HiGuy for your incomplete blender dif import plugin
