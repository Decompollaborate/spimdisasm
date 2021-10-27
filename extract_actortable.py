#!/usr/bin/python3

from __future__ import annotations

import argparse
import os
import sys
import struct
from multiprocessing import Pool, cpu_count, Manager
from typing import Dict, List
import zlib


ROM_FILE_NAME = 'baserom.z64'
ROM_FILE_NAME_V = 'baserom_{}.z64'
CODE_FILE_NAME = 'baserom/code'
CODE_FILE_NAME_V = 'baserom_{}/code'

FILE_TABLE_OFFSET = {
    "NTSC 0.9":     0x07430, # a.k.a. NTSC 1.0 RC
    "NTSC 1.0":     0x07430,
    "NTSC 1.1":     0x07430,
    "PAL 1.0":      0x07950,
    "NTSC 1.2":     0x07960,
    "PAL 1.1":      0x07950,
    "JP GC":        0x07170,
    "JP MQ":        0x07170,
    "USA GC":       0x07170,
    "USA MQ":       0x07170,
    "PAL GC DBG1":  0x12F70,
    "PAL GC DBG2":  0x12F70,
    "PAL MQ DBG":   0x12F70,
    "PAL GC":       0x07170,
    "PAL MQ":       0x07170,
    "JP GC CE":     0x07170, # Zelda collection
    "IQUE CN":      0x0B7A0,
    "IQUE TW":      0x0B240,
    "GATEWAY":      0x0AC80,
}
FILE_TABLE_OFFSET["NTSC J 0.9"]   = FILE_TABLE_OFFSET["NTSC 0.9"]
FILE_TABLE_OFFSET["NTSC J 1.0"]   = FILE_TABLE_OFFSET["NTSC 1.0"]
FILE_TABLE_OFFSET["NTSC J 1.1"]   = FILE_TABLE_OFFSET["NTSC 1.1"]
FILE_TABLE_OFFSET["NTSC J 1.2"]   = FILE_TABLE_OFFSET["NTSC 1.2"]
FILE_TABLE_OFFSET["PAL WII 1.1"]  = FILE_TABLE_OFFSET["PAL 1.1"]

FILE_NAMES: Dict[str, List[str] | None] = {
    "NTSC 0.9":     None, 
    "NTSC 1.0":     None,
    "NTSC 1.1":     None,
    "PAL 1.0":      None,
    "NTSC 1.2":     None,
    "PAL 1.1":      None,
    "JP GC":        None,
    "JP MQ":        None,
    "USA GC":       None,
    "USA MQ":       None,
    "PAL GC DBG1":   None,
    "PAL GC DBG2":  None,
    "PAL MQ DBG":   None,
    "PAL GC":       None,
    "PAL MQ":       None,
    "JP GC CE":     None, # Zelda collector's edition
    "IQUE CN":      None,
    "IQUE TW":      None,
     "GATEWAY":      None,
}
FILE_NAMES["NTSC J 0.9"]  = FILE_NAMES["NTSC 0.9"]
FILE_NAMES["NTSC J 1.0"]  = FILE_NAMES["NTSC 1.0"]
FILE_NAMES["NTSC J 1.1"]  = FILE_NAMES["NTSC 1.1"]
FILE_NAMES["NTSC J 1.2"]  = FILE_NAMES["NTSC 1.2"]
FILE_NAMES["PAL WII 1.1"] = FILE_NAMES["PAL 1.1"]


ACTOR_ID_MAX = 0x01D7

# The offset of the overlay table in file `code`.
offset_ActorOverlayTable = {
    "NTSC 0.9":     0x0D7280,
    "NTSC 1.0":     0x0D7490,
    "NTSC 1.1":     0x0D7650,
    "PAL 1.0":      0xD4D80,
    "NTSC 1.2":     0x0D7490,
    "PAL 1.1":      0x0D4DE0,
    "JP GC":        0x0D6B60,
    "JP MQ":        0x0D6B40,
    "USA GC":       0x0D6B40,
    "USA MQ":       0x0D6B20,
    "PAL GC DBG1":  0x0D44A0,
    "PAL GC DBG2":  0x0F9460,
    "PAL MQ DBG":   0x0F9460,
    "PAL GC":       0x0D4480,
    "PAL MQ":       0x0F9440,
    "JP GC CE":     0x0D6B40,
    "IQUE CN":      0x0D7180,
    "IQUE TW":      0x0D6AA0,
}

actorNames = [
"ACTOR_PLAYER",
"ACTOR_UNSET_1",
"ACTOR_EN_TEST",
"ACTOR_UNSET_3",
"ACTOR_EN_GIRLA",
"ACTOR_UNSET_5",
"ACTOR_UNSET_6",
"ACTOR_EN_PART",
"ACTOR_EN_LIGHT",
"ACTOR_EN_DOOR",
"ACTOR_EN_BOX",
"ACTOR_BG_DY_YOSEIZO",
"ACTOR_BG_HIDAN_FIREWALL",
"ACTOR_EN_POH",
"ACTOR_EN_OKUTA",
"ACTOR_BG_YDAN_SP",
"ACTOR_EN_BOM",
"ACTOR_EN_WALLMAS",
"ACTOR_EN_DODONGO",
"ACTOR_EN_FIREFLY",
"ACTOR_EN_HORSE",
"ACTOR_EN_ITEM00",
"ACTOR_EN_ARROW",
"ACTOR_UNSET_17",
"ACTOR_EN_ELF",
"ACTOR_EN_NIW",
"ACTOR_UNSET_1A",
"ACTOR_EN_TITE",
"ACTOR_EN_REEBA",
"ACTOR_EN_PEEHAT",
"ACTOR_EN_BUTTE",
"ACTOR_UNSET_1F",
"ACTOR_EN_INSECT",
"ACTOR_EN_FISH",
"ACTOR_UNSET_22",
"ACTOR_EN_HOLL",
"ACTOR_EN_SCENE_CHANGE",
"ACTOR_EN_ZF",
"ACTOR_EN_HATA",
"ACTOR_BOSS_DODONGO",
"ACTOR_BOSS_GOMA",
"ACTOR_EN_ZL1",
"ACTOR_EN_VIEWER",
"ACTOR_EN_GOMA",
"ACTOR_BG_PUSHBOX",
"ACTOR_EN_BUBBLE",
"ACTOR_DOOR_SHUTTER",
"ACTOR_EN_DODOJR",
"ACTOR_EN_BDFIRE",
"ACTOR_UNSET_31",
"ACTOR_EN_BOOM",
"ACTOR_EN_TORCH2",
"ACTOR_EN_BILI",
"ACTOR_EN_TP",
"ACTOR_UNSET_36",
"ACTOR_EN_ST",
"ACTOR_EN_BW",
"ACTOR_EN_A_OBJ",
"ACTOR_EN_EIYER",
"ACTOR_EN_RIVER_SOUND",
"ACTOR_EN_HORSE_NORMAL",
"ACTOR_EN_OSSAN",
"ACTOR_BG_TREEMOUTH",
"ACTOR_BG_DODOAGO",
"ACTOR_BG_HIDAN_DALM",
"ACTOR_BG_HIDAN_HROCK",
"ACTOR_EN_HORSE_GANON",
"ACTOR_BG_HIDAN_ROCK",
"ACTOR_BG_HIDAN_RSEKIZOU",
"ACTOR_BG_HIDAN_SEKIZOU",
"ACTOR_BG_HIDAN_SIMA",
"ACTOR_BG_HIDAN_SYOKU",
"ACTOR_EN_XC",
"ACTOR_BG_HIDAN_CURTAIN",
"ACTOR_BG_SPOT00_HANEBASI",
"ACTOR_EN_MB",
"ACTOR_EN_BOMBF",
"ACTOR_EN_ZL2",
"ACTOR_BG_HIDAN_FSLIFT",
"ACTOR_EN_OE2",
"ACTOR_BG_YDAN_HASI",
"ACTOR_BG_YDAN_MARUTA",
"ACTOR_BOSS_GANONDROF",
"ACTOR_UNSET_53",
"ACTOR_EN_AM",
"ACTOR_EN_DEKUBABA",
"ACTOR_EN_M_FIRE1",
"ACTOR_EN_M_THUNDER",
"ACTOR_BG_DDAN_JD",
"ACTOR_BG_BREAKWALL",
"ACTOR_EN_JJ",
"ACTOR_EN_HORSE_ZELDA",
"ACTOR_BG_DDAN_KD",
"ACTOR_DOOR_WARP1",
"ACTOR_OBJ_SYOKUDAI",
"ACTOR_ITEM_B_HEART",
"ACTOR_EN_DEKUNUTS",
"ACTOR_BG_MENKURI_KAITEN",
"ACTOR_BG_MENKURI_EYE",
"ACTOR_EN_VALI",
"ACTOR_BG_MIZU_MOVEBG",
"ACTOR_BG_MIZU_WATER",
"ACTOR_ARMS_HOOK",
"ACTOR_EN_FHG",
"ACTOR_BG_MORI_HINERI",
"ACTOR_EN_BB",
"ACTOR_BG_TOKI_HIKARI",
"ACTOR_EN_YUKABYUN",
"ACTOR_BG_TOKI_SWD",
"ACTOR_EN_FHG_FIRE",
"ACTOR_BG_MJIN",
"ACTOR_BG_HIDAN_KOUSI",
"ACTOR_DOOR_TOKI",
"ACTOR_BG_HIDAN_HAMSTEP",
"ACTOR_EN_BIRD",
"ACTOR_UNSET_73",
"ACTOR_UNSET_74",
"ACTOR_UNSET_75",
"ACTOR_UNSET_76",
"ACTOR_EN_WOOD02",
"ACTOR_UNSET_78",
"ACTOR_UNSET_79",
"ACTOR_UNSET_7A",
"ACTOR_UNSET_7B",
"ACTOR_EN_LIGHTBOX",
"ACTOR_EN_PU_BOX",
"ACTOR_UNSET_7E",
"ACTOR_UNSET_7F",
"ACTOR_EN_TRAP",
"ACTOR_EN_AROW_TRAP",
"ACTOR_EN_VASE",
"ACTOR_UNSET_83",
"ACTOR_EN_TA",
"ACTOR_EN_TK",
"ACTOR_BG_MORI_BIGST",
"ACTOR_BG_MORI_ELEVATOR",
"ACTOR_BG_MORI_KAITENKABE",
"ACTOR_BG_MORI_RAKKATENJO",
"ACTOR_EN_VM",
"ACTOR_DEMO_EFFECT",
"ACTOR_DEMO_KANKYO",
"ACTOR_BG_HIDAN_FWBIG",
"ACTOR_EN_FLOORMAS",
"ACTOR_EN_HEISHI1",
"ACTOR_EN_RD",
"ACTOR_EN_PO_SISTERS",
"ACTOR_BG_HEAVY_BLOCK",
"ACTOR_BG_PO_EVENT",
"ACTOR_OBJ_MURE",
"ACTOR_EN_SW",
"ACTOR_BOSS_FD",
"ACTOR_OBJECT_KANKYO",
"ACTOR_EN_DU",
"ACTOR_EN_FD",
"ACTOR_EN_HORSE_LINK_CHILD",
"ACTOR_DOOR_ANA",
"ACTOR_BG_SPOT02_OBJECTS",
"ACTOR_BG_HAKA",
"ACTOR_MAGIC_WIND",
"ACTOR_MAGIC_FIRE",
"ACTOR_UNSET_A0",
"ACTOR_EN_RU1",
"ACTOR_BOSS_FD2",
"ACTOR_EN_FD_FIRE",
"ACTOR_EN_DH",
"ACTOR_EN_DHA",
"ACTOR_EN_RL",
"ACTOR_EN_ENCOUNT1",
"ACTOR_DEMO_DU",
"ACTOR_DEMO_IM",
"ACTOR_DEMO_TRE_LGT",
"ACTOR_EN_FW",
"ACTOR_BG_VB_SIMA",
"ACTOR_EN_VB_BALL",
"ACTOR_BG_HAKA_MEGANE",
"ACTOR_BG_HAKA_MEGANEBG",
"ACTOR_BG_HAKA_SHIP",
"ACTOR_BG_HAKA_SGAMI",
"ACTOR_UNSET_B2",
"ACTOR_EN_HEISHI2",
"ACTOR_EN_ENCOUNT2",
"ACTOR_EN_FIRE_ROCK",
"ACTOR_EN_BROB",
"ACTOR_MIR_RAY",
"ACTOR_BG_SPOT09_OBJ",
"ACTOR_BG_SPOT18_OBJ",
"ACTOR_BOSS_VA",
"ACTOR_BG_HAKA_TUBO",
"ACTOR_BG_HAKA_TRAP",
"ACTOR_BG_HAKA_HUTA",
"ACTOR_BG_HAKA_ZOU",
"ACTOR_BG_SPOT17_FUNEN",
"ACTOR_EN_SYATEKI_ITM",
"ACTOR_EN_SYATEKI_MAN",
"ACTOR_EN_TANA",
"ACTOR_EN_NB",
"ACTOR_BOSS_MO",
"ACTOR_EN_SB",
"ACTOR_EN_BIGOKUTA",
"ACTOR_EN_KAREBABA",
"ACTOR_BG_BDAN_OBJECTS",
"ACTOR_DEMO_SA",
"ACTOR_DEMO_GO",
"ACTOR_EN_IN",
"ACTOR_EN_TR",
"ACTOR_BG_SPOT16_BOMBSTONE",
"ACTOR_UNSET_CE",
"ACTOR_BG_HIDAN_KOWARERUKABE",
"ACTOR_BG_BOMBWALL",
"ACTOR_BG_SPOT08_ICEBLOCK",
"ACTOR_EN_RU2",
"ACTOR_OBJ_DEKUJR",
"ACTOR_BG_MIZU_UZU",
"ACTOR_BG_SPOT06_OBJECTS",
"ACTOR_BG_ICE_OBJECTS",
"ACTOR_BG_HAKA_WATER",
"ACTOR_UNSET_D8",
"ACTOR_EN_MA2",
"ACTOR_EN_BOM_CHU",
"ACTOR_EN_HORSE_GAME_CHECK",
"ACTOR_BOSS_TW",
"ACTOR_EN_RR",
"ACTOR_EN_BA",
"ACTOR_EN_BX",
"ACTOR_EN_ANUBICE",
"ACTOR_EN_ANUBICE_FIRE",
"ACTOR_BG_MORI_HASHIGO",
"ACTOR_BG_MORI_HASHIRA4",
"ACTOR_BG_MORI_IDOMIZU",
"ACTOR_BG_SPOT16_DOUGHNUT",
"ACTOR_BG_BDAN_SWITCH",
"ACTOR_EN_MA1",
"ACTOR_BOSS_GANON",
"ACTOR_BOSS_SST",
"ACTOR_UNSET_EA",
"ACTOR_UNSET_EB",
"ACTOR_EN_NY",
"ACTOR_EN_FR",
"ACTOR_ITEM_SHIELD",
"ACTOR_BG_ICE_SHELTER",
"ACTOR_EN_ICE_HONO",
"ACTOR_ITEM_OCARINA",
"ACTOR_UNSET_F2",
"ACTOR_UNSET_F3",
"ACTOR_MAGIC_DARK",
"ACTOR_DEMO_6K",
"ACTOR_EN_ANUBICE_TAG",
"ACTOR_BG_HAKA_GATE",
"ACTOR_BG_SPOT15_SAKU",
"ACTOR_BG_JYA_GOROIWA",
"ACTOR_BG_JYA_ZURERUKABE",
"ACTOR_UNSET_FB",
"ACTOR_BG_JYA_COBRA",
"ACTOR_BG_JYA_KANAAMI",
"ACTOR_FISHING",
"ACTOR_OBJ_OSHIHIKI",
"ACTOR_BG_GATE_SHUTTER",
"ACTOR_EFF_DUST",
"ACTOR_BG_SPOT01_FUSYA",
"ACTOR_BG_SPOT01_IDOHASHIRA",
"ACTOR_BG_SPOT01_IDOMIZU",
"ACTOR_BG_PO_SYOKUDAI",
"ACTOR_BG_GANON_OTYUKA",
"ACTOR_BG_SPOT15_RRBOX",
"ACTOR_BG_UMAJUMP",
"ACTOR_UNSET_109",
"ACTOR_ARROW_FIRE",
"ACTOR_ARROW_ICE",
"ACTOR_ARROW_LIGHT",
"ACTOR_UNSET_10D",
"ACTOR_UNSET_10E",
"ACTOR_ITEM_ETCETERA",
"ACTOR_OBJ_KIBAKO",
"ACTOR_OBJ_TSUBO",
"ACTOR_EN_WONDER_ITEM",
"ACTOR_EN_IK",
"ACTOR_DEMO_IK",
"ACTOR_EN_SKJ",
"ACTOR_EN_SKJNEEDLE",
"ACTOR_EN_G_SWITCH",
"ACTOR_DEMO_EXT",
"ACTOR_DEMO_SHD",
"ACTOR_EN_DNS",
"ACTOR_ELF_MSG",
"ACTOR_EN_HONOTRAP",
"ACTOR_EN_TUBO_TRAP",
"ACTOR_OBJ_ICE_POLY",
"ACTOR_BG_SPOT03_TAKI",
"ACTOR_BG_SPOT07_TAKI",
"ACTOR_EN_FZ",
"ACTOR_EN_PO_RELAY",
"ACTOR_BG_RELAY_OBJECTS",
"ACTOR_EN_DIVING_GAME",
"ACTOR_EN_KUSA",
"ACTOR_OBJ_BEAN",
"ACTOR_OBJ_BOMBIWA",
"ACTOR_UNSET_128",
"ACTOR_UNSET_129",
"ACTOR_OBJ_SWITCH",
"ACTOR_OBJ_ELEVATOR",
"ACTOR_OBJ_LIFT",
"ACTOR_OBJ_HSBLOCK",
"ACTOR_EN_OKARINA_TAG",
"ACTOR_EN_YABUSAME_MARK",
"ACTOR_EN_GOROIWA",
"ACTOR_EN_EX_RUPPY",
"ACTOR_EN_TORYO",
"ACTOR_EN_DAIKU",
"ACTOR_UNSET_134",
"ACTOR_EN_NWC",
"ACTOR_EN_BLKOBJ",
"ACTOR_ITEM_INBOX",
"ACTOR_EN_GE1",
"ACTOR_OBJ_BLOCKSTOP",
"ACTOR_EN_SDA",
"ACTOR_EN_CLEAR_TAG",
"ACTOR_EN_NIW_LADY",
"ACTOR_EN_GM",
"ACTOR_EN_MS",
"ACTOR_EN_HS",
"ACTOR_BG_INGATE",
"ACTOR_EN_KANBAN",
"ACTOR_EN_HEISHI3",
"ACTOR_EN_SYATEKI_NIW",
"ACTOR_EN_ATTACK_NIW",
"ACTOR_BG_SPOT01_IDOSOKO",
"ACTOR_EN_SA",
"ACTOR_EN_WONDER_TALK",
"ACTOR_BG_GJYO_BRIDGE",
"ACTOR_EN_DS",
"ACTOR_EN_MK",
"ACTOR_EN_BOM_BOWL_MAN",
"ACTOR_EN_BOM_BOWL_PIT",
"ACTOR_EN_OWL",
"ACTOR_EN_ISHI",
"ACTOR_OBJ_HANA",
"ACTOR_OBJ_LIGHTSWITCH",
"ACTOR_OBJ_MURE2",
"ACTOR_EN_GO",
"ACTOR_EN_FU",
"ACTOR_UNSET_154",
"ACTOR_EN_CHANGER",
"ACTOR_BG_JYA_MEGAMI",
"ACTOR_BG_JYA_LIFT",
"ACTOR_BG_JYA_BIGMIRROR",
"ACTOR_BG_JYA_BOMBCHUIWA",
"ACTOR_BG_JYA_AMISHUTTER",
"ACTOR_BG_JYA_BOMBIWA",
"ACTOR_BG_SPOT18_BASKET",
"ACTOR_UNSET_15D",
"ACTOR_EN_GANON_ORGAN",
"ACTOR_EN_SIOFUKI",
"ACTOR_EN_STREAM",
"ACTOR_UNSET_161",
"ACTOR_EN_MM",
"ACTOR_EN_KO",
"ACTOR_EN_KZ",
"ACTOR_EN_WEATHER_TAG",
"ACTOR_BG_SST_FLOOR",
"ACTOR_EN_ANI",
"ACTOR_EN_EX_ITEM",
"ACTOR_BG_JYA_IRONOBJ",
"ACTOR_EN_JS",
"ACTOR_EN_JSJUTAN",
"ACTOR_EN_CS",
"ACTOR_EN_MD",
"ACTOR_EN_HY",
"ACTOR_EN_GANON_MANT",
"ACTOR_EN_OKARINA_EFFECT",
"ACTOR_EN_MAG",
"ACTOR_DOOR_GERUDO",
"ACTOR_ELF_MSG2",
"ACTOR_DEMO_GT",
"ACTOR_EN_PO_FIELD",
"ACTOR_EFC_ERUPC",
"ACTOR_BG_ZG",
"ACTOR_EN_HEISHI4",
"ACTOR_EN_ZL3",
"ACTOR_BOSS_GANON2",
"ACTOR_EN_KAKASI",
"ACTOR_EN_TAKARA_MAN",
"ACTOR_OBJ_MAKEOSHIHIKI",
"ACTOR_OCEFF_SPOT",
"ACTOR_END_TITLE",
"ACTOR_UNSET_180",
"ACTOR_EN_TORCH",
"ACTOR_DEMO_EC",
"ACTOR_SHOT_SUN",
"ACTOR_EN_DY_EXTRA",
"ACTOR_EN_WONDER_TALK2",
"ACTOR_EN_GE2",
"ACTOR_OBJ_ROOMTIMER",
"ACTOR_EN_SSH",
"ACTOR_EN_STH",
"ACTOR_OCEFF_WIPE",
"ACTOR_OCEFF_STORM",
"ACTOR_EN_WEIYER",
"ACTOR_BG_SPOT05_SOKO",
"ACTOR_BG_JYA_1FLIFT",
"ACTOR_BG_JYA_HAHENIRON",
"ACTOR_BG_SPOT12_GATE",
"ACTOR_BG_SPOT12_SAKU",
"ACTOR_EN_HINTNUTS",
"ACTOR_EN_NUTSBALL",
"ACTOR_BG_SPOT00_BREAK",
"ACTOR_EN_SHOPNUTS",
"ACTOR_EN_IT",
"ACTOR_EN_GELDB",
"ACTOR_OCEFF_WIPE2",
"ACTOR_OCEFF_WIPE3",
"ACTOR_EN_NIW_GIRL",
"ACTOR_EN_DOG",
"ACTOR_EN_SI",
"ACTOR_BG_SPOT01_OBJECTS2",
"ACTOR_OBJ_COMB",
"ACTOR_BG_SPOT11_BAKUDANKABE",
"ACTOR_OBJ_KIBAKO2",
"ACTOR_EN_DNT_DEMO",
"ACTOR_EN_DNT_JIJI",
"ACTOR_EN_DNT_NOMAL",
"ACTOR_EN_GUEST",
"ACTOR_BG_BOM_GUARD",
"ACTOR_EN_HS2",
"ACTOR_DEMO_KEKKAI",
"ACTOR_BG_SPOT08_BAKUDANKABE",
"ACTOR_BG_SPOT17_BAKUDANKABE",
"ACTOR_UNSET_1AA",
"ACTOR_OBJ_MURE3",
"ACTOR_EN_TG",
"ACTOR_EN_MU",
"ACTOR_EN_GO2",
"ACTOR_EN_WF",
"ACTOR_EN_SKB",
"ACTOR_DEMO_GJ",
"ACTOR_DEMO_GEFF",
"ACTOR_BG_GND_FIREMEIRO",
"ACTOR_BG_GND_DARKMEIRO",
"ACTOR_BG_GND_SOULMEIRO",
"ACTOR_BG_GND_NISEKABE",
"ACTOR_BG_GND_ICEBLOCK",
"ACTOR_EN_GB",
"ACTOR_EN_GS",
"ACTOR_BG_MIZU_BWALL",
"ACTOR_BG_MIZU_SHUTTER",
"ACTOR_EN_DAIKU_KAKARIKO",
"ACTOR_BG_BOWL_WALL",
"ACTOR_EN_WALL_TUBO",
"ACTOR_EN_PO_DESERT",
"ACTOR_EN_CROW",
"ACTOR_DOOR_KILLER",
"ACTOR_BG_SPOT11_OASIS",
"ACTOR_BG_SPOT18_FUTA",
"ACTOR_BG_SPOT18_SHUTTER",
"ACTOR_EN_MA3",
"ACTOR_EN_COW",
"ACTOR_BG_ICE_TURARA",
"ACTOR_BG_ICE_SHUTTER",
"ACTOR_EN_KAKASI2",
"ACTOR_EN_KAKASI3",
"ACTOR_OCEFF_WIPE4",
"ACTOR_EN_EG",
"ACTOR_BG_MENKURI_NISEKABE",
"ACTOR_EN_ZO",
"ACTOR_OBJ_MAKEKINSUTA",
"ACTOR_EN_GE3",
"ACTOR_OBJ_TIMEBLOCK",
"ACTOR_OBJ_HAMISHI",
"ACTOR_EN_ZL4",
"ACTOR_EN_MM2",
"ACTOR_BG_JYA_BLOCK",
"ACTOR_OBJ_WARP2BLOCK",
]

romData = None
Edition = "" # "pal_mq"
Version = "" # "PAL MQ"


def readFile(filepath):
    with open(filepath) as f:
        return [x.strip() for x in f.readlines()]

def readFilelists():
    FILE_NAMES["PAL MQ DBG"] = readFile("filelists/filelist_pal_mq_dbg.txt")
    FILE_NAMES["PAL MQ"] = readFile("filelists/filelist_pal_mq.txt")
    FILE_NAMES["USA MQ"] = readFile("filelists/filelist_usa_mq.txt")
    FILE_NAMES["NTSC 1.0"] = readFile("filelists/filelist_ntsc_1.0.txt")
    FILE_NAMES["PAL 1.0"] = readFile("filelists/filelist_pal_1.0.txt")
    FILE_NAMES["JP GC CE"] = readFile("filelists/filelist_jp_gc_ce.txt")
    FILE_NAMES["IQUE CN"] = readFile("filelists/filelist_ique_cn.txt")

    FILE_NAMES["JP MQ"] = FILE_NAMES["USA MQ"]

    FILE_NAMES["USA GC"] = FILE_NAMES["JP GC CE"]
    FILE_NAMES["JP GC"] = FILE_NAMES["USA GC"]
    FILE_NAMES["PAL GC"] = FILE_NAMES["PAL MQ"]

    FILE_NAMES["PAL 1.1"] = FILE_NAMES["PAL 1.0"]

    FILE_NAMES["PAL GC DBG1"] = FILE_NAMES["PAL MQ DBG"]
    FILE_NAMES["PAL GC DBG2"] = FILE_NAMES["PAL MQ DBG"]

    FILE_NAMES["IQUE TW"] = FILE_NAMES["IQUE CN"]

    FILE_NAMES["NTSC 0.9"] = FILE_NAMES["NTSC 1.0"]
    FILE_NAMES["NTSC 1.1"] = FILE_NAMES["NTSC 1.0"]
    FILE_NAMES["NTSC 1.2"] = FILE_NAMES["NTSC 1.0"]

    FILE_NAMES["NTSC J 0.9"]  = FILE_NAMES["NTSC 0.9"]
    FILE_NAMES["NTSC J 1.0"]  = FILE_NAMES["NTSC 1.0"]
    FILE_NAMES["NTSC J 1.1"]  = FILE_NAMES["NTSC 1.1"]
    FILE_NAMES["NTSC J 1.2"]  = FILE_NAMES["NTSC 1.2"]
    FILE_NAMES["PAL WII 1.1"] = FILE_NAMES["PAL 1.1"]
    
    FILE_NAMES["GATEWAY"] = FILE_NAMES["IQUE CN"]

def initialize_worker(rom_data):#, dmaTable):
    global romData
    global globalDmaTable
    romData = rom_data
    # globalDmaTable = dmaTable

def read_uint32_be(offset):
    return struct.unpack('>I', romData[offset:offset+4])[0]

def read_uint16_be(offset):
    return struct.unpack('>H', romData[offset:offset+2])[0]

def read_uint8_be(offset):
    return struct.unpack('>B', romData[offset:offset+1])[0]


# def ExtractFunc(i):
#     versionName = FILE_NAMES[Version][i]
#     if versionName == "":
#         print(f"Skipping {i} because it doesn't have a name.")
#         return
#     filename = f'baserom_{Edition}/' + versionName
#     entryOffset = FILE_TABLE_OFFSET[Version] + 16 * i

#     virtStart = read_uint32_be(entryOffset + 0)
#     virtEnd   = read_uint32_be(entryOffset + 4)
#     physStart = read_uint32_be(entryOffset + 8)
#     physEnd   = read_uint32_be(entryOffset + 12)

#     displayedPhysEnd = physEnd
    
#     if physEnd == 0:  # uncompressed
#         compressed = False
#         size = virtEnd - virtStart
#         compressString = ""
#         actualPhysEnd = physStart + size
#         if showEnd:
#             displayedPhysEnd = actualPhysEnd
#     else:             # compressed
#         compressed = True
#         size = physEnd - physStart
#         compressString = "compressed"

#     print(f"{versionName},{virtStart:X},{virtEnd:X},{physStart:X},{displayedPhysEnd:X},{compressString}")

def ExtractFunc(i):
    # versionName = FILE_NAMES[Version][i]
    # if versionName == "":
    #     print(f"Skipping {i} because it doesn't have a name.")
    #     return
    # filename = f'baserom_{Edition}/' + versionName
    entryOffset = offset_ActorOverlayTable[Version] + 0x20 * i

    vromStart     = read_uint32_be(entryOffset + 0)
    vromEnd       = read_uint32_be(entryOffset + 4)
    vramStart     = read_uint32_be(entryOffset + 8)
    vramEnd       = read_uint32_be(entryOffset + 0xC)
    loadedRam     = read_uint32_be(entryOffset + 0x10)
    actorInitVram = read_uint32_be(entryOffset + 0x14)
    fileNameAdd   = read_uint32_be(entryOffset + 0x18) # Which?
    allocType     = read_uint16_be(entryOffset + 0x1C)
    numberLoaded  = read_uint8_be(entryOffset + 0x1E)

    print(f"{i:04X},{actorNames[i]},{vromStart:X},{vromEnd:X},{vramStart:X},{vramEnd:X}",end="")
    if PrintAllColumns:
        print(f",{loadedRam:X},{actorInitVram:X},{fileNameAdd:X},{allocType},{numberLoaded}")
    print("")


#####################################################################

def extract_rom(): #(j):
    readFilelists()

    file_names_table = FILE_NAMES[Version]
    if file_names_table is None:
        print(f"'{Edition}' is not supported yet.")
        sys.exit(2)


    filename = CODE_FILE_NAME_V.format(Edition)
    if not os.path.exists(filename):
        print(f"{filename} not found. Defaulting to {CODE_FILE_NAME}")
        filename = CODE_FILE_NAME

    # read baserom data
    try:
        with open(filename, 'rb') as f:
            rom_data = f.read()
    except IOError:
        print('Failed to read file ' + filename)
        sys.exit(1)

    if True:
        initialize_worker(rom_data)#, dmaTable)
        for i in range(ACTOR_ID_MAX):
            ExtractFunc(i)


def main():
    description = "Extracts the dmadata table from the rom. Will try to read the rom 'baserom_version.z64', or 'baserom.z64' if that doesn't exists."

    parser = argparse.ArgumentParser(description=description, formatter_class=argparse.RawTextHelpFormatter)
    choices = [x.lower().replace(" ", "_") for x in offset_ActorOverlayTable]
    parser.add_argument("edition", help="Select the version of the game to extract.", choices=choices, default="pal_mq_dbg", nargs='?')
    # parser.add_argument("-j", help="Enables multiprocessing.", action="store_true")
    parser.add_argument("--show-end", help="Show physical ROM end addresses for uncompressed files", action="store_true")
    parser.add_argument("-a", help="Print the whole table, not just the first five columns", action="store_true")
    args = parser.parse_args()

    global Edition
    global Version
    global showEnd
    global PrintAllColumns

    Edition = args.edition
    Version = Edition.upper().replace("_", " ")
    showEnd = args.show_end
    PrintAllColumns = args.a

    print("")
    print("Id,Name,VROM start,VROM end,VRAM start,VRAM end", end="")
    if PrintAllColumns:
        print(",loaded RAM start,ActorInit VRAM,Filename address,Allocation Type,Number loaded", end="")
    print("")

    extract_rom() #(args.j)

if __name__ == "__main__":
    main()
