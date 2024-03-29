#pragma once
#include "DIFBuilder/DIFBuilder.hpp"

#if _MSC_VER
#define PLUGIN_API __declspec(dllexport)
#else
#define PLUGIN_API __attribute__((visibility("default")))
#endif

extern "C"
{
	PLUGIN_API DIF::DIFBuilder *new_difbuilder();

	PLUGIN_API void dispose_difbuilder(DIF::DIFBuilder *difbuilder);

	PLUGIN_API void dispose_dif(DIF::DIF *dif);

	PLUGIN_API void add_triangle(DIF::DIFBuilder *difbuilder, float *p1, float *p2, float *p3, float *uv1, float *uv2, float *uv3, float *n, char *material);

	PLUGIN_API DIF::DIF *build(DIF::DIFBuilder *difbuilder);

	PLUGIN_API void add_pathed_interior(DIF::DIFBuilder *difbuilder, DIF::DIF *difptr, std::vector<DIF::DIFBuilder::Marker> *markerlist);

	PLUGIN_API void write_dif(DIF::DIF *dif, char *path);

	PLUGIN_API std::vector<DIF::DIFBuilder::Marker> *new_marker_list();

	PLUGIN_API void dispose_marker_list(std::vector<DIF::DIFBuilder::Marker> *markerlist);

	PLUGIN_API void push_marker(std::vector<DIF::DIFBuilder::Marker> *markerlist, float *pos, int msToNext, int initialTargetPosition);

	PLUGIN_API void add_game_entity(DIF::DIF *dif, char *gameClass, char *datablock, float *pos, DIF::Dictionary *dict);

	PLUGIN_API void add_trigger(DIF::DIFBuilder *difbuilder, float *position, char *name, char *datablock, DIF::Dictionary *props);

	PLUGIN_API DIF::Dictionary *new_dict();

	PLUGIN_API void dispose_dict(DIF::Dictionary *dict);

	PLUGIN_API void add_dict_kvp(DIF::Dictionary *dict, char *key, char *value);
}