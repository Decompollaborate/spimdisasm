#!/usr/bin/python3

from __future__ import annotations


versions = {
    "ntsc_0.9" : "NNR",
    "ntsc_1.0" : "NN0",
    "ntsc_1.1" : "NN1",
    "pal_1.0" : "NP0",
    "ntsc_1.2" : "NN2",
    "pal_1.1" : "NP1",
    "jp_gc" : "GJO",
    "jp_mq" : "GJM",
    "usa_gc" : "GUO",
    "usa_mq" : "GUM",
    "pal_gc" : "GPO",
    "pal_gc_dbg1" : "GPOD1",
    "pal_gc_dbg2" : "GPOD2",
    "pal_mq" : "GPM",
    "pal_mq_dbg" : "GPMD",
    "jp_gc_ce" : "GJC",
    "ique_cn" : "IC",
    "ique_tw" : "IT",

    "mm_jp_1.0" : "NJ0",
    "mm_jp_1.1" : "NJ1",
    "mm_usa_demo" : "NUK",
    "mm_usa" : "NU0",
    "mm_pal_1.0" : "NE0",
    "mm_pal_dbg" : "NED",
    "mm_pal_1.1" : "NE1",
    "mm_usa_gc" : "GU",
    "mm_pal_gc" : "GE",
    "mm_jp_gc" : "GJ",
}

def getVersionAbbr(filename: str) -> str:
    for ver in versions:
        if "baserom_" + ver + "/" in filename:
            return versions[ver]
    # If the version wasn't found.
    return filename


ENTRYPOINT = 0x80000400

ACTOR_ID_MAX    = 0x01D7
ACTOR_ID_MAX_MM = 0x2B2

# The offset of the overlay table in file `code`.
offset_ActorOverlayTable = {
    "ntsc_0.9" : 0x0D7280,
    "ntsc_1.0" : 0x0D7490,
    "ntsc_1.1" : 0x0D7650,
    "pal_1.0" : 0xD4D80,
    "ntsc_1.2" : 0x0D7490,
    "pal_1.1" : 0x0D4DE0,
    "jp_gc" : 0x0D6B60,
    "jp_mq" : 0x0D6B40,
    "usa_gc" : 0x0D6B40,
    "usa_mq" : 0x0D6B20,
    "pal_gc" : 0x0D44A0,
    "pal_gc_dbg1" : 0x0F9460,
    "pal_gc_dbg2" : 0x0F9460,
    "pal_mq" : 0x0D4480,
    "pal_mq_dbg" : 0x0F9440,
    "jp_gc_ce" : 0x0D6B40,
    "ique_cn" : 0x0D7180,
    "ique_tw" : 0x0D6AA0,

    "mm_jp_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0x109510,
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}

bootVramStart = {
    "ntsc_0.9" : 0x80000460,
    "ntsc_1.0" : 0x80000460,
    "ntsc_1.1" : 0x80000460,
    "pal_1.0" : 0x80000460,
    "ntsc_1.2" : 0x80000460,
    "pal_1.1" : 0x80000460,
    "jp_gc" : 0x80000460,
    "jp_mq" : 0x80000460,
    "usa_gc" : 0x80000460,
    "usa_mq" : 0x80000460,
    "pal_gc" : 0x80000460,
    "pal_gc_dbg1" : 0x80000460,
    "pal_gc_dbg2" : 0x80000460,
    "pal_mq" : 0x80000460,
    "pal_mq_dbg" : 0x80000460,
    "jp_gc_ce" : 0x80000460,
    "ique_cn" : 0x80000450, # iQue likes to be special
    "ique_tw" : 0x80000450,

    "mm_jp_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}

bootDataStart = {
    "ntsc_0.9" : -1,
    "ntsc_1.0" : -1,
    "ntsc_1.1" : -1,
    "pal_1.0" : 0x62B0,
    "ntsc_1.2" : 0x6310,
    "pal_1.1" : 0x62B0,
    "jp_gc" : 0x5C70,
    "jp_mq" : 0x5C70,
    "usa_gc" : 0x5C70,
    "usa_mq" : 0x5C70,
    "pal_gc" : 0x5C70,
    "pal_gc_dbg1" : 0x8FD0,
    "pal_gc_dbg2" : 0x8FD0,
    "pal_mq" : 0x5C70,
    "pal_mq_dbg" : 0x8FD0,
    "jp_gc_ce" : 0x5C70,
    "ique_cn" : 0x98F0,
    "ique_tw" : 0x9380,

    "mm_jp_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}

bootRodataStart = {
    "ntsc_0.9" : 0x60F0,
    "ntsc_1.0" : 0x60F0,
    "ntsc_1.1" : 0x60F0,
    "pal_1.0" : 0x6610,
    "ntsc_1.2" : 0x6620,
    "pal_1.1" : 0x6610,
    "jp_gc" : 0x5F40,
    "jp_mq" : 0x5F40,
    "usa_gc" : 0x5F40,
    "usa_mq" : 0x5F40,
    "pal_gc" : 0x5F40,
    "pal_gc_dbg1" : 0xAB60,
    "pal_gc_dbg2" : 0xAB60,
    "pal_mq" : 0x5F40,
    "pal_mq_dbg" : 0xAB60,
    "jp_gc_ce" : 0x5F40,
    "ique_cn" : 0x9D40,
    "ique_tw" : 0x97F0,

    "mm_jp_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}

codeVramStart = {
    "ntsc_0.9" : 0x800110A0,
    "ntsc_1.0" : 0x800110A0,
    "ntsc_1.1" : 0x800110A0,
    "pal_1.0" : 0x800116E0,
    "ntsc_1.2" : 0x800116E0,
    "pal_1.1" : 0x800116E0,
    "jp_gc" : 0x80010EE0,
    "jp_mq" : 0x80010EE0,
    "usa_gc" : 0x80010EE0,
    "usa_mq" : 0x80010EE0,
    "pal_gc" : 0x80010F00,
    "pal_gc_dbg1" : 0x8001CE60,
    "pal_gc_dbg2" : 0x8001CE60,
    "pal_mq" : 0x80010F00,
    "pal_mq_dbg" : 0x8001CE60,
    "jp_gc_ce" : 0x80010EE0,
    "ique_cn" : 0x80018FA0,
    "ique_tw" : 0x80018A40,

    "mm_jp_1.0" : 0x800A76A0,
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0x800A5AC0,
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}

codeDataStart = {
    "ntsc_0.9" : 0x0D6400,
    "ntsc_1.0" : 0x0D6610,
    "ntsc_1.1" : 0x0D67D0,
    "pal_1.0" : 0x0D3F20,
    "ntsc_1.2" : 0x0D6610,
    "pal_1.1" : 0x0D3F60,
    "jp_gc" : 0x0D5CE0,
    "jp_mq" : 0x0D5CC0,
    "usa_gc" : 0x0D5CC0,
    "usa_mq" : 0x0D5CA0,
    "pal_gc" : 0x0D3620,
    "pal_gc_dbg1" : 0x0F85E0,
    "pal_gc_dbg2" : 0x0F85E0,
    "pal_mq" : 0x0D3600,
    "pal_mq_dbg" : 0x0F85C0,
    "jp_gc_ce" : 0x0D5CC0,
    "ique_cn" : 0x0D6330,
    "ique_tw" : 0x0D5C10,

    "mm_jp_1.0" : 0xFE2F0,
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0x104FF0,
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}

codeRodataStart = {
    "ntsc_0.9" : 0x0F4A50,
    "ntsc_1.0" : 0x0F4C60,
    "ntsc_1.1" : 0x0F4E20,
    "pal_1.0" : 0x0F2570,
    "ntsc_1.2" : 0x0F4C60,
    "pal_1.1" : 0x0F25B0,
    "jp_gc" : 0x0F3E90,
    "jp_mq" : 0x0F3E70,
    "usa_gc" : 0x0F3E70,
    "usa_mq" : 0x0F3E50,
    "pal_gc" : 0x0F17D0,
    "pal_gc_dbg1" : 0x117EF0,
    "pal_gc_dbg2" : 0x117EF0,
    "pal_mq" : 0x0F17B0,
    "pal_mq_dbg" : 0x117ED0,
    "jp_gc_ce" : 0x0F3E70,
    "ique_cn" : 0x0F44C0,
    "ique_tw" : 0x0F3DE0,

    "mm_jp_1.0" : 0x12EE80,
    "mm_jp_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_demo" : 0xFFFFFF, # TODO: FIX
    "mm_usa" : 0x136330,
    "mm_pal_1.0" : 0xFFFFFF, # TODO: FIX
    "mm_pal_dbg" : 0xFFFFFF, # TODO: FIX
    "mm_pal_1.1" : 0xFFFFFF, # TODO: FIX
    "mm_usa_gc" : 0xFFFFFF, # TODO: FIX
    "mm_pal_gc" : 0xFFFFFF, # TODO: FIX
    "mm_jp_gc" : 0xFFFFFF, # TODO: FIX
}
