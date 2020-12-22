import array
import os
import time
import bpy
from bpy.types import Curve, Object
import mathutils
from .hxDif import *
from bpy_extras.io_utils import unpack_list
from bpy_extras.image_utils import load_image
from .util import default_materials, resolve_texture, get_rgb_colors

from bpy_extras.wm_utils.progress_report import ProgressReport, ProgressReportSubstep


def create_material(filepath, matname):
    mat = bpy.data.materials.new(matname)
    mat.use_nodes = True

    texname = resolve_texture(filepath, matname)
    if texname is not None:
        try:
            teximg = bpy.data.images.load(texname)
        except:
            teximg = None
            print("Cannot load image", texname)

        texslot = mat.node_tree.nodes.new("ShaderNodeTexImage")
        texslot.name = matname
        texslot.image = teximg
        mat.node_tree.nodes["Principled BSDF"].inputs["Specular"].default_value = 0
        mat.node_tree.links.new(
            mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"],
            texslot.outputs["Color"],
        )

    return mat


def create_mesh(filepath, interior: Interior):
    """
    :param Interior interior:
    :return:
    """
    me = bpy.data.meshes.new("Mesh")

    faces = []

    normals = []
    tex_coords = []

    indices = []

    for mat in interior.materialList:
        me.materials.append(create_material(filepath, mat))

    for surface in interior.surfaces:
        for i in range(0, surface.windingCount - 2):
            if i % 2 == 0:
                index0 = interior.windings[i + surface.windingStart + 2]
                index1 = interior.windings[i + surface.windingStart + 1]
                index2 = interior.windings[i + surface.windingStart]
            else:
                index0 = interior.windings[i + surface.windingStart]
                index1 = interior.windings[i + surface.windingStart + 1]
                index2 = interior.windings[i + surface.windingStart + 2]

            plane_flipped = (surface.planeIndex & 0x8000) == 0x8000
            print(surface.planeIndex)
            print(surface.planeIndex & ~0x8000)
            normal_index = interior.planes[surface.planeIndex & ~0x8000].normalIndex
            tex_gen = interior.texGenEQs[surface.texGenIndex]

            normal = interior.normals[normal_index]
            if plane_flipped:
                normal.x *= -1
                normal.y *= -1
                normal.z *= -1

            pt0 = interior.points[index0]
            pt1 = interior.points[index1]
            pt2 = interior.points[index2]

            def plane_to_uv(pt, plane):
                return pt.x * plane.x + pt.y * plane.y + pt.z * plane.z + plane.d

            coord0 = [
                plane_to_uv(pt0, tex_gen.planeX),
                plane_to_uv(pt0, tex_gen.planeY),
            ]
            coord1 = [
                plane_to_uv(pt1, tex_gen.planeX),
                plane_to_uv(pt1, tex_gen.planeY),
            ]
            coord2 = [
                plane_to_uv(pt2, tex_gen.planeX),
                plane_to_uv(pt2, tex_gen.planeY),
            ]

            indices.append((index0, len(normals), len(tex_coords)))
            normals.append(normal)
            tex_coords.append(coord0)

            indices.append((index1, len(normals), len(tex_coords)))
            normals.append(normal)
            tex_coords.append(coord1)

            indices.append((index2, len(normals), len(tex_coords)))
            normals.append(normal)
            tex_coords.append(coord2)

            faces.append(
                (
                    (len(indices) - 3, len(indices) - 2, len(indices) - 1),
                    surface.textureIndex,
                )
            )

    me.vertices.add(len(interior.points))
    for i in range(0, len(interior.points)):
        me.vertices[i].co = [
            interior.points[i].x,
            interior.points[i].y,
            interior.points[i].z,
        ]
        me.vertices[i].normal = [normals[i].x, normals[i].y, normals[i].z]

    me.polygons.add(len(faces))
    me.loops.add(len(faces) * 3)

    me.uv_layers.new()
    uvs = me.uv_layers[0]

    for i, ((verts, material), poly) in enumerate(zip(faces, me.polygons)):
        poly.use_smooth = True
        poly.loop_total = 3
        poly.loop_start = i * 3

        poly.material_index = material

        for j, index in zip(poly.loop_indices, verts):
            me.loops[j].vertex_index = indices[index][0]
            uvs.data[j].uv = (
                tex_coords[indices[index][1]][0],
                tex_coords[indices[index][1]][1],
            )

    me.validate()
    me.update()

    ob = bpy.data.objects.new("Object", me)
    ob.empty_display_type = "SINGLE_ARROW"
    ob.empty_display_size = 0.5

    return ob


def load(
    context: bpy.types.Context,
    filepath,
    *,
    global_clamp_size=0.0,
    use_smooth_groups=True,
    use_edges=True,
    use_split_objects=True,
    use_split_groups=True,
    use_image_search=True,
    use_groups_as_vgroups=False,
    use_cycles=True,
    relpath=None,
    global_matrix=None
):
    """
    Called by the user interface or another script.
    load_obj(path) - should give acceptable results.
    This function passes the file and sends the data off
        to be split into objects and then converted into mesh objects
    """

    dif = Dif.Load(str(filepath))

    if global_matrix is None:
        global_matrix = mathutils.Matrix()

    # deselect all
    if bpy.ops.object.select_all.poll():
        bpy.ops.object.select_all(action="DESELECT")

    scene = context.scene
    new_objects = []  # put new objects here

    for interior in dif.interiors:
        new_objects.append(create_mesh(filepath, interior))

    pathedInteriors: list[Object] = []
    for pathedInterior in dif.subObjects:
        pathedInteriors.append(create_mesh(filepath, pathedInterior))

    # Create new obj
    for obj in new_objects:
        base = scene.collection.objects.link(obj)

        # we could apply this anywhere before scaling.
        obj.matrix_world = global_matrix

    for mover in dif.interiorPathfollowers:
        pos = mover.offset
        itr = pathedInteriors[mover.interiorResIndex]
        itr: Object = itr.copy()
        base = scene.collection.objects.link(itr)
        itr.location = [pos.x, pos.y, pos.z]
        itr.dif_props.interior_type = "pathed_interior"

        waypoints: list[WayPoint] = mover.wayPoint

        markerpts = [
            (waypt.position.x, waypt.position.y, waypt.position.z)
            for waypt in waypoints
        ]

        curve = bpy.data.curves.new("markers", type="CURVE")
        curve.dimensions = "3D"
        spline = curve.splines.new(type="NURBS")
        spline.points.add(len(markerpts) - 1)

        for p, new_co in zip(spline.points, markerpts):
            p.co = new_co + (1.0,)

        path = bpy.data.objects.new("path", curve)
        scene.collection.objects.link(path)

        itr.dif_props.marker_path = curve

    for ge in dif.gameEntities:
        g: GameEntity = ge
        gobj = bpy.data.objects.new(g.datablock, None)
        gobj.location = (g.position.x, g.position.y, g.position.z)
        gobj.dif_props.interior_type = "game_entity"
        gobj.dif_props.game_entity_datablock = g.datablock
        gobj.dif_props.game_entity_gameclass = g.gameClass
        scene.collection.objects.link(gobj)

    context.view_layer.update()

    axis_min = [1000000000] * 3
    axis_max = [-1000000000] * 3

    if global_clamp_size:
        # Get all object bounds
        for ob in new_objects:
            for v in ob.bound_box:
                for axis, value in enumerate(v):
                    if axis_min[axis] > value:
                        axis_min[axis] = value
                    if axis_max[axis] < value:
                        axis_max[axis] = value

        # Scale objects
        max_axis = max(
            axis_max[0] - axis_min[0],
            axis_max[1] - axis_min[1],
            axis_max[2] - axis_min[2],
        )
        scale = 1.0

        while global_clamp_size < max_axis * scale:
            scale = scale / 10.0

        for obj in new_objects:
            obj.scale = scale, scale, scale

    # progress.leave_substeps("Done.")

    return {"FINISHED"}
