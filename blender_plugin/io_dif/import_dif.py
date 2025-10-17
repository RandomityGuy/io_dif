import array
import os
import time
import bpy
from bpy.props import CollectionProperty
from bpy.types import Curve, Object
import mathutils
from .hxDif import *
from bpy_extras.io_utils import unpack_list
from bpy_extras.image_utils import load_image
from .util import default_materials, resolve_texture, get_rgb_colors

from bpy_extras.wm_utils.progress_report import ProgressReport, ProgressReportSubstep


def create_material(filepath, matname):
    if "/" in matname:
        matname = matname.split("/")[1]
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
        if bpy.app.version < (4, 0, 0):
            mat.node_tree.nodes["Principled BSDF"].inputs["Specular"].default_value = 0
        else:
            mat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
        mat.node_tree.links.new(
            mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"],
            texslot.outputs["Color"],
        )

    return mat


def fix_indices(indices: list[int]):
    new_indices = [0] * len(indices)
    for i in range(len(indices)):
        if i >= 2:
            if i % 2 == 0:
                new_indices[len(indices) - 1 - (i - 2) // 2] = indices[i]
            else:
                new_indices[(i + 1) // 2] = indices[i]
        else:
            new_indices[i] = indices[i]
    return new_indices


def create_mesh(filepath, interior: Interior):
    """
    :param Interior interior:
    :return:
    """
    me = bpy.data.meshes.new("Mesh")

    for mat in interior.materialList:
        me.materials.append(create_material(filepath, mat))

    surfaces: list[Surface] = interior.surfaces

    if bpy.app.version < (4, 0, 0):
        me.vertices.add(len(interior.points))
        for i in range(0, len(interior.points)):
            me.vertices[i].co = [
                interior.points[i].x,
                interior.points[i].y,
                interior.points[i].z,
            ]

        me.polygons.add(len(surfaces))

        loop_count = 0
        for surf in surfaces:
            loop_count += surf.windingCount

        me.loops.add(loop_count)

        surface_uvs = {}
        cur_loop_idx = 0

        for (i, surface) in enumerate(surfaces):
            surf_indices = interior.windings[
                surface.windingStart : (surface.windingStart + surface.windingCount)
            ]

            surf_indices = fix_indices(surf_indices)

            plane_flipped = surface.planeFlipped
            normal_index = interior.planes[surface.planeIndex & ~0x8000].normalIndex
            tex_gen = interior.texGenEQs[surface.texGenIndex]

            normal = interior.normals[normal_index]
            if plane_flipped:
                normal.x *= -1
                normal.y *= -1
                normal.z *= -1

            polygon = me.polygons[i]
            polygon.loop_start = cur_loop_idx
            polygon.loop_total = len(surf_indices)
            cur_loop_idx += polygon.loop_total
            polygon.material_index = surface.textureIndex

            def plane_to_uv(pt, plane):
                return pt.x * plane.x + pt.y * plane.y + pt.z * plane.z + plane.d

            for j, index in enumerate(surf_indices):
                me.loops[j + polygon.loop_start].vertex_index = index
                me.loops[j + polygon.loop_start].normal = (normal.x, normal.y, normal.z)

                pt = interior.points[index]

                uv = (
                    plane_to_uv(pt, tex_gen.planeX),
                    -plane_to_uv(pt, tex_gen.planeY),
                )
                surface_uvs[j + polygon.loop_start] = uv

        me.uv_layers.new()
        uvs = me.uv_layers[0]

        for loop_idx in surface_uvs:
            uvs.data[loop_idx].uv = surface_uvs[loop_idx]
    else:
        mesh_verts = []
        for i in range(0, len(interior.points)):
            mesh_verts.append((interior.points[i].x, interior.points[i].y, interior.points[i].z))

        mesh_faces = []
        face_texs = []
        face_uvs = []
        cur_loop_idx = 0

        for (i, surface) in enumerate(surfaces):
            surf_indices = interior.windings[
                surface.windingStart : (surface.windingStart + surface.windingCount)
            ]

            surf_indices = fix_indices(surf_indices)

            plane_flipped = surface.planeFlipped
            normal_index = interior.planes[surface.planeIndex & ~0x8000].normalIndex
            tex_gen = interior.texGenEQs[surface.texGenIndex]

            normal = interior.normals[normal_index]
            if plane_flipped:
                normal.x *= -1
                normal.y *= -1
                normal.z *= -1

            polygon = surf_indices
            cur_loop_idx += len(surf_indices)
            mesh_faces.append(polygon)

            face_texs.append(surface.textureIndex)

            def plane_to_uv(pt, plane):
                return pt.x * plane.x + pt.y * plane.y + pt.z * plane.z + plane.d

            face_uv = []
            for j, index in enumerate(surf_indices):
                pt = interior.points[index]

                uv = (
                    plane_to_uv(pt, tex_gen.planeX),
                    -plane_to_uv(pt, tex_gen.planeY),
                )
                face_uv.append(uv)
            face_uvs.append(face_uv)

        me.from_pydata(mesh_verts, [], mesh_faces)

        if not me.uv_layers:
            me.uv_layers.new()

        uv_layer = me.uv_layers.active.data

        for i, poly in enumerate(me.polygons):
            p: bpy.types.MeshPolygon = poly
            p.material_index = face_texs[i]
            
            for j, loop_index in enumerate(p.loop_indices):
                loop = me.loops[loop_index]
                uv_layer[loop.index].uv = face_uvs[i][j]

    me.validate(verbose=True)
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
        itr.location = [-pos.x, -pos.y, -pos.z]
        itr.dif_props.interior_type = "pathed_interior"
        itr.dif_props.start_time = int(mover.properties.h.get("initialPosition", 0))
        itr.dif_props.reverse = mover.properties.h.get("initialTargetPosition", 0) == "-2"
        itr.dif_props.constant_speed = False

        waypoints: list[WayPoint] = mover.wayPoint

        markerpts = [
            (waypt.position.x, waypt.position.y, waypt.position.z)
            for waypt in waypoints
        ]

        curve = bpy.data.curves.new("markers", type="CURVE")
        curve.dimensions = "3D"
        spline = curve.splines.new(type="POLY")
        spline.points.add(len(markerpts) - 1)

        for p, new_co in zip(spline.points, markerpts):
            p.co = new_co + (1.0,)

        path = bpy.data.objects.new("path", curve)
        scene.collection.objects.link(path)

        itr.dif_props.marker_path = curve

        total_time = 0
        for pt in waypoints:
            total_time += pt.msToNext
        itr.dif_props.total_time = total_time

        first_type = waypoints[0].smoothingType
        if first_type == 0:
            itr.dif_props.marker_type = "linear"
        elif first_type == 1:
            itr.dif_props.marker_type = "spline"
        elif first_type == 2:
            itr.dif_props.marker_type = "accelerate"

        for trigger_id in mover.triggerId:
            trigger = dif.triggers[trigger_id]
            tobj = bpy.data.objects.new(trigger.datablock, None)
            tobj.dif_props.interior_type = "path_trigger"
            tobj.dif_props.pathed_interior_target = itr
            tobj.dif_props.game_entity_datablock = trigger.datablock
            tobj.dif_props.target_marker = False
            for key in trigger.properties.h:
                prop = tobj.dif_props.game_entity_properties.add()
                prop.key = key
                prop.value = trigger.properties.get(key)

            t_min = mathutils.Vector((float('inf'), float('inf'), float('inf')))
            t_max = mathutils.Vector((-float('inf'), -float('inf'), -float('inf')))
            for p in trigger.polyhedron.pointList:
                t_min.x = min(t_min.x, p.x)
                t_min.y = min(t_min.y, p.y)
                t_min.z = min(t_min.z, p.z)

                t_max.x = max(t_max.x, p.x)
                t_max.y = max(t_max.y, p.y)
                t_max.z = max(t_max.z, p.z)

            tobj.location = t_min
            tobj.scale = mathutils.Vector((t_max.x - t_min.x, t_max.y - t_min.y, t_max.z - t_min.z))
            tobj.location.y += tobj.scale.y
            tobj.location += mathutils.Vector((trigger.offset.x, trigger.offset.y, trigger.offset.z))
            scene.collection.objects.link(tobj)

    if dif.gameEntities != None:
        for ge in dif.gameEntities:
            g: GameEntity = ge
            gobj = bpy.data.objects.new(g.datablock, None)
            gobj.location = (g.position.x, g.position.y, g.position.z)
            gobj.dif_props.interior_type = "game_entity"
            gobj.dif_props.game_entity_datablock = g.datablock
            gobj.dif_props.game_entity_gameclass = g.gameClass
            for key in g.properties.h:
                prop = gobj.dif_props.game_entity_properties.add()
                prop.key = key
                prop.value = g.properties.get(key)
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
