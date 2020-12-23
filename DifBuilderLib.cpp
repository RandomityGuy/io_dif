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

	void add_pathed_interior(DIF::DIFBuilder *builder, DIF::DIF *dif, std::vector<DIF::DIFBuilder::Marker> *markerlist)
	{
		builder->addPathedInterior(dif->interior[0], *markerlist);
	}

	void add_trigger(DIF::DIFBuilder *difbuilder, float *position, char *name, char *datablock, DIF::Dictionary *props)
	{
		DIF::DIFBuilder::Trigger trigger;
		trigger.name = std::string(name);
		trigger.datablock = std::string(datablock);
		trigger.properties = DIF::Dictionary(*props);
		trigger.position = glm::vec3(position[0], position[1], position[2]);
		difbuilder->addTrigger(trigger);
	}

	void write_dif(DIF::DIF *dif, char *path)
	{
		std::ofstream outStr;
		outStr.open(path, std::ios::out | std::ios::binary);
		DIF::Version ver;
		ver.dif.type = DIF::Version::DIFVersion::MBG;
		dif->write(outStr, ver);
	}

	std::vector<DIF::DIFBuilder::Marker> *new_marker_list()
	{
		return new std::vector<DIF::DIFBuilder::Marker>();
	}

	void dispose_marker_list(std::vector<DIF::DIFBuilder::Marker> *markerlist)
	{
		delete markerlist;
	}

	void push_marker(std::vector<DIF::DIFBuilder::Marker> *markerlist, float *pos, int msToNext, int initialPathPosition)
	{
		DIF::DIFBuilder::Marker m;
		m.position = glm::vec3(pos[0], pos[1], pos[2]);
		m.msToNext = msToNext;
		m.smoothing = 0;
		m.initialPathPosition = initialPathPosition;
		markerlist->push_back(m);
	}

	void add_game_entity(DIF::DIF *dif, char *gameClass, char *datablock, float *pos, DIF::Dictionary *dict)
	{
		DIF::GameEntity g;
		g.datablock = std::string(datablock);
		g.gameClass = std::string(gameClass);
		g.position = glm::vec3(pos[0], pos[1], pos[2]);
		g.properties = DIF::Dictionary(*dict);
		g.properties.push_back(std::pair<std::string, std::string>(std::string("static"), std::string("1")));
		g.properties.push_back(std::pair<std::string, std::string>(std::string("rotate"), std::string("1")));
		dif->readGameEntities = 2;
		dif->gameEntity.push_back(g);
	}

	DIF::Dictionary *new_dict()
	{
		return new DIF::Dictionary();
	}

	void dispose_dict(DIF::Dictionary *dict)
	{
		delete dict;
	}

	void add_dict_kvp(DIF::Dictionary *dict, char *key, char *value)
	{
		dict->push_back(std::pair<std::string, std::string>(std::string(key), std::string(value)));
	}
}
