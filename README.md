# IO DIF

Blender plugin to import and export Torque DIF interiors.

## Features

### Import DIF

- Powered by [hxDIF](https://github.com/RandomityGuy/hxDIF)
- Supports PathedInteriors and its path
- Supports embedded GameEntities with properties
- Supports loading textures

### Export DIF

- Powered by [DifBuilder](https://github.com/RandomityGuy/DifBuilder)
- Export support for PathedInteriors and Markers
- Export support for GameEntities and its properties
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
    - Properties: a list of additional key value pairs which will be set to the object on Create Subs

## Limitations

- Sometimes difs are broken. To fix them, theres a few methods:
  - Split the object file either manually or use the "Polygons per DIF" option in Export DIF
  - Move the object somewhere else
- No Trigger support: I tried but Torque was being Torque even when I successfully embedded them into difs.
- No Game Entity rotation support: there isnt even a rotation field for Game Entities in difs, and torque doesnt even use the rotation field explicitly passed as a property

## Previews

![Imgur](https://imgur.com/OkSM6lY.png)

![Imgur](https://imgur.com/3NC5JmH.png)

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
