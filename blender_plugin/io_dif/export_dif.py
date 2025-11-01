from os.path import join
import re
from typing import Dict
from . import set_progress, stop_progress
import bpy
import ctypes
import os
import platform
from pathlib import Path

from bpy.types import Curve, Image, Material, Mesh, Object, ShaderNodeTexImage
from bpy_extras.wm_utils.progress_report import ProgressReport, ProgressReportSubstep
from mathutils import Quaternion, Vector, Matrix

if platform.system() == "Windows":
    dllpath = os.path.join(os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.dll")
elif platform.system() == "Darwin":
    dllpath = os.path.join(os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.dylib")
elif platform.system() == "Linux":
    dllpath = os.path.join(os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.so")
difbuilderlib = None
try:
    difbuilderlib = ctypes.CDLL(dllpath)
except:
    raise Exception(
        "There was an error loading the necessary dll required for dif export. Please download the plugin from the proper location: https://github.com/RandomityGuy/io_dif/releases"
    )

STATUSFN = ctypes.CFUNCTYPE(None, ctypes.c_bool, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_char_p, ctypes.c_char_p)

difbuilderlib.new_difbuilder.restype = ctypes.c_void_p
difbuilderlib.dispose_difbuilder.argtypes = [ctypes.c_void_p]
difbuilderlib.add_triangle.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_char_p,
]
difbuilderlib.build.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_int32, ctypes.c_float, ctypes.c_float, ctypes.c_float, STATUSFN]
difbuilderlib.build.restype = ctypes.c_void_p

difbuilderlib.dispose_dif.argtypes = [ctypes.c_void_p]
difbuilderlib.write_dif.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

difbuilderlib.add_pathed_interior.argtypes = [
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_float),
]

difbuilderlib.new_marker_list.restype = ctypes.c_void_p
difbuilderlib.dispose_marker_list.argtypes = [ctypes.c_void_p]
difbuilderlib.push_marker.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_int,
    ctypes.c_int,
]
difbuilderlib.new_trigger_id_list.restype = ctypes.c_void_p
difbuilderlib.dispose_trigger_id_list.argtypes = [ctypes.c_void_p]
difbuilderlib.push_trigger_id.argtypes = [
    ctypes.c_void_p,
    ctypes.c_int,
]

difbuilderlib.add_game_entity.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.c_char_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_void_p,
]
difbuilderlib.new_dict.restype = ctypes.c_void_p
difbuilderlib.dispose_dict.argtypes = [ctypes.c_void_p]
difbuilderlib.add_dict_kvp.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.c_char_p,
]
difbuilderlib.add_trigger.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_char_p,
    ctypes.c_char_p,
    ctypes.c_void_p,
]

current_status = (False, 0, 0, "", "")

def update_status(stop, current, total, status, finish_status):
    global current_status
    current_status = (stop, current, total, status.decode('utf-8'), finish_status.decode('utf-8'))
    set_progress(current / total if total != 0 else 1, status.decode('utf-8'))
    if stop:
        stop_progress()

update_status_c = STATUSFN(update_status)

class MarkerList:
    def __init__(self):
        self.__ptr__ = difbuilderlib.new_marker_list()

    def __del__(self):
        difbuilderlib.dispose_marker_list(self.__ptr__)

    def push_marker(self, vec, msToNext, smoothing_type):
        vecarr = (ctypes.c_float * len(vec))(*vec)
        difbuilderlib.push_marker(self.__ptr__, vecarr, msToNext, smoothing_type)


class TriggerIDList:
    def __init__(self):
        self.__ptr__ = difbuilderlib.new_trigger_id_list()

    def __del__(self):
        difbuilderlib.dispose_trigger_id_list(self.__ptr__)

    def push_trigger_id(self, num):
        difbuilderlib.push_trigger_id(self.__ptr__, num)


class DIFDict:
    def __init__(self):
        self.__ptr__ = difbuilderlib.new_dict()

    def __del__(self):
        difbuilderlib.dispose_dict(self.__ptr__)

    def add_kvp(self, key, value):
        difbuilderlib.add_dict_kvp(
            self.__ptr__,
            ctypes.create_string_buffer(key.encode("ascii")),
            ctypes.create_string_buffer(value.encode("ascii")),
        )


class Dif:
    def __init__(self, ptr):
        self.__ptr__ = ptr

    def __del__(self):
        difbuilderlib.dispose_dif(self.__ptr__)

    def write_dif(self, path):
        difbuilderlib.write_dif(
            self.__ptr__, ctypes.create_string_buffer(path.encode("ascii"))
        )

    def add_game_entity(self, entity):
        vecarr = (ctypes.c_float * len(entity.position))(*entity.position)
        difbuilderlib.add_game_entity(
            self.__ptr__,
            ctypes.create_string_buffer(entity.gameclass.encode("ascii")),
            ctypes.create_string_buffer(entity.datablock.encode("ascii")),
            vecarr,
            entity.properties.__ptr__,
        )

    def add_trigger(self, trigger):
        pos_vecarr = (ctypes.c_float * len(trigger.position))(*trigger.position)
        size_vecarr = (ctypes.c_float * len(trigger.size))(*trigger.size)
        difbuilderlib.add_trigger(
            self.__ptr__,
            pos_vecarr,
            size_vecarr,
            ctypes.create_string_buffer(trigger.name.encode("ascii")),
            ctypes.create_string_buffer(trigger.datablock.encode("ascii")),
            trigger.properties.__ptr__,
        )


class DifBuilder:
    def __init__(self):
        self.__ptr__ = difbuilderlib.new_difbuilder()

    def __del__(self):
        difbuilderlib.dispose_difbuilder(self.__ptr__)

    def add_triangle(self, p1, p2, p3, uv1, uv2, uv3, n, material):
        p1arr = (ctypes.c_float * len(p1))(*p1)
        p2arr = (ctypes.c_float * len(p2))(*p2)
        p3arr = (ctypes.c_float * len(p3))(*p3)

        uv1 = (uv1[0], -uv1[1])
        uv2 = (uv2[0], -uv2[1])
        uv3 = (uv3[0], -uv3[1])

        uv1arr = (ctypes.c_float * len(uv1))(*uv1)
        uv2arr = (ctypes.c_float * len(uv2))(*uv2)
        uv3arr = (ctypes.c_float * len(uv3))(*uv3)

        narr = (ctypes.c_float * len(n))(*n)

        mat = ctypes.c_char_p(material.encode("ascii"))

        difbuilderlib.add_triangle(
            self.__ptr__, p3arr, p2arr, p1arr, uv3arr, uv2arr, uv1arr, narr, mat
        )

    def add_pathed_interior(self, mp):
        vecarr = (ctypes.c_float * len(mp.offset))(*mp.offset)
        difbuilderlib.add_pathed_interior(self.__ptr__, mp.dif.__ptr__, mp.marker_list.__ptr__, mp.trigger_id_list.__ptr__, mp.properties.__ptr__, vecarr)

    def build(self, mbonly, bspmode, pointepsilon, planeepsilon, splitepsilon):
        return Dif(difbuilderlib.build(self.__ptr__, mbonly, bspmode, pointepsilon, planeepsilon, splitepsilon, update_status_c))


def mesh_triangulate(me):
    import bmesh

    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()


def resolve_texture(mat: Material, usematnames: bool):
    if usematnames:
        matname = mat.name
        # Strip off the .\d+ extension
        matname = re.sub(r"\.\d+$", "", matname)
        return matname
    img: ShaderNodeTexImage = None
    for n in mat.node_tree.nodes:
        if n.type == "TEX_IMAGE":
            img = n
            break

    if img == None:
        matname = mat.name
        # Strip off the .\d+ extension
        matname = re.sub(r"\.\d+$", "", matname)
        return matname

    return Path(img.image.filepath).stem


def get_offset(depsgraph, applymodifiers=True):
    obs = bpy.context.scene.objects
    minv = [1e9, 1e9, 1e9]
    maxv = [-1e9, -1e9, -1e9]

    for obj in obs:
        ob_eval = obj.evaluated_get(depsgraph) if applymodifiers else obj
        try:
            mesh = ob_eval.to_mesh()
        except RuntimeError:
            continue

        mesh.transform(ob_eval.matrix_world)

        for vert in mesh.vertices:
            for i in range(0, 3):
                if minv[i] > vert.co[i]:
                    minv[i] = vert.co[i]
                if maxv[i] < vert.co[i]:
                    maxv[i] = vert.co[i]

        ob_eval.to_mesh_clear()

    off = [((maxv[i] - minv[i]) / 2) + 50 for i in range(0, 3)]
    return off


def formatScale(scale):
    return "%.5f %.5f %.5f" % (scale[0], scale[1], scale[2])


def formatRotation(axis_ang):
    from math import degrees
    return "%.5f %.5f %.5f %.5f" % (
            axis_ang[0][0], 
            axis_ang[0][1], 
            axis_ang[0][2],
            degrees(-axis_ang[1]))
 
def is_degenerate_triangle(p1: Vector, p2: Vector, p3: Vector):
    return (p1 - p2).cross(p1 - p3).length < 1e-6

class GamePathedInterior:
    def __init__(self, ob: Object, triggers: list[Object], offset, flip, double, usematnames, mbonly=True, bspmode="Fast", pointepsilon=1e-6, planeepsilon=1e-5, splitepsilon=1e-4):
        difbuilder = DifBuilder()

        mesh = ob.to_mesh()

        mesh.calc_loop_triangles()
        if bpy.app.version < (4, 0, 0):
            mesh.calc_normals_split()

        mesh_verts = mesh.vertices

        if mesh.uv_layers != None and mesh.uv_layers.active != None:
            active_uv_layer = mesh.uv_layers.active.data
        else:
            active_uv_layer = mesh.attributes.get('UVMap')

        for tri_idx in mesh.loop_triangles:
            tri: bpy.types.MeshLoopTriangle = tri_idx

            rawp1 = mesh_verts[tri.vertices[0]].co
            rawp2 = mesh_verts[tri.vertices[1]].co
            rawp3 = mesh_verts[tri.vertices[2]].co

            if is_degenerate_triangle(Vector(rawp1), Vector(rawp2), Vector(rawp3)):
                continue

            p1 = [rawp1[i] + offset[i] for i in range(0, 3)]
            p2 = [rawp2[i] + offset[i] for i in range(0, 3)]
            p3 = [rawp3[i] + offset[i] for i in range(0, 3)]

            uv1 = active_uv_layer[tri.loops[0]].uv[:]
            uv2 = active_uv_layer[tri.loops[1]].uv[:]
            uv3 = active_uv_layer[tri.loops[2]].uv[:]

            n = tri.normal

            material = (
                resolve_texture(mesh.materials[tri.material_index], usematnames)
                if tri.material_index != None
                else "NULL"
            )

            if not flip:
                difbuilder.add_triangle(p1, p2, p3, uv1, uv2, uv3, n, material)
                if double:
                    difbuilder.add_triangle(p3, p2, p1, uv3, uv2, uv1, n, material)
            else:
                difbuilder.add_triangle(p3, p2, p1, uv3, uv2, uv1, n, material)
                if double:
                    difbuilder.add_triangle(p1, p2, p3, uv1, uv2, uv3, n, material)

        bspvalue = None
        if bspmode == "Fast":
            bspvalue = 0
        elif bspmode == "Exhaustive":
            bspvalue = 1
        else:
            bspvalue = 2

        dif = difbuilder.build(mbonly, bspvalue, pointepsilon, planeepsilon, splitepsilon)

        marker_ob = ob.dif_props.marker_path

        marker_list = MarkerList()

        if(marker_ob):
            marker_pts = (
                marker_ob.splines[0].bezier_points
                if (len(marker_ob.splines[0].bezier_points) != 0)
                else marker_ob.splines[0].points
            )

            path_type = ob.dif_props.marker_type
            if path_type == "linear":
                smoothing_type = 0
            elif path_type == "spline":
                smoothing_type = 1
            elif path_type == "accelerate":
                smoothing_type = 2

            curve_transform = None
            curve_obj = next((obj for obj in bpy.data.objects if obj.data == marker_ob.original), None)
            if curve_obj:
                curve_transform = curve_obj.matrix_world

            cum_times = [0] # Used for "target marker" triggers and "start index"

            for index, pt in enumerate(marker_pts):
                if index == len(marker_pts)-1:
                    msToNext = 0
                else:
                    if(ob.dif_props.constant_speed):
                        p0 = Vector(marker_pts[index].co[:3])
                        p1 = Vector(marker_pts[index+1].co[:3])
                        marker_dist = (p1 - p0).length

                        if(ob.dif_props.marker_type == "spline"):
                            p0 = marker_pts[index-1].co[:3]
                            p1 = marker_pts[index].co[:3]
                            p2 = marker_pts[index+1].co[:3]
                            p3 = marker_pts[(index+2) % len(marker_pts)].co[:3]
                            length = GamePathedInterior.catmull_rom_length(p0, p1, p2, p3)
                        else:
                            length = marker_dist
                        
                        if(marker_dist < 0.01):
                            msToNext = ob.dif_props.pause_duration
                        else:
                            msToNext = length / (ob.dif_props.speed / 1000)

                    else:
                        msToNext = ob.dif_props.total_time / (len(marker_pts)-1)

                    msToNext = int(max(msToNext, 1))

                co = pt.co
                if len(co) == 4:
                    co = Vector((co.x, co.y, co.z))

                if(curve_transform):
                    co = curve_transform @ co

                marker_list.push_marker(co, msToNext, smoothing_type)

                cum_times.append(cum_times[-1]+msToNext)

        else:
            marker_list.push_marker(ob.location, ob.dif_props.total_time, 0)
            marker_list.push_marker(ob.location, 0, 0)

        trigger_id_list = TriggerIDList()

        if(ob.dif_props.constant_speed):
            marker_idx = min(ob.dif_props.start_index, len(cum_times)-1)
            starting_time = cum_times[marker_idx]
        else:
            starting_time = ob.dif_props.start_time

        if(ob.dif_props.reverse):
            initial_target_position = -2
        else:
            initial_target_position = -1

        for index, trigger in enumerate(triggers):
            if trigger.target_object is ob:
                trigger_id_list.push_trigger_id(index)
                initial_target_position = starting_time

                # Update the trigger target time if using "target marker"
                if(trigger.target_marker): 
                    marker_idx = min(trigger.target_index, len(cum_times)-1)
                    trigger.properties.add_kvp("targetTime", str(cum_times[marker_idx]))
                    trigger.name = "MustChange_m" + str(marker_idx)

        ob.to_mesh_clear()

        self.dif = dif
        self.marker_list = marker_list
        self.trigger_id_list = trigger_id_list

        propertydict = DIFDict()
        propertydict.add_kvp("initialTargetPosition", str(initial_target_position))
        propertydict.add_kvp("initialPosition", str(starting_time))

        if(ob.matrix_world != Matrix.Identity(4)):
            propertydict.add_kvp("baseScale", formatScale(ob.scale))
            axis_ang_raw: Vector = ob.matrix_world.to_quaternion().to_axis_angle()
            propertydict.add_kvp("baseRotation", formatRotation(axis_ang_raw))

        self.properties = propertydict
        self.offset = [-(ob.location[i] + offset[i]) for i in range(0, 3)]

    @staticmethod
    def catmull_rom(t, p0, p1, p2, p3):
        return 0.5 * ((3*p1 - 3*p2 + p3 - p0)*t*t*t
            + (2*p0 - 5*p1 + 4*p2 - p3)*t*t
            + (p2 - p0)*t
            + 2*p1)
    
    @staticmethod
    def catmull_rom_length(p0, p1, p2, p3, samples=20):
        total_length = 0
        last_vec = None

        for i in range(0, samples+1):
            t = i / samples
            x = GamePathedInterior.catmull_rom(t, p0[0], p1[0], p2[0], p3[0])
            y = GamePathedInterior.catmull_rom(t, p0[1], p1[1], p2[1], p3[1])
            z = GamePathedInterior.catmull_rom(t, p0[2], p1[2], p2[2], p3[2])
            new_vec = Vector((x, y, z))
            if last_vec:
                total_length += (new_vec - last_vec).length
            last_vec = new_vec

        return total_length
    

class GameEntity:
    def __init__(self, ob, offset):
        props = ob.dif_props

        propertydict = DIFDict()
        for prop in props.game_entity_properties:
            propertydict.add_kvp(prop.key, prop.value)

        propertydict.add_kvp("scale", formatScale(ob.scale))

        axis_ang_raw: Vector = ob.matrix_world.to_quaternion().to_axis_angle()
        propertydict.add_kvp("rotation", formatRotation(axis_ang_raw))

        if props.game_entity_gameclass == "Trigger":
            propertydict.add_kvp("polyhedron", "0 0 0 1 0 0 0 -1 0 0 0 1")

        self.position = [ob.location[i] + offset[i] for i in range(0, 3)]
        self.datablock = props.game_entity_datablock
        self.gameclass = props.game_entity_gameclass
        self.properties = propertydict


class GameTrigger:
    def __init__(self, ob, offset):
        props = ob.dif_props

        propertydict = DIFDict()
        for prop in props.game_entity_properties:
            propertydict.add_kvp(prop.key, prop.value)

        #axis_ang_raw: Vector = ob.matrix_world.to_quaternion().to_axis_angle()
        #propertydict.add_kvp("rotation", formatRotation(axis_ang_raw))

        self.position = [ob.location[i] + offset[i] for i in range(0, 3)]
        self.size = [ob.scale[0], -ob.scale[1], ob.scale[2]]
        self.datablock = props.game_entity_datablock
        self.properties = propertydict
        self.name = "MustChange"

        self.target_object = ob.dif_props.pathed_interior_target
        self.target_marker = ob.dif_props.target_marker
        self.target_index = ob.dif_props.target_index


def save(
    context: bpy.types.Context,
    filepath: str = "",
    flip=False,
    double=False,
    maxtricount=12000,
    applymodifiers=True,
    exportvisible=True,
    exportselected=False,
    usematnames=False,
    mbonly=True,
    bspmode="Fast",
    pointepsilon=1e-6,
    planeepsilon=1e-5,
    splitepsilon=1e-4
):
    import bpy
    import bmesh

    builders = [DifBuilder()]

    difbuilder = builders[0]

    depsgraph = context.evaluated_depsgraph_get()

    off = [0, 0, 0]  # get_offset(depsgraph, applymodifiers)

    tris = 0

    def save_mesh(obj: Object, mesh: Mesh, offset, flip=False, double=False):
        import bpy

        nonlocal tris, difbuilder

        mesh.calc_loop_triangles()
        if bpy.app.version < (4, 0, 0):
            mesh.calc_normals_split()

        mesh_verts = mesh.vertices

        if mesh.uv_layers != None and mesh.uv_layers.active != None:
            active_uv_layer = mesh.uv_layers.active.data
        else:
            active_uv_layer = mesh.attributes.get('UVMap')

        for tri_idx in mesh.loop_triangles:

            tri: bpy.types.MeshLoopTriangle = tri_idx

            if tris > maxtricount:
                tris = 0
                builders.append(DifBuilder())
                difbuilder = builders[-1]

            rawp1 = mesh_verts[tri.vertices[0]].co
            rawp2 = mesh_verts[tri.vertices[1]].co
            rawp3 = mesh_verts[tri.vertices[2]].co

            if is_degenerate_triangle(Vector(rawp1), Vector(rawp2), Vector(rawp3)):
                continue

            p1 = [rawp1[i] + offset[i] for i in range(0, 3)]
            p2 = [rawp2[i] + offset[i] for i in range(0, 3)]
            p3 = [rawp3[i] + offset[i] for i in range(0, 3)]

            # uv = [
            #     active_uv_layer[l].uv[:]
            #     for l in range(poly.loop_start, poly.loop_start + poly.loop_total)
            # ]

            uv1 = active_uv_layer[tri.loops[0]].uv[:]
            uv2 = active_uv_layer[tri.loops[1]].uv[:]
            uv3 = active_uv_layer[tri.loops[2]].uv[:]

            n = tri.normal

            material = (
                resolve_texture(mesh.materials[tri.material_index], usematnames)
                if tri.material_index != None
                else "NULL"
            )

            if not flip:
                difbuilder.add_triangle(p1, p2, p3, uv1, uv2, uv3, n, material)
                tris += 1
                if double:
                    difbuilder.add_triangle(p3, p2, p1, uv3, uv2, uv1, n, material)
                    tris += 1
            else:
                difbuilder.add_triangle(p3, p2, p1, uv3, uv2, uv1, n, material)
                tris += 1
                if double:
                    difbuilder.add_triangle(p1, p2, p3, uv1, uv2, uv3, n, material)
                    tris += 1

    mp_list = []
    game_entities: list[GameEntity] = []
    triggers: list[GameTrigger] = []

    def is_object_instance_selected(object_instance):
        # For instanced objects we check selection of their instancer (more accurately: check
        # selection status of the original object corresponding to the instancer).
        if object_instance.parent:
            return object_instance.parent.original.select_get()
        # For non-instanced objects we check selection state of the original object.
        return object_instance.object.original.select_get()

    def is_object_instance_visible(object_instance):
        # For instanced objects we check visibility of their instancer (more accurately: check
        # visibility status of the original object corresponding to the instancer).
        if object_instance.parent:
            return object_instance.parent.original.visible_get()
        # For non-instanced objects we check visibility state of the original object.
        return object_instance.object.original.visible_get()

    # handle normal export for lower versions
    if bpy.app.version < (3, 1, 0) or not applymodifiers:
        obs = (
            bpy.context.selected_objects
            if exportselected
            else bpy.context.scene.objects
        )
        for ob in obs:
            ob: Object = ob
            if exportvisible:
                if not ob.visible_get():
                    continue

            ob_eval = ob.evaluated_get(depsgraph) if applymodifiers else ob

            dif_props = ob_eval.dif_props

            if dif_props.interior_type == "game_entity":
                game_entities.append(GameEntity(ob_eval, off))
                
            if dif_props.interior_type == "path_trigger":
                triggers.append(GameTrigger(ob_eval, off))

            try:
                me = ob_eval.to_mesh()
            except RuntimeError:
                continue

            if dif_props.interior_type == "static_interior":
                me.transform(ob_eval.matrix_world)
                try:
                    save_mesh(ob_eval, me, off, flip, double)
                except:
                    print("Skipping mesh due to issue while saving")

            ob_eval.to_mesh_clear()

            if dif_props.interior_type == "pathed_interior":
                mp_list.append(ob_eval)

    # handle object instances for these versions, ew code duplication
    if bpy.app.version >= (3, 1, 0) and applymodifiers:
        for object_instance in depsgraph.object_instances:
            if exportselected:
                if not is_object_instance_selected(object_instance):
                    continue

            if exportvisible:
                if not is_object_instance_visible(object_instance):
                    continue

            ob_eval = (
                object_instance.object
                if applymodifiers
                else object_instance.object.original
            )

            dif_props = ob_eval.dif_props

            if dif_props.interior_type == "game_entity":
                game_entities.append(GameEntity(ob_eval, off))
                
            if dif_props.interior_type == "path_trigger":
                triggers.append(GameTrigger(ob_eval, off))

            try:
                me = ob_eval.to_mesh()
            except RuntimeError:
                print("Skipping mesh due to bad eval")
                continue

            if dif_props.interior_type == "static_interior":
                me.transform(ob_eval.matrix_world)
                try:
                    save_mesh(ob_eval, me, off, flip, double)
                except:
                    print("Skipping mesh due to issue while saving")

            ob_eval.to_mesh_clear()

            if dif_props.interior_type == "pathed_interior":
                mp_list.append(ob_eval)

    mp_difs = []

    for mp in mp_list:
        mp_difs.append(GamePathedInterior(mp, triggers, off, flip, double, usematnames, mbonly, bspmode, pointepsilon, planeepsilon, splitepsilon))

    bspvalue = None
    if bspmode == "Fast":
        bspvalue = 0
    elif bspmode == "Exhaustive":
        bspvalue = 1
    else:
        bspvalue = 2

    if tris != 0:
        for i in range(0, len(builders)):
            if i == 0:
                for mp in mp_difs:
                    builders[i].add_pathed_interior(mp)

            dif = builders[i].build(mbonly, bspvalue, pointepsilon, planeepsilon, splitepsilon)

            if i == 0:
                for ge in game_entities:
                    dif.add_game_entity(ge)

                for trigger in triggers:
                    dif.add_trigger(trigger)

            dif.write_dif(str(Path(filepath).with_suffix("")) + str(i) + ".dif")
