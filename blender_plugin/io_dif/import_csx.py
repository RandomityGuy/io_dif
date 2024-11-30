import math
from typing import Tuple
import bpy
from bpy.props import CollectionProperty
from bpy.types import Curve, Object
import mathutils
from bpy_extras.io_utils import unpack_list
from bpy_extras.image_utils import load_image

from .util import default_materials, resolve_texture, get_rgb_colors

from bpy_extras.wm_utils.progress_report import ProgressReport, ProgressReportSubstep

import xml.etree.ElementTree as ET


class CSXEntity:
    def __init__(
        self,
        id: str,
        className: str,
        origin: list[float],
        properties: dict,
    ):
        self.id = id
        self.className = className
        self.origin = origin
        self.properties = properties


class CSXTexGen:
    def __init__(
        self,
        planeX: list[float],
        planeY: list[float],
        texRot: float,
        texScale: list[float],
    ):
        self.texRot = texRot
        self.texScale = texScale
        self.texPlaneX = planeX
        self.texPlaneY = planeY

    def compute_uv(self, vertex: list[float], texsizes: list[float]):
        if self.texScale[0] * self.texScale[1] == 0:
            return [0, 0]
        axisU, axisV = self.transform_axes()
        target = self.project_raw(vertex, axisU, axisV)
        target[0] *= (1 / self.texScale[0]) * (32 / texsizes[0])
        target[1] *= (1 / -self.texScale[1]) * (32 / texsizes[1])

        shift = [self.texPlaneX[3] / texsizes[0], -self.texPlaneY[3] / texsizes[1]]

        # rotate shift
        # if self.texRot % 360 != 0:
        #     shift[0], shift[1] = (
        #         shift[0] * math.cos(math.radians(self.texRot))
        #         - shift[1] * math.sin(math.radians(self.texRot)),
        #         shift[0] * math.sin(math.radians(self.texRot))
        #         + shift[1] * math.cos(math.radians(self.texRot)),
        #     )

        target[0] += shift[0]
        target[1] += shift[1]
        return target

    def project_raw(self, vertex: list[float], axisU: list[float], axisV: list[float]):
        return [
            vertex[0] * axisU[0] + vertex[1] * axisU[1] + vertex[2] * axisU[2],
            vertex[0] * axisV[0] + vertex[1] * axisV[1] + vertex[2] * axisV[2],
        ]

    def transform_axes(self):
        axisU = mathutils.Vector(
            (self.texPlaneX[0], self.texPlaneX[1], self.texPlaneX[2])
        )
        axisV = mathutils.Vector(
            (self.texPlaneY[0], self.texPlaneY[1], self.texPlaneY[2])
        )
        if self.texRot % 360 == 0:
            return axisU, axisV
        upDir = axisU.cross(axisV)
        rotMat = mathutils.Matrix.Rotation(math.radians(self.texRot), 3, upDir)
        axisU.rotate(rotMat)
        axisV.rotate(rotMat)
        return axisU, axisV


class CSXBrushFace:
    def __init__(
        self,
        id: str,
        plane: list[float],
        material: str,
        texgen: CSXTexGen,
        indices: list[int],
        texSize: list[int],
    ):
        self.id = id
        self.plane = plane
        self.material = material
        self.texgen = texgen
        self.indices = indices
        self.texSize = texSize


class CSXBrush:
    def __init__(
        self,
        id: str,
        owner: str,
        type: int,
        pos: list[float],
        rot: list[float],
        transform: list[float],
        vertices: list[list[float]],
        faces: list[CSXBrushFace],
    ):
        self.id = id
        self.owner = owner
        self.type = type
        self.pos = pos
        self.rot = rot
        self.transform = transform
        self.vertices = vertices
        self.faces = faces


class CSXDetail:
    def __init__(self, brushes: list[CSXBrush], entities: list[CSXEntity]):
        self.brushes = brushes
        self.entities = entities


class CSX:
    def __init__(self, details: list[CSXDetail]):
        self.details = details


def parse_texgen(texgen: str):
    texgen = texgen.split(" ")
    planeX = [float(x) for x in texgen[0:4]]
    planeY = [float(x) for x in texgen[4:8]]
    texRot = float(texgen[8])
    texScale = [float(x) for x in texgen[9:]]
    return CSXTexGen(planeX, planeY, texRot, texScale)


def parse_csx(path):
    csx: ET.ElementTree = ET.parse(path)
    csscene = csx.getroot()
    detailsxml = csscene.find("DetailLevels")

    details = []
    for detail in detailsxml.iter("DetailLevel"):
        detailbrushes = []
        for brush in detail.find("InteriorMap").find("Brushes").iter("Brush"):
            brushverts = []
            for vert in brush.find("Vertices").iter("Vertex"):
                vertdata = [float(x) for x in vert.get("pos").split(" ")]
                brushverts.append(vertdata)
            brushfaces = []
            for face in brush.iter("Face"):
                brushfaces.append(
                    CSXBrushFace(
                        face.get("id"),
                        [float(x) for x in face.get("plane").split(" ")],
                        face.get("material"),
                        parse_texgen(face.get("texgens")),
                        [
                            int(x)
                            for x in face.find("Indices")
                            .get("indices")
                            .strip()
                            .split(" ")
                        ],
                        [int(x) for x in face.get("texDiv").split(" ")],
                    )
                )
            detailbrushes.append(
                CSXBrush(
                    brush.get("id"),
                    brush.get("owner"),
                    int(brush.get("type")),
                    [float(x) for x in brush.get("pos").split(" ")],
                    [float(x) for x in brush.get("rot").split(" ")],
                    [float(x) for x in brush.get("transform").split(" ")],
                    brushverts,
                    brushfaces,
                )
            )

        detailentities = []
        for entity in detail.find("InteriorMap").find("Entities").iter("Entity"):
            if entity.get("isPointEntity") == "0":
                continue
            entityprops = entity.find("Properties").attrib
            detailentities.append(
                CSXEntity(
                    entity.get("id"),
                    entity.get("classname"),
                    [float(x) for x in entity.get("origin").split(" ")],
                    entityprops,
                )
            )

        details.append(CSXDetail(detailbrushes, detailentities))

    cscene = CSX(details)
    return cscene


def create_material(filepath, matname):
    if "/" in matname:
        matname = matname.split("/")[1]
    prevmat = bpy.data.materials.find(matname)
    if prevmat != -1:
        return bpy.data.materials.get(matname)
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


# texsize: scale: factor
# 16     : 4    :  1/4
# 16     : 3    :  1
# 16     : 2    :  4
# 16     : 1    :  16


def create_mesh(filepath, brush: CSXBrush):
    """
    :param Interior interior:
    :return:
    """
    me = bpy.data.meshes.new("Mesh")

    materials = list(set(x.material for x in brush.faces))

    for mat in materials:
        me.materials.append(create_material(filepath, mat))

    if bpy.app.version < (4, 0, 0):
        me.vertices.add(len(brush.vertices))
        for i in range(0, len(brush.vertices)):
            me.vertices[i].co = brush.vertices[i]

        me.polygons.add(len(brush.faces))
        tot_loops = 0
        for face in brush.faces:
            tot_loops += len(face.indices)

        me.loops.add(tot_loops)

        surface_uvs = {}
        cur_loop_idx = 0

        for (i, face) in enumerate(brush.faces):
            tex_gen = face.texgen

            normal = face.plane[:3]

            polygon = me.polygons[i]
            polygon.loop_start = cur_loop_idx
            polygon.loop_total = len(face.indices)
            cur_loop_idx += polygon.loop_total
            polygon.material_index = materials.index(face.material)

            for j, index in enumerate(face.indices):
                me.loops[j + polygon.loop_start].vertex_index = index
                me.loops[j + polygon.loop_start].normal = normal

                pt = brush.vertices[index]

                uv = tex_gen.compute_uv(pt, face.texSize)
                surface_uvs[j + polygon.loop_start] = uv

        me.uv_layers.new()
        uvs = me.uv_layers[0]

        for loop_idx in surface_uvs:
            uvs.data[loop_idx].uv = surface_uvs[loop_idx]
    else:
        verts = []
        faces = []
        face_texs = []
        face_uvs = []
        for vert in brush.vertices:
            verts.append(vert)

        for face in brush.faces:
            face_verts = []
            for index in face.indices:
                face_verts.append(index)
            faces.append(face_verts)
            face_texs.append(face.material)
            uvs = []
            for i in range(0, len(face.indices)):
                uvs.append(face.texgen.compute_uv(verts[face.indices[i]], face.texSize))
            face_uvs.append(uvs)

        me.from_pydata(verts, [], faces)

        if not me.uv_layers:
            me.uv_layers.new()

        uv_layer = me.uv_layers.active.data

        for i, poly in enumerate(me.polygons):
            p: bpy.types.MeshPolygon = poly
            p.material_index = materials.index(face_texs[i])
            
            for j, loop_index in enumerate(p.loop_indices):
                loop = me.loops[loop_index]
                uv_layer[loop.index].uv = face_uvs[i][j]

    me.validate()
    me.update()

    transformmat = mathutils.Matrix(
        [[brush.transform[4 * i + j] for j in range(0, 4)] for i in range(0, 4)]
    )

    _, rotq, scale = transformmat.decompose()
    rot: mathutils.Quaternion = rotq

    newmat = mathutils.Matrix(
        ((scale.x, 0, 0, 0), (0, scale.y, 0, 0), (0, 0, scale.z, 0), (0, 0, 0, 1))
    )
    tmp = mathutils.Matrix.Rotation(
        rot.angle, 4, mathutils.Vector((rot.axis.x, rot.axis.y, rot.axis.z))
    )

    newmat = newmat @ tmp

    newmat[0][3] = transformmat[0][3]
    newmat[1][3] = transformmat[1][3]
    newmat[2][3] = transformmat[2][3]

    ob = bpy.data.objects.new("Object", me)
    ob.empty_display_type = "SINGLE_ARROW"
    ob.empty_display_size = 0.5
    # ob.matrix_world = [
    #     [brush.transform[4 * i + j] for j in range(0, 4)] for i in range(0, 4)
    # ]
    ob.matrix_world = newmat
    # ob.rotation_axis_angle = (rot.axis.x, rot.axis.y, rot.axis.z, rot.angle)
    # ob.rotation_mode = "AXIS_ANGLE"
    # ob.scale = (scale.x, scale.y, scale.z)
    # ob.location = [brush.transform[3], brush.transform[7], brush.transform[11]]

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

    csscene = parse_csx(str(filepath))

    if global_matrix is None:
        global_matrix = mathutils.Matrix()

    # deselect all
    if bpy.ops.object.select_all.poll():
        bpy.ops.object.select_all(action="DESELECT")

    scene = context.scene
    new_objects: list[Object] = []  # put new objects here

    for detail in csscene.details:
        for brush in detail.brushes:
            new_objects.append(create_mesh(filepath, brush))

        for ge in detail.entities:
            g: CSXEntity = ge
            gobj = bpy.data.objects.new(g.className, None)
            gobj.location = g.origin
            gobj.dif_props.interior_type = "game_entity"
            gobj.dif_props.game_entity_datablock = g.className
            gobj.dif_props.game_entity_gameclass = g.properties["game_class"]
            for key in g.properties:
                prop = gobj.dif_props.game_entity_properties.add()
                prop.key = key
                prop.value = g.properties.get(key)
            scene.collection.objects.link(gobj)

    # Create new obj
    for obj in new_objects:
        base = scene.collection.objects.link(obj)

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
