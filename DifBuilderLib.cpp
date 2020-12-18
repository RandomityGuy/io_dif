// dllmain.cpp : Defines the entry point for the DLL application.
#include "DifBuilderLib.h"
#include <DIFBuilder/DIFBuilder.hpp>

extern "C"
{
	DIF::DIFBuilder *new_difbuilder()
	{
		return new DIF::DIFBuilder();
	}

	void dispose_difbuilder(DIF::DIFBuilder *difbuilder)
	{
		if (difbuilder != NULL)
			delete difbuilder;
	}

	void dispose_dif(DIF::DIF *dif)
	{
		if (dif != NULL)
			delete dif;
	}

	void add_triangle(DIF::DIFBuilder *builder, float *p1, float *p2, float *p3, float *uv1, float *uv2, float *uv3, float *n, char *material)
	{
		DIF::DIFBuilder::Triangle tri = DIF::DIFBuilder::Triangle();
		tri.points[0].vertex = glm::vec3(p1[0], p1[1], p1[2]);
		tri.points[1].vertex = glm::vec3(p2[0], p2[1], p2[2]);
		tri.points[2].vertex = glm::vec3(p3[0], p3[1], p3[2]);

		tri.points[0].uv = glm::vec2(uv1[0], uv1[1]);
		tri.points[1].uv = glm::vec2(uv2[0], uv2[1]);
		tri.points[2].uv = glm::vec2(uv3[0], uv3[1]);

		tri.points[0].normal = glm::vec3(n[0], n[1], n[2]);
		tri.points[1].normal = tri.points[0].normal;
		tri.points[2].normal = tri.points[0].normal;

		builder->addTriangle(tri, std::string(material));
	}

	DIF::DIF *build(DIF::DIFBuilder *builder)
	{
		DIF::DIF dif;
		builder->build(dif);
		return new DIF::DIF(dif);
	}

	void add_pathed_interior(DIF::DIFBuilder *builder, DIF::DIF *dif)
	{
		builder->addPathedInterior(dif->interior[0], std::vector<DIF::Marker>());
	}

	void write_dif(DIF::DIF *dif, char *path)
	{
		std::ofstream outStr;
		outStr.open(path, std::ios::out | std::ios::binary);
		dif->write(outStr, DIF::Version());
	}
}
