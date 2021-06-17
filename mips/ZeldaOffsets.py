#!/usr/bin/python3

from __future__ import annotations


versions = {
    "ntsc_1.0_rc" : "NNR",
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
    "pal_gc_dbg" : "GPOD",
    "pal_gc_dbg2" : "GPOD2",
    "pal_mq" : "GPM",
    "pal_mq_dbg" : "GPMD",
    "jp_gc_ce" : "GJC",
    "cn_ique" : "IC",
    "tw_ique" : "IT",
}

def getVersionAbbr(filename: str) -> str:
    for ver in versions:
        if "baserom_" + ver + "/" in filename:
            return versions[ver]
    # If the version wasn't found.
    return filename


# in JAL format. # Real address would be (address << 2)
address_Graph_OpenDisps = {
    "ntsc_1.0_rc" : 0x001F856,
    "ntsc_1.0" : 0x001F8A6,
    "ntsc_1.1" : 0x001F8A6,
    "pal_1.0" : 0x001FA2A,
    "ntsc_1.2" : 0x001FA4A,
    "pal_1.1" : 0x001FA2A,
    "jp_gc" : 0x001F792,
    "jp_mq" : 0x001F792,
    "usa_gc" : 0x001F78A,
    "usa_mq" : 0x001F78A,
    "pal_gc" : 0x001F77E,
    "pal_gc_dbg" : 0x0,
    "pal_gc_dbg2" : 0x0,
    "pal_mq" : 0x001F77E,
    "pal_mq_dbg" : 0x0031AB1,
    "jp_gc_ce" : 0x001F78A,
    "cn_ique" : 0x0,
    "tw_ique" : 0x0,
}

ENTRYPOINT = 0x80000400

ACTOR_ID_MAX = 0x01D7

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
    "pal_gc_dbg" : 0x0F9460,
    "pal_gc_dbg2" : 0x0F9460,
    "pal_mq" : 0x0D4480,
    "pal_mq_dbg" : 0x0F9440,
    "jp_gc_ce" : 0x0D6B40,
    "ique_cn" : 0x0D7180,
    "ique_tw" : 0x0D6AA0,
}