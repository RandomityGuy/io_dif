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

import bpy
from bpy.props import (
    BoolProperty,
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
    "name": "blend2dif",
    "author": "RandomityGuy",
    "description": "Directly export Torque DIFs from blender using the obj2difplus engine",
    "blender": (2, 80, 0),
    "version": (1, 0, 0),
    "location": "",
    "warning": "",
    "category": "Generic",
}

class InteriorSettings(bpy.types.PropertyGroup):
    interior_type = EnumProperty(
        name="Interior Type",
        items=(
            ("static_interior", "InteriorResource", "Normal static interior"),
            ("pathed_interior", "PathedInterior", "Moving interior")
        ),
        default="static_interior")

    marker_path = PointerProperty(type=bpy.types.Curve, name="Marker Path")

class InteriorPanel(bpy.types.Panel):
    bl_label = "DIF properties"
    bl_idname = "dif_properties"
    bl_space_type = "PROPERTIES"
    bl_region_type = "WINDOW"
    bl_context = "object"

    def draw(self, context):
        layout = self.layout;
        obj = context

        sublayout = layout.row()
        sublayout.prop(context.object.dif_props,"interior_type")
        sublayout = layout.row()
        sublayout.prop(context.object.dif_props,"marker_path")
        sublayout.active = context.object.dif_props.interior_type == "pathed_interior"

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

    check_extension = True

    def execute(self, context):
        from . import export_dif

        keywords = self.as_keywords(ignore=("check_existing", "filter_glob"))
        export_dif.save(
            self,
            keywords["filepath"],
            keywords.get("flip", False),
            keywords.get("double", False),
            keywords.get("maxpolys", 16000),
        )
        return {"FINISHED"}


classes = (ExportDIF,)


def menu_func_export_dif(self, context):
    self.layout.operator(ExportDIF.bl_idname, text="Torque (.dif)")


def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export_dif)
    bpy.utils.register_class(InteriorPanel)
    bpy.utils.register_class(InteriorSettings)

    bpy.types.Object.dif_props = PointerProperty(type=InteriorSettings)


def unregister():
    for cls in classes:
        bpy.utils.unregister_class(cls)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export_dif)
    bpy.utils.unregister_class(InteriorPanel)
    bpy.utils.unregister_class(InteriorSettings)

    del bpy.types.Object.dif_props

if __name__ == "__main__":
    register()
