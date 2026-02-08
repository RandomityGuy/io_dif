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


class DIFSurfaceFlags:
    """
    DIF surface flags - these are per-surface geometry flags, different from DTS material flags.
    Note: DIF does not have per-material flags like SelfIlluminating, Translucent, etc.
    Those flags are only available in DTS (shape) files.
    """
    SurfaceDetail = 1 << 0
    SurfaceAmbiguous = 1 << 1
    SurfaceOrphan = 1 << 2
    SurfaceSharedLMaps = 1 << 3
    SurfaceOutsideVisible = 1 << 4

    FLAG_NAMES = {
        1 << 0: "SurfaceDetail",
        1 << 1: "SurfaceAmbiguous",
        1 << 2: "SurfaceOrphan",
        1 << 3: "SurfaceSharedLMaps",
        1 << 4: "SurfaceOutsideVisible",
    }

    @classmethod
    def get_flag_names(cls, flags: int) -> list:
        """Return list of surface flag names that are set in the given flags value."""
        return [name for bit, name in cls.FLAG_NAMES.items() if flags & bit]


def decode_lm_texgen(surface):
    """
    Decode the lightmap texgen from the packed surface data.

    The lightMapFinalWord contains packed data:
    - bits 0-5: logScaleY (6 bits)
    - bits 6-11: logScaleX (6 bits)
    - bits 13-15: stEnc (3 bits) - which axes to use

    Returns (sc, tc, scaleX, scaleY, offsetX, offsetY) where:
    - sc, tc: axis indices (0=X, 1=Y, 2=Z)
    - scaleX, scaleY: UV scale factors
    - offsetX, offsetY: UV offsets
    """
    finalWord = surface.lightMapFinalWord
    xOffset = surface.lightMapTexGenXD
    yOffset = surface.lightMapTexGenYD

    logScaleY = (finalWord >> 0) & 0x3F  # 6 bits
    logScaleX = (finalWord >> 6) & 0x3F  # 6 bits
    stEnc = (finalWord >> 13) & 0x7      # 3 bits

    # Decode which axes to use for U and V
    if stEnc == 0: sc, tc = 0, 1  # X, Y
    elif stEnc == 1: sc, tc = 0, 2  # X, Z
    elif stEnc == 2: sc, tc = 1, 0  # Y, X
    elif stEnc == 3: sc, tc = 1, 2  # Y, Z
    elif stEnc == 4: sc, tc = 2, 0  # Z, X
    elif stEnc == 5: sc, tc = 2, 1  # Z, Y
    else: sc, tc = 0, 1  # fallback

    # Decode scales (inverse power of 2)
    invScaleX = 1 << logScaleX if logScaleX < 32 else 1
    invScaleY = 1 << logScaleY if logScaleY < 32 else 1
    scaleX = 1.0 / invScaleX
    scaleY = 1.0 / invScaleY

    return sc, tc, scaleX, scaleY, xOffset, yOffset


def compute_lm_uv(pt, sc, tc, scaleX, scaleY, offsetX, offsetY):
    """Compute lightmap UV for a vertex point."""
    coords = [pt.x, pt.y, pt.z]
    u = coords[sc] * scaleX + offsetX
    v = coords[tc] * scaleY + offsetY
    return (u, v)


def extract_lightmaps(filepath, interior, interior_index=0):
    """
    Extract lightmap textures from a DIF interior and load them into Blender.
    Returns a list of Blender image objects.
    """
    if not interior.lightMaps or len(interior.lightMaps) == 0:
        return []

    import io as io_module
    images = []
    base_name = os.path.splitext(os.path.basename(filepath))[0]

    for i, lm in enumerate(interior.lightMaps):
        if not lm.lightmap or len(lm.lightmap) == 0:
            images.append(None)
            continue

        try:
            # Create a unique name for this lightmap
            img_name = f"{base_name}_lm{interior_index}_{i}"

            # Check if image already exists
            if img_name in bpy.data.images:
                images.append(bpy.data.images[img_name])
                continue

            # The lightmap data is raw PNG bytes
            png_data = bytes(lm.lightmap)

            # Create a new Blender image and load from bytes
            # First we need to save to a temp file since Blender can't load from bytes directly
            import tempfile
            with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as tmp:
                tmp.write(png_data)
                tmp_path = tmp.name

            img = bpy.data.images.load(tmp_path)
            img.name = img_name
            img.pack()  # Pack into .blend file

            # Clean up temp file
            os.unlink(tmp_path)

            images.append(img)
            print(f"Loaded lightmap: {img_name} ({img.size[0]}x{img.size[1]})")

        except Exception as e:
            print(f"Failed to load lightmap {i}: {e}")
            images.append(None)

    return images


def create_material(filepath, matname, surface_flags=0, lightmap_info=None):
    """
    Create a Blender material for a DIF texture.

    Args:
        filepath: Path to the DIF file
        matname: Texture path from the DIF
        surface_flags: DIF surface flags bitmask
        lightmap_info: Optional dict with lightmap info:
            - lightmap_index: Index into the lightmap array
            - lightmap_image_objects: List of Blender image objects
    """
    # Store original path and extract basename for material name
    original_path = matname
    # Split on both forward and back slashes to get basename
    basename = matname.replace('\\', '/').split('/')[-1]

    mat = bpy.data.materials.new(basename)
    mat.use_nodes = True

    # Store original path as custom property for glTF export
    mat["resource_path"] = original_path

    # Build flag_names list for glTF export (matching io_scene_dtst3d format)
    flag_names = []

    # Check for IFL (animated texture) material based on file extension
    if basename.lower().endswith('.ifl'):
        flag_names.append("IflMaterial")

    # Add flag_names to material custom properties if any flags are set
    if flag_names:
        mat["flag_names"] = flag_names

    # Add surface flags if any are set (these are DIF-specific, different from DTS material flags)
    if surface_flags:
        surface_flag_names = DIFSurfaceFlags.get_flag_names(surface_flags)
        if surface_flag_names:
            mat["surface_flag_names"] = surface_flag_names

    # Add lightmap info and texture if provided
    if lightmap_info:
        lm_idx = lightmap_info.get("lightmap_index", 0)
        lm_images = lightmap_info.get("lightmap_image_objects", [])

        # Add the lightmap as a texture connected to the material.
        # We use the emissive channel with 0 strength to include the texture
        # without affecting the visual output.
        if lm_images and lm_idx < len(lm_images) and lm_images[lm_idx]:
            lm_image = lm_images[lm_idx]

            # Create lightmap texture node
            lm_texslot = mat.node_tree.nodes.new("ShaderNodeTexImage")
            lm_texslot.name = "Lightmap"
            lm_texslot.label = "Lightmap"
            lm_texslot.image = lm_image
            lm_texslot.location = (-500, -200)

            # Create a UV Map node to use the second UV layer (LightmapUV)
            uv_node = mat.node_tree.nodes.new("ShaderNodeUVMap")
            uv_node.uv_map = "LightmapUV"
            uv_node.location = (-700, -200)

            # Connect UV to lightmap texture
            mat.node_tree.links.new(lm_texslot.inputs["Vector"], uv_node.outputs["UV"])

            # Connect lightmap to emission channel
            principled = mat.node_tree.nodes.get("Principled BSDF")
            if principled:
                if bpy.app.version >= (4, 0, 0):
                    principled.inputs["Emission Strength"].default_value = 0.0
                mat.node_tree.links.new(principled.inputs["Emission Color"], lm_texslot.outputs["Color"])

    texname = resolve_texture(filepath, basename)
    if texname is not None:
        try:
            teximg = bpy.data.images.load(texname)
        except:
            teximg = None
            print("Cannot load image", texname)

        texslot = mat.node_tree.nodes.new("ShaderNodeTexImage")
        texslot.name = basename
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


def create_mesh(filepath, interior: Interior, lightmap_images=None):
    """
    :param Interior interior:
    :return:
    """
    me = bpy.data.meshes.new("Mesh")

    surfaces: list[Surface] = interior.surfaces

    # Check if we have lightmaps
    has_lightmaps = (interior.lightMaps is not None and
                     len(interior.lightMaps) > 0 and
                     interior.normalLMapIndices is not None)

    # Build a mapping from (texture_index, lightmap_index) -> material slot index
    # This ensures each unique texture+lightmap combination gets its own material
    # so that lightmap UVs match the correct lightmap texture
    material_key_to_slot = {}  # (tex_idx, lm_idx) -> slot index
    surface_to_material_slot = []  # surface index -> material slot index

    for surf_idx, surface in enumerate(surfaces):
        tex_idx = surface.textureIndex
        lm_idx = interior.normalLMapIndices[surf_idx] if has_lightmaps and surf_idx < len(interior.normalLMapIndices) else -1

        key = (tex_idx, lm_idx)
        if key not in material_key_to_slot:
            material_key_to_slot[key] = len(material_key_to_slot)
        surface_to_material_slot.append(material_key_to_slot[key])

    # Collect surface flags per material key
    material_key_flags = {}
    for surf_idx, surface in enumerate(surfaces):
        tex_idx = surface.textureIndex
        lm_idx = interior.normalLMapIndices[surf_idx] if has_lightmaps and surf_idx < len(interior.normalLMapIndices) else -1
        key = (tex_idx, lm_idx)
        if key not in material_key_flags:
            material_key_flags[key] = 0
        material_key_flags[key] |= surface.surfaceFlags

    # Create materials for each unique (texture, lightmap) combination
    # Sort by slot index to ensure correct ordering
    for key, slot_idx in sorted(material_key_to_slot.items(), key=lambda x: x[1]):
        tex_idx, lm_idx = key
        mat_name = interior.materialList[tex_idx]
        surface_flags = material_key_flags.get(key, 0)

        lightmap_info = None
        if has_lightmaps and lm_idx >= 0:
            lightmap_info = {
                "lightmap_index": lm_idx,
                "lightmap_image_objects": lightmap_images,
            }

        me.materials.append(create_material(filepath, mat_name, surface_flags, lightmap_info))

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
            # Reverse winding order so normals point outward (for correct FrontSide rendering)
            surf_indices = surf_indices[::-1]

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
            polygon.material_index = surface_to_material_slot[i]

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
        face_uvs = []
        face_lm_uvs = []  # Lightmap UVs
        face_lm_indices = []  # Which lightmap atlas each face uses
        cur_loop_idx = 0

        for (i, surface) in enumerate(surfaces):
            surf_indices = interior.windings[
                surface.windingStart : (surface.windingStart + surface.windingCount)
            ]

            surf_indices = fix_indices(surf_indices)
            # Reverse winding order so normals point outward (for correct FrontSide rendering)
            surf_indices = surf_indices[::-1]

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

            def plane_to_uv(pt, plane):
                return pt.x * plane.x + pt.y * plane.y + pt.z * plane.z + plane.d

            face_uv = []
            face_lm_uv = []
            for j, index in enumerate(surf_indices):
                pt = interior.points[index]

                uv = (
                    plane_to_uv(pt, tex_gen.planeX),
                    -plane_to_uv(pt, tex_gen.planeY),
                )
                face_uv.append(uv)

                # Compute lightmap UV if available
                if has_lightmaps:
                    sc, tc, scaleX, scaleY, offsetX, offsetY = decode_lm_texgen(surface)
                    lm_uv = compute_lm_uv(pt, sc, tc, scaleX, scaleY, offsetX, offsetY)
                    # Flip V coordinate to match Blender convention
                    face_lm_uv.append((lm_uv[0], 1.0 - lm_uv[1]))

            face_uvs.append(face_uv)
            if has_lightmaps:
                face_lm_uvs.append(face_lm_uv)
                face_lm_indices.append(interior.normalLMapIndices[i] if i < len(interior.normalLMapIndices) else 0)

        me.from_pydata(mesh_verts, [], mesh_faces)

        # Create texture UV layer
        if not me.uv_layers:
            me.uv_layers.new(name="UVMap")

        uv_layer = me.uv_layers.active.data

        for i, poly in enumerate(me.polygons):
            p: bpy.types.MeshPolygon = poly
            p.material_index = surface_to_material_slot[i]

            for j, loop_index in enumerate(p.loop_indices):
                loop = me.loops[loop_index]
                uv_layer[loop.index].uv = face_uvs[i][j]

        # Create lightmap UV layer if we have lightmaps
        if has_lightmaps and face_lm_uvs:
            lm_uv_layer = me.uv_layers.new(name="LightmapUV")
            for i, poly in enumerate(me.polygons):
                for j, loop_index in enumerate(poly.loop_indices):
                    loop = me.loops[loop_index]
                    lm_uv_layer.data[loop.index].uv = face_lm_uvs[i][j]

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
    global_matrix=None,
    import_highest_lod_only=False,
    import_lightmaps=True,
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

    # DIF files contain multiple LOD levels (detail levels)
    for idx, interior in enumerate(dif.interiors):
        lightmap_images = extract_lightmaps(filepath, interior, idx) if import_lightmaps else None
        new_objects.append(create_mesh(filepath, interior, lightmap_images))
        if import_highest_lod_only:
            break

    pathedInteriors: list[Object] = []
    for idx, pathedInterior in enumerate(dif.subObjects):
        lightmap_images = extract_lightmaps(filepath, pathedInterior, 1 + idx) if import_lightmaps else None
        pathedInteriors.append(create_mesh(filepath, pathedInterior, lightmap_images))

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

        if len(waypoints) > 0:
            first_type = waypoints[0].smoothingType
            if first_type == 0:
                itr.dif_props.marker_type = "linear"
            elif first_type == 1:
                itr.dif_props.marker_type = "spline"
            elif first_type == 2:
                itr.dif_props.marker_type = "accelerate"
        else:
            itr.dif_props.marker_type = "linear"

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
