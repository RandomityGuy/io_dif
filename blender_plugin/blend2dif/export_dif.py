from os.path import join
import bpy
import ctypes
import os
from pathlib import Path

from bpy.types import Mesh, Object

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

scene = bpy.context.scene

obj = bpy.context.active_object


class Dif:
    def __init__(self, ptr):
        self.__ptr__ = ptr

    def dispose(self):
        difbuilderlib.dispose_dif(self.__ptr__)

    def write_dif(self, path):
        difbuilderlib.write_dif(
            self.__ptr__, ctypes.create_string_buffer(path.encode("ascii"))
        )


class DifBuilder:
    def __init__(self):
        self.__ptr__ = difbuilderlib.new_difbuilder()

    def dispose(self):
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

    def build(self):
        return Dif(difbuilderlib.build(self.__ptr__))


def mesh_triangulate(me):
    import bmesh

    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()


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
    print(off)
    return off


def save(context, filepath: str = "", flip=False, double=False, maxtricount=16000):
    import bpy
    import bmesh

    obs = bpy.context.scene.objects

    builders = [DifBuilder()]

    difbuilder = builders[0]

    off = get_offset()

    tris = 0

    def save_mesh(mesh: Mesh, offset, flip=False, double=False):
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
                "NULL"
                if (len(mesh.materials) == 0)
                else mesh.materials[poly.material_index].name
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

    for ob in obs:
        ob_eval = ob

        try:
            me = ob_eval.to_mesh()
        except RuntimeError:
            continue

        me.transform(ob.matrix_world)

        save_mesh(me, off, flip, double)

    for i in range(0, len(builders)):
        dif = builders[i].build()
        dif.write_dif(str(Path(filepath).with_suffix("")) + str(i) + ".dif")
