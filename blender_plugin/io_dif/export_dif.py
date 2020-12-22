from os.path import join
import bpy
import ctypes
import os
from pathlib import Path

from bpy.types import Curve, Image, Material, Mesh, Object, ShaderNodeTexImage
from bpy_extras.wm_utils.progress_report import ProgressReport, ProgressReportSubstep

dllpath = os.path.join(os.path.dirname(os.path.realpath(__file__)), "DifBuilderLib.dll")
difbuilderlib = ctypes.CDLL(dllpath)

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


class Dif:
    def __init__(self, ptr):
        self.__ptr__ = ptr

    def __del__(self):
        difbuilderlib.dispose_dif(self.__ptr__)

    def write_dif(self, path):
        difbuilderlib.write_dif(
            self.__ptr__, ctypes.create_string_buffer(path.encode("ascii"))
        )

    def add_game_entity(self, gameClass, datablock, position):
        vecarr = (ctypes.c_float * len(position))(*position)
        difbuilderlib.add_game_entity(
            self.__ptr__,
            ctypes.create_string_buffer(gameClass.encode("ascii")),
            ctypes.create_string_buffer(datablock.encode("ascii")),
            vecarr,
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

        uv1arr = (ctypes.c_float * len(uv1))(*uv1)
        uv2arr = (ctypes.c_float * len(uv2))(*uv2)
        uv3arr = (ctypes.c_float * len(uv3))(*uv3)

        narr = (ctypes.c_float * len(n))(*n)

        mat = ctypes.c_char_p(material.encode("ascii"))

        difbuilderlib.add_triangle(
            self.__ptr__, p3arr, p2arr, p1arr, uv3arr, uv2arr, uv1arr, narr, mat
        )

    def add_pathed_interior(self, dif: Dif, markerlist: MarkerList):
        difbuilderlib.add_pathed_interior(self.__ptr__, dif.__ptr__, markerlist.__ptr__)

    def build(self):
        return Dif(difbuilderlib.build(self.__ptr__))


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


def get_offset():
    obs = bpy.context.scene.objects
    minv = [1e9, 1e9, 1e9]
    maxv = [-1e9, -1e9, -1e9]

    for obj in obs:
        ob_eval = obj
        try:
            mesh = ob_eval.to_mesh()
        except RuntimeError:
            continue

        mesh.transform(obj.matrix_world)

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
    return (props.game_entity_datablock, props.game_entity_gameclass)


def save(
    context: bpy.types.Context,
    filepath: str = "",
    flip=False,
    double=False,
    maxtricount=16000,
):
    import bpy
    import bmesh

    obs = bpy.context.scene.objects

    builders = [DifBuilder()]

    difbuilder = builders[0]

    off = get_offset()

    tris = 0

    def save_mesh(obj: Object, mesh: Mesh, offset, flip=False, double=False):
        import bpy

        nonlocal tris, difbuilder

        mesh_triangulate(mesh)

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

    for ob in obs:
        ob_eval = ob

        dif_props = ob_eval.dif_props

        if dif_props.interior_type == "game_entity":
            game_entities.append(ob_eval)

        try:
            me = ob_eval.to_mesh()
        except RuntimeError:
            continue

        if dif_props.interior_type == "static_interior":
            me.transform(ob.matrix_world)
            save_mesh(ob, me, off, flip, double)

        if dif_props.interior_type == "pathed_interior":
            mp_list.append((ob_eval, dif_props.marker_path))

    mp_difs = []

    for (mp, curve) in mp_list:
        mp_difs.append(build_pathed_interior(mp, curve, off, flip, double))

    if tris != 0:
        for i in range(0, len(builders)):
            if i == 0:
                for (mpdif, markerlist) in mp_difs:
                    builders[i].add_pathed_interior(mpdif, markerlist)

            dif = builders[i].build()

            if i == 0:
                for ge in game_entities:
                    dif.add_game_entity(
                        ge.dif_props.game_entity_gameclass,
                        ge.dif_props.game_entity_datablock,
                        [ge.location[i] + off[i] for i in range(0, 3)],
                    )

            dif.write_dif(str(Path(filepath).with_suffix("")) + str(i) + ".dif")
