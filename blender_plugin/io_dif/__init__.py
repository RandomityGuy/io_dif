# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful, but
# WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTIBILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
# General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program. If not, see <http://www.gnu.org/licenses/>.

if "bpy" in locals():
    import importlib

    if "export_dif" in locals():
        importlib.reload(export_dif)
    if "import_dif" in locals():
        importlib.reload(import_dif)

import bpy
from bpy.props import (
    BoolProperty,
    CollectionProperty,
    FloatProperty,
    IntProperty,
    StringProperty,
    EnumProperty,
    PointerProperty,
)
from bpy_extras.io_utils import (
    ImportHelper,
    ExportHelper,
)

bl_info = {
    "name": "Torque DIF",
    "author": "RandomityGuy",
    "description": "Dif import and export plguin for blender",
    "blender": (2, 80, 0),
    "version": (1, 1, 1),
    "location": "File > Import-Export",
    "warning": "",
    "category": "Import-Export",
}


class InteriorSettings(bpy.types.PropertyGroup):
    interior_type = EnumProperty(
        name="Interior Entity Type",
        items=(
            ("static_interior", "InteriorResource", "Normal static interior"),
            ("pathed_interior", "PathedInterior", "Moving interior"),
            ("game_entity", "Game Entity", "An entity in the game"),
        ),
        default="static_interior",
    )

    marker_path = PointerProperty(type=bpy.types.Curve, name="Marker Path")
    game_entity_datablock = StringProperty(name="Datablock")
    game_entity_gameclass = StringProperty(name="Game Class")


class InteriorPanel(bpy.types.Panel):
    bl_label = "DIF properties"
    bl_idname = "dif_properties"
    bl_space_type = "PROPERTIES"
    bl_region_type = "WINDOW"
    bl_context = "object"

    def draw(self, context):
        layout = self.layout
        obj = context

        sublayout = layout.row()
        sublayout.prop(context.object.dif_props, "interior_type")
        if context.object.dif_props.interior_type == "pathed_interior":
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "marker_path")
        if context.object.dif_props.interior_type == "game_entity":
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "game_entity_datablock")
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "game_entity_gameclass")


class ImportDIF(bpy.types.Operator, ImportHelper):
    """Load a Torque DIF File"""

    bl_idname = "import_scene.dif"
    bl_label = "Import DIF"
    bl_options = {"PRESET"}

    filename_ext = ".dif"
    filter_glob = StringProperty(
        default="*.dif",
        options={"HIDDEN"},
    )

    check_extension = True

    def execute(self, context):
        # print("Selected: " + context.active_object.name)
        from . import import_dif

        keywords = self.as_keywords(
            ignore=(
                "axis_forward",
                "axis_up",
                "filter_glob",
            )
        )

        if bpy.data.is_saved and context.user_preferences.filepaths.use_relative_paths:
            import os

            keywords["relpath"] = os.path.dirname(bpy.data.filepath)

        return import_dif.load(context, **keywords)

    def draw(self, context):
        pass


class ExportDIF(bpy.types.Operator, ExportHelper):
    """Save a Torque DIF File"""

    bl_idname = "export_scene.dif"
    bl_label = "Export DIF"
    bl_options = {"PRESET"}

    filename_ext = ".dif"
    filter_glob = StringProperty(
        default="*.dif",
        options={"HIDDEN"},
    )

    flip = BoolProperty(
        name="Flip faces",
        description="Flip normals of the faces, in case the resultant dif is inside out.",
        default=False,
    )

    double = BoolProperty(
        name="Double faces",
        description="Make all the faces double sided, may cause lag during collision detection.",
        default=False,
    )

    maxpolys = IntProperty(
        name="Polygons per DIF",
        description="Maximum number of polygons till a dif split is done",
        default=16000,
        min=1,
        max=16000,
    )

    applymodifiers = BoolProperty(
        name="Apply Modifiers",
        description="Apply modifiers during export",
        default=True,
    )

    check_extension = True

    def execute(self, context):
        from . import export_dif

        keywords = self.as_keywords(ignore=("check_existing", "filter_glob"))
        export_dif.save(
            context,
            keywords["filepath"],
            keywords.get("flip", False),
            keywords.get("double", False),
            keywords.get("maxpolys", 16000),
            keywords.get("applymodifiers", True),
        )
        return {"FINISHED"}


classes = (ExportDIF, ImportDIF)


def menu_func_export_dif(self, context):
    self.layout.operator(ExportDIF.bl_idname, text="Torque (.dif)")


def menu_func_import_dif(self, context):
    self.layout.operator(ImportDIF.bl_idname, text="Torque (.dif)")


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export_dif)
    bpy.types.TOPBAR_MT_file_import.append(menu_func_import_dif)
    bpy.utils.register_class(InteriorPanel)
    bpy.utils.register_class(InteriorSettings)

    bpy.types.Object.dif_props = PointerProperty(type=InteriorSettings)


def unregister():
    for cls in classes:
        bpy.utils.unregister_class(cls)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export_dif)
    bpy.types.TOPBAR_MT_file_import.remove(menu_func_import_dif)
    bpy.utils.unregister_class(InteriorPanel)
    bpy.utils.unregister_class(InteriorSettings)

    del bpy.types.Object.dif_props


if __name__ == "__main__":
    register()
