from os.path import join
from typing import Dict
import bpy
import ctypes
import os
from pathlib import Path

from bpy.types import Curve, Image, Material, Mesh, Object, ShaderNodeTexImage
from bpy_extras.wm_utils.progress_report import ProgressReport, ProgressReportSubstep
from mathutils import Quaternion, Vector

dllpath = os.path.join(os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.dll")
difbuilderlib = None
try:
    difbuilderlib = ctypes.CDLL(dllpath)
except:
    raise Exception(
        "There was an error loading the necessary dll required for dif export. Please download the plugin from the proper location: https://github.com/RandomityGuy/io_dif/releases"
    )

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
difbuilderlib.build.argtypes = [ctypes.c_void_p]
difbuilderlib.build.restype = ctypes.c_void_p

difbuilderlib.dispose_dif.argtypes = [ctypes.c_void_p]
difbuilderlib.write_dif.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

difbuilderlib.add_pathed_interior.argtypes = [
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_void_p,
]

difbuilderlib.new_marker_list.restype = ctypes.c_void_p
difbuilderlib.dispose_marker_list.argtypes = [ctypes.c_void_p]
difbuilderlib.push_marker.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_float),
    ctypes.c_int,
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
    ctypes.c_char_p,
    ctypes.c_char_p,
    ctypes.c_void_p,
]


scene = bpy.context.scene

obj = bpy.context.active_object


class MarkerList:
    def __init__(self):
        self.__ptr__ = difbuilderlib.new_marker_list()

    def __del__(self):
        difbuilderlib.dispose_marker_list(self.__ptr__)

    def push_marker(self, vec, msToNext, initialPathPosition):
        vecarr = (ctypes.c_float * len(vec))(*vec)
        difbuilderlib.push_marker(self.__ptr__, vecarr, msToNext, initialPathPosition)


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

    def add_game_entity(self, gameClass, datablock, position, scale, properties: dict):
        vecarr = (ctypes.c_float * len(position))(*position)
        propertydict = DIFDict()
        for key in properties:
            propertydict.add_kvp(key, properties[key])
        propertydict.add_kvp("scale", "%.5f %.5f %.5f" % (scale[0], scale[1], scale[2]))
        if gameClass == "Trigger":
            propertydict.add_kvp("polyhedron", "0 0 0 1 0 0 0 -1 0 0 0 1")
        difbuilderlib.add_game_entity(
            self.__ptr__,
            ctypes.create_string_buffer(gameClass.encode("ascii")),
            ctypes.create_string_buffer(datablock.encode("ascii")),
            vecarr,
            propertydict.__ptr__,
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

        #Debug lines 
        # print((p3arr, p2arr, p1arr, uv3arr, uv2arr, uv1arr, narr, mat))
        #Cont
        difbuilderlib.add_triangle(
            self.__ptr__, p3arr, p2arr, p1arr, uv3arr, uv2arr, uv1arr, narr, mat
        )

    def add_pathed_interior(self, dif: Dif, markerlist: MarkerList):
        difbuilderlib.add_pathed_interior(self.__ptr__, dif.__ptr__, markerlist.__ptr__)

    # NONFUNCTIONAL, TRIGGERS ARENT GETTING CREATED WHEN PRESSING CREATE SUBS
    def add_trigger(self, datablock, name, position, scale, props: DIFDict):
        posarr = (ctypes.c_float * len(position))(*position)
        props.add_kvp("scale", f"{scale[0]} {scale[1]} {scale[2]}")
        difbuilderlib.add_trigger(
            self.__ptr__,
            posarr,
            ctypes.create_string_buffer(name.encode("ascii")),
            ctypes.create_string_buffer(datablock.encode("ascii")),
            props.__ptr__,
        )

    def build(self):
        
        try:
            built = difbuilderlib.build(self.__ptr__)
        except OSError as e:
            print(e)
            print(e.winerror)
            print(e.strerror)
            raise e
        try:
            return Dif(built)
        except OSError as e:
            print(e)
            print(e.winerror)
            print(e.strerror)
            raise e


def mesh_triangulate(me):
    import bmesh

    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()


def resolve_texture(mat: Material):
    img: ShaderNodeTexImage = None
    for n in mat.node_tree.nodes:
        if n.type == "TEX_IMAGE":
            img = n
            break

    if img == None:
        return mat.name

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

    off = [((maxv[i] - minv[i]) / 2) + 50 for i in range(0, 3)]
    return off


def build_pathed_interior(ob: Object, marker_ob: Curve, offset, flip, double):
    difbuilder = DifBuilder()
    mesh = ob.to_mesh()
    mesh_triangulate(mesh)

    mesh_verts = mesh.vertices

    active_uv_layer = mesh.uv_layers.active.data

    for poly in mesh.polygons:

        rawp1 = mesh_verts[poly.vertices[0]].co
        rawp2 = mesh_verts[poly.vertices[1]].co
        rawp3 = mesh_verts[poly.vertices[2]].co

        p1 = [rawp1[i] + offset[i] for i in range(0, 3)]
        p2 = [rawp2[i] + offset[i] for i in range(0, 3)]
        p3 = [rawp3[i] + offset[i] for i in range(0, 3)]

        uv = [
            active_uv_layer[l].uv[:]
            for l in range(poly.loop_start, poly.loop_start + poly.loop_total)
        ]

        uv1 = uv[0]
        uv2 = uv[1]
        uv3 = uv[2]

        n = mesh_verts[poly.vertices[0]].normal

        material = (
            resolve_texture(mesh.materials[poly.material_index])
            if poly.material_index != None
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

    dif = difbuilder.build()

    marker_pts = (
        marker_ob.splines[0].bezier_points
        if (len(marker_ob.splines[0].bezier_points) != 0)
        else marker_ob.splines[0].points
    )
    msToNext = int((marker_ob.path_duration / len(marker_pts)))
    initialPathPosition = int(marker_ob.eval_time)

    marker_list = MarkerList()

    for pt in marker_pts:
        marker_list.push_marker(pt.co, msToNext, initialPathPosition)

    return (dif, marker_list)


def build_game_entity(ob: Object):
    props = ob.dif_props
    propertydict = {}
    for prop in props.game_entity_properties:
        propertydict[prop.key] = prop.value

    axis_ang_raw: Vector = ob.matrix_world.to_quaternion().to_axis_angle()
    axis_ang = (
        axis_ang_raw[1],
        axis_ang_raw[0][0],
        axis_ang_raw[0][1],
        axis_ang_raw[0][2],
    )

    return (
        props.game_entity_datablock,
        props.game_entity_gameclass,
        propertydict,
        ob.scale,
    )


def save(
    context: bpy.types.Context,
    filepath: str = "",
    flip=False,
    double=False,
    maxtricount=12000,
    applymodifiers=True,
    exportvisible=True,
    exportselected=False,
):
    import bpy
    import bmesh

    #bpy.ops.object.duplicates_make_real()
    
    #def get_objects():
    #    return bpy.context.selected_objects if exportselected else bpy.context.scene.objects

    #obs = get_objects()
    obs = bpy.context.selected_objects if exportselected else bpy.context.scene.objects

    builders = [DifBuilder()]

    difbuilder = builders[0]

    depsgraph = context.evaluated_depsgraph_get()

    off = [0, 0, 0] #get_offset(depsgraph, applymodifiers)

    tris = 0

    def save_mesh(obj: Object, mesh: Mesh, offset, flip=False, double=False):
        import bpy

        nonlocal tris, difbuilder

        mesh_triangulate(mesh)
        
        print(mesh)
        print(mesh.uv_layers)
        print(mesh.uv_layers.items())

        mesh_verts = mesh.vertices

        active_uv_layer = mesh.uv_layers.active.data

        for poly in mesh.polygons:

            if tris > maxtricount:
                tris = 0
                builders.append(DifBuilder())
                difbuilder = builders[-1]

            rawp1 = mesh_verts[poly.vertices[0]].co
            rawp2 = mesh_verts[poly.vertices[1]].co
            rawp3 = mesh_verts[poly.vertices[2]].co

            p1 = [rawp1[i] + offset[i] for i in range(0, 3)]
            p2 = [rawp2[i] + offset[i] for i in range(0, 3)]
            p3 = [rawp3[i] + offset[i] for i in range(0, 3)]

            uv = [
                active_uv_layer[l].uv[:]
                for l in range(poly.loop_start, poly.loop_start + poly.loop_total)
            ]

            uv1 = uv[0]
            uv2 = uv[1]
            uv3 = uv[2]

            n = mesh_verts[poly.vertices[0]].normal

            material = (
                resolve_texture(mesh.materials[poly.material_index])
                if poly.material_index != None
                else "NULL"
            )

            # Debug
            print((p1, p2, p3, uv1, uv2, uv3, n, material))

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
    game_entities: list[Object] = []
    
    def is_object_instance_selected(object_instance):
        # For instanced objects we check selection of their instancer (more accurately: check
        # selection status of the original object corresponding to the instancer).
        if object_instance.parent:
            return object_instance.parent.original.select_get()
        # For non-instanced objects we check selection state of the original object.
        return object_instance.object.original.select_get()
    
    def is_object_instance_visible(object_instance):
        # For instanced objects we check visibility of their instancer (more accurately: check
        # vsibility status of the original object corresponding to the instancer).
        if object_instance.parent:
            return object_instance.parent.original.visible_get()
        # For non-instanced objects we check visibility state of the original object.
        return object_instance.object.original.visible_get()
    
    def create_mesh_for_object_instance(object_instance):
        if applymodifiers:
            return object_instance.object.to_mesh()
        else:
            return object_instance.object.original.to_mesh()

    #for ob in obs:
    for object_instance in depsgraph.object_instances:
        if exportselected:
            if not is_object_instance_selected(object_instance):
                continue
        if exportvisible:
            #if not ob.visible_get():
            if not is_object_instance_visible(object_instance):
                continue

        #ob_eval = ob.evaluated_get(depsgraph) if applymodifiers else ob
        ob_eval = object_instance.object if applymodifiers else object_instance.object.original

        dif_props = ob_eval.dif_props
        #dif_props = object_instance.object.dif_props

        if dif_props.interior_type == "game_entity":
            game_entities.append(ob_eval)

        try:
            me = ob_eval.to_mesh()
            #me = create_mesh_for_object_instance(object_instance)
        except RuntimeError:
            continue

        if dif_props.interior_type == "static_interior":
            me.transform(ob_eval.matrix_world)
            try:
                save_mesh(ob_eval, me, off, flip, double)
            except:
                print("Skipping mesh")
                continue
            

        if dif_props.interior_type == "pathed_interior":
            mp_list.append((ob_eval, dif_props.marker_path))

    mp_difs = []

    for (mp, curve) in mp_list:
        mp_difs.append(build_pathed_interior(mp, curve, off, flip, double))

    #DEBUG
    print("Generated tris:")
    print(tris)

    if tris != 0:
        for i in range(0, len(builders)):
            if i == 0:
                for (mpdif, markerlist) in mp_difs:
                    builders[i].add_pathed_interior(mpdif, markerlist)

            dif = builders[i].build()

            if i == 0:
                for ge in game_entities:
                    entity = build_game_entity(ge)
                    dif.add_game_entity(
                        entity[1],
                        entity[0],
                        [ge.location[i] + off[i] for i in range(0, 3)],
                        entity[3],
                        entity[2],
                    )

            dif.write_dif(str(Path(filepath).with_suffix("")) + str(i) + ".dif")
