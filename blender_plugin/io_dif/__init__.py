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

    if "import_csx" in locals():
        importlib.reload(import_csx)

import os
import platform
import bpy
import threading
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
    "description": "Dif import and export plugin for blender",
    "blender": (2, 80, 0),
    "version": (1, 3, 6),
    "location": "File > Import-Export",
    "warning": "",
    "category": "Import-Export",
}


class InteriorKVP(bpy.types.PropertyGroup):
    key: StringProperty(name="")
    value: StringProperty(name="")


class AddCustomProperty(bpy.types.Operator):
    bl_idname = "dif.add_prop"
    bl_label = "Add Property"

    def execute(self, context):
        dif_props: InteriorSettings = context.object.dif_props
        prop = dif_props.game_entity_properties.add()
        prop.key = "Key"
        prop.value = "Value"
        return {"FINISHED"}


class DeleteCustomProperty(bpy.types.Operator):
    bl_idname = "dif.delete_prop"
    bl_label = "Delete Property"

    delete_id: IntProperty()

    def execute(self, context):
        dif_props: InteriorSettings = context.object.dif_props
        prop = dif_props.game_entity_properties.remove(self.delete_id)
        return {"FINISHED"}
    
    
def set_marker_path(self, context):
    curve = self.marker_path
    if curve:
        for spline in curve.splines:
            spline.type = 'POLY'

class InteriorSettings(bpy.types.PropertyGroup):
    # Interiors
    interior_type: EnumProperty(
        name="Interior Entity Type",
        items=(
            ("static_interior", "Interior Resource", "Normal static interior"),
            ("pathed_interior", "Pathed Interior", "Moving interior"),
            ("game_entity", "Game Entity", "A game object"),
            ("path_trigger", "Path Trigger", "A trigger for a pathed interior"),
        ),
        default="static_interior",
        description="How this object should be interpreted for the exporter.",
    )
    marker_path: PointerProperty(type=bpy.types.Curve, name="Marker Path", description="The path to create markers from.", update=set_marker_path)
    constant_speed: BoolProperty(name = "Constant Speed", description = "If the marker durations should be based on speed instead of total time.", default=True)
    speed: FloatProperty(name="Speed", description="The speed that the platform should be moving at. If using Accelerate smoothing, this is max speed.", default=1, min=0.01, max=100)
    total_time: IntProperty(name="Total Time", description="The total time (in ms) from path start to end. Equally divided across each marker on export.", default=3000, min=1)
    start_time: IntProperty(name="Start Time", description="The time in the path (in ms) that the platform should be at level restart.", default=0, min=0)
    start_index: IntProperty(name="Start Index", description="The marker that the platform should be at level restart (0 is 1st marker).", default=0, min=0, soft_max=10)
    pause_duration: IntProperty(name = "Pause Duration", description="At a path segment of length 0, the platform will wait this long (in ms).", default=0, min=0, soft_max=10000)
    reverse: BoolProperty(name = "Reverse", description = "If the platform should loop backwards (if not using a trigger).")

    marker_type: EnumProperty(
        name="Marker Type",
        items=(
            ("linear", "Linear", "Linear interpolation"),
            ("spline", "Spline", "Centripetal Catmull–Rom path"),
            ("accelerate", "Accelerate", "Sinusoidal easing"),
        ),
        description="The type of smoothing that should be applied to all markers exported from the path.",
    )

    # Triggers
    pathed_interior_target: PointerProperty(type=bpy.types.Object, name="Pathed Interior Target", description="The platform to trigger.")
    target_marker: BoolProperty(name = "Calculate Target Time", description="If enabled, the targetTime will be calculated to be at a specific marker.", default=True)
    target_index: IntProperty(name = "Target Index", description="The marker to target (0 is 1st marker).", default=0, min=0, soft_max=10)
    
    # Entities
    game_entity_datablock: StringProperty(name="Datablock")
    game_entity_gameclass: StringProperty(name="Game Class")
    game_entity_properties: CollectionProperty(
        type=InteriorKVP, name="Custom Properties"
    )


class InteriorPanel(bpy.types.Panel):
    bl_label = "DIF properties"
    bl_idname = "dif_properties"
    bl_space_type = "PROPERTIES"
    bl_region_type = "WINDOW"
    bl_context = "object"

    def draw(self, context):
        layout = self.layout
        sublayout = layout.row()
        sublayout.prop(context.object.dif_props, "interior_type") #TODO only show this on relevant objects?

        if(isinstance(context.object, bpy.types.Curve)):
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "marker_type")

        if context.object.dif_props.interior_type == "pathed_interior":
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "marker_path")
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "marker_type")
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "constant_speed")
            sublayout = layout.row()
            if context.object.dif_props.constant_speed:
                sublayout.prop(context.object.dif_props, "speed")
                sublayout = layout.row()
                sublayout.prop(context.object.dif_props, "start_index")
                sublayout = layout.row()
                sublayout.prop(context.object.dif_props, "pause_duration")
                sublayout = layout.row()
            else:
                sublayout.prop(context.object.dif_props, "total_time")
                sublayout = layout.row()
                sublayout.prop(context.object.dif_props, "start_time")
                sublayout = layout.row()

            sublayout.prop(context.object.dif_props, "reverse")

        if context.object.dif_props.interior_type in ["game_entity", "path_trigger"]:
            sublayout = layout.row()
            sublayout.prop(context.object.dif_props, "game_entity_datablock")
            sublayout = layout.row()
            if context.object.dif_props.interior_type == "path_trigger":
                sublayout.prop(context.object.dif_props, "pathed_interior_target")
                sublayout = layout.row()
                sublayout.prop(context.object.dif_props, "target_marker")
                sublayout = layout.row()
                if context.object.dif_props.target_marker:
                    sublayout.prop(context.object.dif_props, "target_index")
                    sublayout = layout.row()
            else:
                sublayout.prop(context.object.dif_props, "game_entity_gameclass")
            sublayout = layout.row()
            sublayout.label(text="Properties:")
            sublayout = layout.row()
            sublayout.operator(AddCustomProperty.bl_idname, text="Add Property")
            for i, custom_property in enumerate(
                context.object.dif_props.game_entity_properties
            ):
                sublayout = layout.row()
                sublayout.prop(
                    context.object.dif_props.game_entity_properties[i], "key"
                )
                sublayout.prop(
                    context.object.dif_props.game_entity_properties[i], "value"
                )
                sublayout.operator(
                    DeleteCustomProperty.bl_idname, icon="X", text=""
                ).delete_id = i


class ImportCSX(bpy.types.Operator, ImportHelper):
    """Load a Torque Constructor CSX File"""

    bl_idname = "import_scene.csx"
    bl_label = "Import Constructor CSX"
    bl_options = {"PRESET"}

    filename_ext = ".csx"
    filter_glob: StringProperty(
        default="*.csx",
        options={"HIDDEN"},
    )

    check_extension = True

    def execute(self, context):
        # print("Selected: " + context.active_object.name)
        from . import import_csx

        keywords = self.as_keywords(
            ignore=(
                "axis_forward",
                "axis_up",
                "filter_glob",
            )
        )

        if bpy.data.is_saved and context.preferences.filepaths.use_relative_paths:
            import os

            keywords["relpath"] = os.path.dirname(bpy.data.filepath)

        return import_csx.load(context, **keywords)

    def draw(self, context):
        pass


class ImportDIF(bpy.types.Operator, ImportHelper):
    """Load a Torque DIF File"""

    bl_idname = "import_scene.dif"
    bl_label = "Import DIF"
    bl_options = {"PRESET"}

    filename_ext = ".dif"
    filter_glob: StringProperty(
        default="*.dif",
        options={"HIDDEN"},
    )

    import_highest_lod_only: BoolProperty(
        name="Highest LOD Only",
        description="Import only the highest detail LOD level",
        default=False,
    )

    import_lightmaps: BoolProperty(
        name="Import Lightmaps",
        description="Import embedded lightmap textures",
        default=True,
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

        if bpy.data.is_saved and context.preferences.filepaths.use_relative_paths:
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
    filter_glob: StringProperty(
        default="*.dif",
        options={"HIDDEN"},
    )

    flip: BoolProperty(
        name="Flip faces",
        description="Flip normals of the faces, in case the resultant dif is inside out.",
        default=False,
    )

    double: BoolProperty(
        name="Double faces",
        description="Make all the faces double sided, may cause lag during collision detection.",
        default=False,
    )

    maxpolys: IntProperty(
        name="Polygons per DIF",
        description="Maximum number of polygons till a dif split is done",
        default=12000,
        min=1,
        max=12000,
    )

    applymodifiers: BoolProperty(
        name="Apply Modifiers",
        description="Apply modifiers during export",
        default=True,
    )

    exportvisible: BoolProperty(
        name="Export Visible",
        description="Export only visible geometry",
        default=True,
    )

    exportselected: BoolProperty(
        name="Export Selected",
        description="Export only selected geometry",
        default=False,
    )

    usematnames: BoolProperty(
        name="Use Material Names",
        description="Use material names instead of material texture file names",
        default=True,
    )

    mbonly: BoolProperty(
        name="Optimize for Marble Blast",
        description="Make the resultant DIF optimized for Marble Blast. Uncheck this if you want to use this for other Torque games.",
        default=True
    )

    bspmode: EnumProperty(
        items=[
            ("Fast", "Fast", "Use a sampling algorithm to determine the best splitter."),
            ("Exhaustive", "Exhaustive", "Use an exhaustive search algorithm to determine the best splitter. May take longer but builds more balanced trees."),
            ("None", "None", "Do not build a BSP Tree, utilize this for fast conversion times or if building the BSP Tree fails.")
        ],
        name="BSP Algorithm",
        description="The algorithm used for building the BSP Tree of the DIF.",
        default="Fast"
    )

    pointepsilon: FloatProperty(
        name="Point Epsilon",
        description="Minimum distance between two points to be considered equal.",
        default=1e-6
    )

    planeepsilon: FloatProperty(
        name="Plane Epsilon",
        description="Minimum difference between values of two plane to be considered equal.",
        default=1e-5
    )

    splitepsilon: FloatProperty(
        name="Split Epsilon",
        description="Minimum difference between values of two splitting planes to be considered equal.",
        default=1e-4
    )

    check_extension = True

    # def draw(self, context):
    #     layout = self.layout
    #     layout.prop(self, "flip")
    #     layout.prop(self, "double")
    #     layout.prop(self, "maxpolys")
    #     layout.prop(self, "applymodifiers")
    #     layout.prop(self, "exportvisible")
    #     layout.prop(self, "exportselected")
    #     layout.prop(self, "usematnames")
    #     layout.prop(self, "mbonly")
    #     layout.prop(self, "bspmode")
    #     layout.prop(self, "pointepsilon")
    #     layout.prop(self, "planeepsilon")
    #     layout.prop(self, "splitepsilon")
    #     layout.label(text="BSP Algorithms:")
    #     layout.label(text="Fast: Default mode, uses a sampling algorithm to determine the best splitter.")
    #     layout.label(text="Exhaustive: Tries to find the most optimal splits for BSP Tree. May take longer but it is deterministic.")
    #     layout.label(text="None: Do not generate BSP Tree")
    #     layout.label(text="If your geometry is too complex, consider using None mode as there is no guarantee that a BSP Tree can be optimally built within set constraints.")
    #     layout.label(text="BSP Trees are only used for Raycasts and Drop To Ground feature in Marble Blast. If you are not using these features, you can safely disable the BSP Tree.")

    def execute(self, context):
        from . import export_dif
        if bpy.app.version >= (4, 0, 0):
            bpy.types.VIEW3D_HT_header.append(progress_bar)
        keywords = self.as_keywords(ignore=("check_existing", "filter_glob"))
        export_dif.save(
            context,
            keywords["filepath"],
            keywords.get("flip", False),
            keywords.get("double", False),
            keywords.get("maxpolys", 12000),
            keywords.get("applymodifiers", True),
            keywords.get("exportvisible", True),
            keywords.get("exportselected", False),
            keywords.get("usematnames", False),
            keywords.get("mbonly", True),
            keywords.get("bspmode", "Fast"),
            keywords.get("pointepsilon", 1e-6),
            keywords.get("planeepsilon", 1e-5),
            keywords.get("splitepsilon", 1e-4),
        )
        stop_progress()

        return {"FINISHED"}

classes = (ExportDIF, ImportDIF, ImportCSX)


def menu_func_export_dif(self, context):
    self.layout.operator(ExportDIF.bl_idname, text="Torque (.dif)")


def menu_func_import_dif(self, context):
    self.layout.operator(ImportDIF.bl_idname, text="Torque (.dif)")


def menu_func_import_csx(self, context):
    self.layout.operator(ImportCSX.bl_idname, text="Torque Constructor (.csx)")


def progress_bar(self, context):
    row = self.layout.row()
    if bpy.app.version >= (4, 0, 0):
        row.progress(
            factor=progress_bar.progress,
            type="BAR",
            text=progress_bar.progress_text
        )
    row.scale_x = 2

def set_progress(progress, progress_text):
    delta = progress - progress_bar.progress
    if abs(delta) >= 0.1:
        progress_bar.progress = progress
        progress_bar.progress_text = progress_text
        bpy.ops.wm.redraw_timer(type='DRAW_WIN_SWAP', iterations=1)

def stop_progress():
    if bpy.app.version >= (4, 0, 0):
        bpy.types.VIEW3D_HT_header.remove(progress_bar)

progress_bar.progress = 0
progress_bar.progress_text = ""

def register():
    for cls in classes:
        bpy.utils.register_class(cls)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export_dif)
    bpy.types.TOPBAR_MT_file_import.append(menu_func_import_dif)
    bpy.types.TOPBAR_MT_file_import.append(menu_func_import_csx)
    # bpy.types.STATUSBAR_HT_header.append(progress_bar)
    bpy.utils.register_class(InteriorPanel)
    bpy.utils.register_class(AddCustomProperty)
    bpy.utils.register_class(DeleteCustomProperty)
    bpy.utils.register_class(InteriorKVP)
    bpy.utils.register_class(InteriorSettings)

    if platform.system() == "Windows":
        dllpath = os.path.join(
            os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.dll"
        )
    elif platform.system() == "Darwin":
        dllpath = os.path.join(
            os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.dylib"
        )
    elif platform.system() == "Linux":
        dllpath = os.path.join(
            os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.so"
        )
    if not os.path.isfile(dllpath):
        print(
            "Warning: DifBuilderLib not found - DIF export will be unavailable. "
            "DIF import will still work. For export support, download from: "
            "https://github.com/RandomityGuy/io_dif/releases"
        )

    bpy.types.Object.dif_props = PointerProperty(type=InteriorSettings)


def unregister():
    for cls in classes:
        bpy.utils.unregister_class(cls)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export_dif)
    bpy.types.TOPBAR_MT_file_import.remove(menu_func_import_dif)
    bpy.types.TOPBAR_MT_file_import.remove(menu_func_import_csx)
    bpy.utils.unregister_class(InteriorPanel)
    bpy.utils.unregister_class(AddCustomProperty)
    bpy.utils.unregister_class(DeleteCustomProperty)
    bpy.utils.unregister_class(InteriorKVP)
    bpy.utils.unregister_class(InteriorSettings)

    del bpy.types.Object.dif_props


if __name__ == "__main__":
    register()
