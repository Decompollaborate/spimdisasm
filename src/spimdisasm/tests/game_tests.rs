/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{InstructionFlags, Vram};
use spimdisasm::{
    address_range::AddressRange,
    context::{Context, GlobalConfig, InputEndian},
    parent_segment_info::ParentSegmentInfo,
    rom_address::RomAddress,
    sections::{SectionText, SectionTextSettings},
    size::Size,
};

pub enum TestSection {
    Text(RomAddress, &'static str),
    Data(RomAddress, &'static str),
    Rodata(RomAddress, &'static str),
    Bss(Vram, &'static str),

    Bin(RomAddress, &'static str),
}

pub struct TestSegmentInfo {
    #[allow(dead_code)]
    name: &'static str,
    rom: RomAddress,
    vram: Vram,
    noload_size: Option<Size>,
    sections: Vec<TestSection>,
}
pub enum TestSegment {
    Info(TestSegmentInfo),
    EndMarker(RomAddress),
}

impl TestSegmentInfo {
    pub fn vram_end(&self, rom_size: Size) -> Vram {
        let noload_size = if let Some(noload_size) = self.noload_size {
            noload_size
        } else {
            Size::new(0)
        };
        self.vram + rom_size + noload_size
    }
    pub fn vram_from_rom(&self, rom: RomAddress) -> Vram {
        (rom - self.rom) + self.vram
    }
}

pub struct SegmentData {
    name: String,
    #[allow(dead_code)]
    text_sections: Vec<SectionText>,
    // data_sections: Vec<SectionData>,
    // rodata_sections: Vec<SectionRodata>,
    // bss_sections: Vec<SectionBss>,
}

// #[test]
#[allow(dead_code)]
fn drmario64_us() {
    let drmario64_us_segments = vec![
        TestSegment::Info(TestSegmentInfo {
            name: "header",
            rom: RomAddress::new(0x000000),
            vram: Vram::new(0x80000000),
            noload_size: None,
            sections: vec![TestSection::Bin(RomAddress::new(0x000000), "rom_header")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "ipl3",
            rom: RomAddress::new(0x000040),
            vram: Vram::new(0x80000040),
            noload_size: None,
            sections: vec![TestSection::Bin(RomAddress::new(0x000040), "ipl3")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "entry",
            rom: RomAddress::new(0x001000),
            vram: Vram::new(0x80000400),
            noload_size: None,
            sections: vec![TestSection::Text(RomAddress::new(0x001000), "entry")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "boot",
            rom: RomAddress::new(0x001060),
            vram: Vram::new(0x80000460),
            noload_size: Some(Size::new(0x18DE0)),
            sections: vec![
                TestSection::Text(RomAddress::new(0x001060), "boot/boot_main"),
                TestSection::Text(RomAddress::new(0x001220), "boot/dmadata"),
                TestSection::Text(RomAddress::new(0x001320), "gzip/inflate"),
                TestSection::Text(RomAddress::new(0x002B90), "gzip/unzip"),
                TestSection::Text(RomAddress::new(0x002F80), "boot/util"),
                TestSection::Text(
                    RomAddress::new(0x0030A0),
                    "../lib/ultralib/src/io/conteepprobe",
                ),
                TestSection::Text(
                    RomAddress::new(0x003120),
                    "../lib/ultralib/src/io/conteeplongwrite",
                ),
                TestSection::Text(
                    RomAddress::new(0x003210),
                    "../lib/ultralib/src/io/conteeplongread",
                ),
                TestSection::Text(RomAddress::new(0x003280), "../lib/ultralib/src/io/pimgr"),
                TestSection::Text(
                    RomAddress::new(0x003410),
                    "../lib/ultralib/src/io/epirawdma",
                ),
                TestSection::Text(RomAddress::new(0x003600), "../lib/ultralib/src/io/epidma"),
                TestSection::Text(
                    RomAddress::new(0x0036A0),
                    "../lib/ultralib/src/io/cartrominit",
                ),
                TestSection::Text(RomAddress::new(0x003820), "../lib/ultralib/src/io/devmgr"),
                TestSection::Text(RomAddress::new(0x003BE0), "../lib/ultralib/src/io/piacs"),
                TestSection::Text(RomAddress::new(0x003CD0), "../lib/ultralib/src/io/pidma"),
                TestSection::Text(RomAddress::new(0x003D80), "../lib/ultralib/src/io/aigetlen"),
                TestSection::Text(
                    RomAddress::new(0x003D90),
                    "../lib/ultralib/src/io/aigetstat",
                ),
                TestSection::Text(
                    RomAddress::new(0x003DA0),
                    "../lib/ultralib/src/io/aisetfreq",
                ),
                TestSection::Text(
                    RomAddress::new(0x003EC0),
                    "../lib/ultralib/src/io/aisetnextbuf",
                ),
                TestSection::Text(RomAddress::new(0x003F60), "../lib/ultralib/src/audio/env"),
                TestSection::Text(RomAddress::new(0x004C10), "../lib/ultralib/src/audio/sl"),
                TestSection::Text(
                    RomAddress::new(0x004CC0),
                    "../lib/ultralib/src/audio/heapinit",
                ),
                TestSection::Text(
                    RomAddress::new(0x004D00),
                    "../lib/ultralib/src/audio/synthesizer",
                ),
                TestSection::Text(
                    RomAddress::new(0x005340),
                    "../lib/ultralib/src/audio/syndelete",
                ),
                TestSection::Text(
                    RomAddress::new(0x005350),
                    "../lib/ultralib/src/audio/synaddplayer",
                ),
                TestSection::Text(
                    RomAddress::new(0x0053A0),
                    "../lib/ultralib/src/audio/synallocvoice",
                ),
                TestSection::Text(
                    RomAddress::new(0x005580),
                    "../lib/ultralib/src/audio/synstopvoice",
                ),
                TestSection::Text(
                    RomAddress::new(0x005600),
                    "../lib/ultralib/src/audio/synstartvoice",
                ),
                TestSection::Text(
                    RomAddress::new(0x0056A0),
                    "../lib/ultralib/src/audio/synsetpitch",
                ),
                TestSection::Text(
                    RomAddress::new(0x005730),
                    "../lib/ultralib/src/audio/synsetvol",
                ),
                TestSection::Text(
                    RomAddress::new(0x0057F0),
                    "../lib/ultralib/src/audio/synsetfxmix",
                ),
                TestSection::Text(
                    RomAddress::new(0x005890),
                    "../lib/ultralib/src/audio/synsetpan",
                ),
                TestSection::Text(
                    RomAddress::new(0x005920),
                    "../lib/ultralib/src/audio/synallocfx",
                ),
                TestSection::Text(
                    RomAddress::new(0x0059C0),
                    "../lib/ultralib/src/os/invaldcache",
                ),
                TestSection::Text(
                    RomAddress::new(0x005A70),
                    "../lib/ultralib/src/os/writebackdcacheall",
                ),
                TestSection::Text(
                    RomAddress::new(0x005AA0),
                    "../lib/ultralib/src/io/contquery",
                ),
                TestSection::Text(
                    RomAddress::new(0x005B40),
                    "../lib/ultralib/src/io/contreaddata",
                ),
                TestSection::Text(
                    RomAddress::new(0x005D20),
                    "../lib/ultralib/src/io/controller",
                ),
                TestSection::Text(
                    RomAddress::new(0x006030),
                    "../lib/ultralib/src/io/contsetch",
                ),
                TestSection::Text(
                    RomAddress::new(0x006090),
                    "../lib/ultralib/src/os/virtualtophysical",
                ),
                TestSection::Text(RomAddress::new(0x0060F0), "../lib/ultralib/src/gu/sqrtf"),
                TestSection::Text(RomAddress::new(0x006100), "../lib/ultralib/src/gu/cosf"),
                TestSection::Text(RomAddress::new(0x006250), "../lib/ultralib/src/gu/ortho"),
                TestSection::Text(
                    RomAddress::new(0x0064D0),
                    "../lib/ultralib/src/gu/perspective",
                ),
                TestSection::Text(
                    RomAddress::new(0x006850),
                    "../lib/ultralib/src/gu/rotaterpy",
                ),
                TestSection::Text(RomAddress::new(0x006B70), "../lib/ultralib/src/gu/sinf"),
                TestSection::Text(RomAddress::new(0x006D10), "../lib/ultralib/src/gu/sins"),
                TestSection::Text(RomAddress::new(0x006D70), "../lib/ultralib/src/libc/bcmp"),
                TestSection::Text(RomAddress::new(0x006E80), "../lib/ultralib/src/libc/bcopy"),
                TestSection::Text(RomAddress::new(0x0071A0), "../lib/ultralib/src/libc/bzero"),
                TestSection::Text(RomAddress::new(0x007240), "../lib/ultralib/src/libc/string"),
                TestSection::Text(
                    RomAddress::new(0x0072D0),
                    "../lib/ultralib/src/os/createmesgqueue",
                ),
                TestSection::Text(RomAddress::new(0x007300), "../lib/ultralib/src/os/jammesg"),
                TestSection::Text(RomAddress::new(0x007440), "../lib/ultralib/src/os/recvmesg"),
                TestSection::Text(RomAddress::new(0x007570), "../lib/ultralib/src/os/sendmesg"),
                TestSection::Text(
                    RomAddress::new(0x0076A0),
                    "../lib/ultralib/src/os/seteventmesg",
                ),
                TestSection::Text(RomAddress::new(0x007750), "../lib/ultralib/src/io/sptask"),
                TestSection::Text(
                    RomAddress::new(0x007990),
                    "../lib/ultralib/src/io/sptaskyield",
                ),
                TestSection::Text(
                    RomAddress::new(0x0079B0),
                    "../lib/ultralib/src/io/sptaskyielded",
                ),
                TestSection::Text(RomAddress::new(0x007A00), "../lib/ultralib/src/sched/sched"),
                TestSection::Text(RomAddress::new(0x008410), "../lib/ultralib/src/io/sirawdma"),
                TestSection::Text(RomAddress::new(0x0084C0), "../lib/ultralib/src/io/siacs"),
                TestSection::Text(
                    RomAddress::new(0x0085B0),
                    "../lib/ultralib/src/os/createthread",
                ),
                TestSection::Text(
                    RomAddress::new(0x008680),
                    "../lib/ultralib/src/os/getthreadpri",
                ),
                TestSection::Text(
                    RomAddress::new(0x0086A0),
                    "../lib/ultralib/src/os/setthreadpri",
                ),
                TestSection::Text(
                    RomAddress::new(0x008770),
                    "../lib/ultralib/src/os/startthread",
                ),
                TestSection::Text(
                    RomAddress::new(0x008890),
                    "../lib/ultralib/src/os/stopthread",
                ),
                TestSection::Text(RomAddress::new(0x008950), "../lib/ultralib/src/os/thread"),
                TestSection::Text(
                    RomAddress::new(0x008990),
                    "../lib/ultralib/src/os/yieldthread",
                ),
                TestSection::Text(RomAddress::new(0x0089E0), "../lib/ultralib/src/os/gettime"),
                TestSection::Text(RomAddress::new(0x008A70), "../lib/ultralib/src/os/settime"),
                TestSection::Text(RomAddress::new(0x008A90), "../lib/ultralib/src/os/settimer"),
                TestSection::Text(
                    RomAddress::new(0x008BC0),
                    "../lib/ultralib/src/os/timerintr",
                ),
                TestSection::Text(RomAddress::new(0x008EE0), "../lib/ultralib/src/os/probetlb"),
                TestSection::Text(
                    RomAddress::new(0x008FA0),
                    "../lib/ultralib/src/io/vigetcurrframebuf",
                ),
                TestSection::Text(
                    RomAddress::new(0x008FE0),
                    "../lib/ultralib/src/io/vigetnextframebuf",
                ),
                TestSection::Text(RomAddress::new(0x009020), "../lib/ultralib/src/io/vimgr"),
                TestSection::Text(
                    RomAddress::new(0x009360),
                    "../lib/ultralib/src/io/visetevent",
                ),
                TestSection::Text(
                    RomAddress::new(0x0093C0),
                    "../lib/ultralib/src/io/visetmode",
                ),
                TestSection::Text(
                    RomAddress::new(0x009410),
                    "../lib/ultralib/src/io/visetspecial",
                ),
                TestSection::Text(
                    RomAddress::new(0x009580),
                    "../lib/ultralib/src/io/visetyscale",
                ),
                TestSection::Text(
                    RomAddress::new(0x0095D0),
                    "../lib/ultralib/src/io/viswapbuf",
                ),
                TestSection::Text(
                    RomAddress::new(0x009620),
                    "../lib/ultralib/src/io/viswapcontext",
                ),
                TestSection::Text(RomAddress::new(0x009930), "../lib/ultralib/src/io/viblack"),
                TestSection::Text(
                    RomAddress::new(0x0099A0),
                    "../lib/ultralib/src/mgu/mtxident",
                ),
                TestSection::Text(
                    RomAddress::new(0x009A00),
                    "../lib/ultralib/src/mgu/mtxidentf",
                ),
                TestSection::Text(RomAddress::new(0x009A60), "../lib/ultralib/src/mgu/mtxf2l"),
                TestSection::Text(RomAddress::new(0x009AE0), "../lib/ultralib/src/mgu/mtxl2f"),
                TestSection::Text(RomAddress::new(0x009B60), "../lib/ultralib/src/mgu/mtxcatf"),
                TestSection::Text(RomAddress::new(0x009C60), "../lib/ultralib/src/mgu/scale"),
                TestSection::Text(RomAddress::new(0x009D20), "../lib/ultralib/src/mgu/scalef"),
                TestSection::Text(
                    RomAddress::new(0x009D80),
                    "../lib/ultralib/src/mgu/translate",
                ),
                TestSection::Text(
                    RomAddress::new(0x009E60),
                    "../lib/ultralib/src/mgu/translatef",
                ),
                TestSection::Text(
                    RomAddress::new(0x009EC0),
                    "../lib/ultralib/src/os/initialize",
                ),
                TestSection::Text(
                    RomAddress::new(0x00A260),
                    "../lib/ultralib/src/io/conteepread",
                ),
                TestSection::Text(
                    RomAddress::new(0x00A480),
                    "../lib/ultralib/src/io/conteepwrite",
                ),
                TestSection::Text(
                    RomAddress::new(0x00A830),
                    "../lib/ultralib/src/os/exceptasm",
                ),
                TestSection::Text(
                    RomAddress::new(0x00B1A0),
                    "../lib/ultralib/src/os/interrupt",
                ),
                TestSection::Text(
                    RomAddress::new(0x00B230),
                    "../lib/ultralib/src/os/setintmask",
                ),
                TestSection::Text(
                    RomAddress::new(0x00B2D0),
                    "../lib/ultralib/src/os/setglobalintmask",
                ),
                TestSection::Text(
                    RomAddress::new(0x00B310),
                    "../lib/ultralib/src/os/resetglobalintmask",
                ),
                TestSection::Text(RomAddress::new(0x00B360), "../lib/ultralib/src/io/pirawdma"),
                TestSection::Text(
                    RomAddress::new(0x00B430),
                    "../lib/ultralib/src/io/pigetcmdq",
                ),
                TestSection::Text(
                    RomAddress::new(0x00B450),
                    "../lib/ultralib/src/io/epirawread",
                ),
                TestSection::Text(
                    RomAddress::new(0x00B5C0),
                    "../lib/ultralib/src/io/epirawwrite",
                ),
                TestSection::Text(RomAddress::new(0x00B730), "../lib/ultralib/src/io/ai"),
                TestSection::Text(
                    RomAddress::new(0x00B750),
                    "../lib/ultralib/src/audio/drvrnew",
                ),
                TestSection::Text(RomAddress::new(0x00BF80), "../lib/ultralib/src/audio/load"),
                TestSection::Text(
                    RomAddress::new(0x00CA90),
                    "../lib/ultralib/src/audio/auxbus",
                ),
                TestSection::Text(
                    RomAddress::new(0x00CBA0),
                    "../lib/ultralib/src/audio/filter",
                ),
                TestSection::Text(
                    RomAddress::new(0x00CBC0),
                    "../lib/ultralib/src/audio/mainbus",
                ),
                TestSection::Text(
                    RomAddress::new(0x00CD30),
                    "../lib/ultralib/src/audio/resample",
                ),
                TestSection::Text(
                    RomAddress::new(0x00CFE0),
                    "../lib/ultralib/src/audio/reverb",
                ),
                TestSection::Text(RomAddress::new(0x00DB80), "../lib/ultralib/src/audio/save"),
                TestSection::Text(
                    RomAddress::new(0x00DC50),
                    "../lib/ultralib/src/audio/heapalloc",
                ),
                TestSection::Text(RomAddress::new(0x00DCA0), "../lib/ultralib/src/audio/copy"),
                TestSection::Text(
                    RomAddress::new(0x00DCE0),
                    "../lib/ultralib/src/os/invalicache",
                ),
                TestSection::Text(
                    RomAddress::new(0x00DD60),
                    "../lib/ultralib/src/os/writebackdcache",
                ),
                TestSection::Text(
                    RomAddress::new(0x00DDE0),
                    "../lib/ultralib/src/io/dpsetnextbuf",
                ),
                TestSection::Text(RomAddress::new(0x00DE80), "../lib/ultralib/src/os/getcause"),
                TestSection::Text(RomAddress::new(0x00DE90), "../lib/ultralib/src/os/getcount"),
                TestSection::Text(RomAddress::new(0x00DEA0), "../lib/ultralib/src/os/getsr"),
                TestSection::Text(
                    RomAddress::new(0x00DEB0),
                    "../lib/ultralib/src/os/setcompare",
                ),
                TestSection::Text(
                    RomAddress::new(0x00DEC0),
                    "../lib/ultralib/src/os/setfpccsr",
                ),
                TestSection::Text(RomAddress::new(0x00DED0), "../lib/ultralib/src/os/setsr"),
                TestSection::Text(
                    RomAddress::new(0x00DEE0),
                    "../lib/ultralib/src/os/setwatchlo",
                ),
                TestSection::Text(RomAddress::new(0x00DEF0), "../lib/ultralib/src/io/sp"),
                TestSection::Text(
                    RomAddress::new(0x00DF10),
                    "../lib/ultralib/src/io/spgetstat",
                ),
                TestSection::Text(
                    RomAddress::new(0x00DF20),
                    "../lib/ultralib/src/io/spsetstat",
                ),
                TestSection::Text(RomAddress::new(0x00DF30), "../lib/ultralib/src/io/spsetpc"),
                TestSection::Text(RomAddress::new(0x00DF60), "../lib/ultralib/src/io/sprawdma"),
                TestSection::Text(
                    RomAddress::new(0x00DFF0),
                    "../lib/ultralib/src/io/sirawread",
                ),
                TestSection::Text(
                    RomAddress::new(0x00E040),
                    "../lib/ultralib/src/io/sirawwrite",
                ),
                TestSection::Text(
                    RomAddress::new(0x00E090),
                    "../lib/ultralib/src/os/destroythread",
                ),
                TestSection::Text(
                    RomAddress::new(0x00E170),
                    "../lib/ultralib/src/os/maptlbrdb",
                ),
                TestSection::Text(
                    RomAddress::new(0x00E1D0),
                    "../lib/ultralib/src/os/unmaptlball",
                ),
                TestSection::Text(RomAddress::new(0x00E220), "../lib/ultralib/src/io/vi"),
                TestSection::Text(
                    RomAddress::new(0x00E330),
                    "../lib/ultralib/src/io/vigetcurrcontext",
                ),
                TestSection::Text(
                    RomAddress::new(0x00E340),
                    "../lib/ultralib/src/os/parameters",
                ),
                TestSection::Text(RomAddress::new(0x00E3A0), "../lib/ultralib/src/io/dp"),
                TestSection::Text(RomAddress::new(0x00E3C0), "../lib/ultralib/src/io/si"),
                TestSection::Text(RomAddress::new(0x00E3E0), "libkmc/fmod"),
                TestSection::Text(RomAddress::new(0x00E5C0), "libkmc/memmove"),
                TestSection::Text(RomAddress::new(0x00E730), "libkmc/memset"),
                TestSection::Text(RomAddress::new(0x00E800), "libkmc/modf"),
                TestSection::Text(RomAddress::new(0x00E8B0), "libkmc/rand"),
                TestSection::Text(RomAddress::new(0x00E8F0), "libkmc/strcpy"),
                TestSection::Text(RomAddress::new(0x00E9C0), "libkmc/mmuldi3"),
                TestSection::Data(RomAddress::new(0x00ED90), "boot/boot_main"),
                TestSection::Data(RomAddress::new(0x00EDA0), "boot/dmadata"),
                TestSection::Data(RomAddress::new(0x00EDB0), "gzip/inflate"),
                TestSection::Data(RomAddress::new(0x00EF30), "gzip/unzip"),
                TestSection::Data(RomAddress::new(0x00F340), "boot/rom_offsets"),
                TestSection::Data(RomAddress::new(0x00F650), "../lib/ultralib/src/io/pimgr"),
                TestSection::Data(
                    RomAddress::new(0x00F680),
                    "../lib/ultralib/src/io/cartrominit",
                ),
                TestSection::Data(RomAddress::new(0x00F690), "../lib/ultralib/src/io/piacs"),
                TestSection::Data(
                    RomAddress::new(0x00F6A0),
                    "../lib/ultralib/src/io/aisetnextbuf",
                ),
                TestSection::Data(RomAddress::new(0x00F6B0), "../lib/ultralib/src/audio/env"),
                TestSection::Data(RomAddress::new(0x00F7B0), "../lib/ultralib/src/audio/sl"),
                TestSection::Data(
                    RomAddress::new(0x00F7C0),
                    "../lib/ultralib/src/io/controller",
                ),
                TestSection::Data(
                    RomAddress::new(0x00F7D0),
                    "../lib/ultralib/src/gu/rotaterpy",
                ),
                TestSection::Data(RomAddress::new(0x00F7E0), "../lib/ultralib/src/gu/sins"),
                TestSection::Data(
                    RomAddress::new(0x00FFE0),
                    "../lib/ultralib/src/os/seteventmesg",
                ),
                TestSection::Data(RomAddress::new(0x00FFF0), "../lib/ultralib/src/sched/sched"),
                TestSection::Data(RomAddress::new(0x010000), "../lib/ultralib/src/io/siacs"),
                TestSection::Data(RomAddress::new(0x010010), "../lib/ultralib/src/os/thread"),
                TestSection::Data(
                    RomAddress::new(0x010030),
                    "../lib/ultralib/src/os/timerintr",
                ),
                TestSection::Data(RomAddress::new(0x010040), "../lib/ultralib/src/io/vimgr"),
                TestSection::Data(RomAddress::new(0x010060), "../lib/ultralib/src/io/vitbl"),
                TestSection::Data(
                    RomAddress::new(0x0111E0),
                    "../lib/ultralib/src/os/initialize",
                ),
                TestSection::Data(
                    RomAddress::new(0x011200),
                    "../lib/ultralib/src/os/exceptasm",
                ),
                TestSection::Data(
                    RomAddress::new(0x011230),
                    "../lib/ultralib/src/audio/drvrnew",
                ),
                TestSection::Data(
                    RomAddress::new(0x0113C0),
                    "../lib/ultralib/src/audio/reverb",
                ),
                TestSection::Data(RomAddress::new(0x0113E0), "../lib/ultralib/src/io/vi"),
                TestSection::Data(
                    RomAddress::new(0x011450),
                    "../lib/ultralib/src/vimodes/vimodentsclan1",
                ),
                TestSection::Data(
                    RomAddress::new(0x0114A0),
                    "../lib/ultralib/src/vimodes/vimodepallan1",
                ),
                TestSection::Data(
                    RomAddress::new(0x0114F0),
                    "../lib/ultralib/src/vimodes/vimodempallan1",
                ),
                TestSection::Data(RomAddress::new(0x011540), "libkmc/ctype"),
                TestSection::Rodata(RomAddress::new(0x011640), "../lib/ultralib/src/io/devmgr"),
                TestSection::Rodata(
                    RomAddress::new(0x011660),
                    "../lib/ultralib/src/io/aisetfreq",
                ),
                TestSection::Rodata(RomAddress::new(0x011670), "../lib/ultralib/src/audio/env"),
                TestSection::Rodata(
                    RomAddress::new(0x011760),
                    "../lib/ultralib/src/audio/synthesizer",
                ),
                TestSection::Rodata(RomAddress::new(0x011780), "../lib/ultralib/src/gu/cosf"),
                TestSection::Rodata(
                    RomAddress::new(0x0117E0),
                    "../lib/ultralib/src/gu/perspective",
                ),
                TestSection::Rodata(RomAddress::new(0x011820), "../lib/ultralib/src/gu/sinf"),
                TestSection::Rodata(RomAddress::new(0x011880), "../lib/ultralib/src/sched/sched"),
                TestSection::Rodata(
                    RomAddress::new(0x0118A0),
                    "../lib/ultralib/src/io/viswapcontext",
                ),
                TestSection::Rodata(
                    RomAddress::new(0x0118B0),
                    "../lib/ultralib/src/os/exceptasm",
                ),
                TestSection::Rodata(
                    RomAddress::new(0x011900),
                    "../lib/ultralib/src/os/setintmask",
                ),
                TestSection::Rodata(
                    RomAddress::new(0x011980),
                    "../lib/ultralib/src/audio/drvrnew",
                ),
                TestSection::Rodata(
                    RomAddress::new(0x0119C0),
                    "../lib/ultralib/src/audio/resample",
                ),
                TestSection::Rodata(
                    RomAddress::new(0x0119F0),
                    "../lib/ultralib/src/audio/reverb",
                ),
                TestSection::Rodata(
                    RomAddress::new(0x011A40),
                    "../lib/ultralib/src/gu/libm_vals",
                ),
                TestSection::Rodata(RomAddress::new(0x011A50), "libkmc/fmod"),
                TestSection::Bss(Vram::new(0x80010E60), "boot/boot_main"),
                TestSection::Bss(Vram::new(0x800151C0), "boot/dmadata"),
                TestSection::Bss(Vram::new(0x800151E0), "gzip/inflate"),
                TestSection::Bss(Vram::new(0x800191E0), "../lib/ultralib/src/io/pimgr"),
                TestSection::Bss(Vram::new(0x8001A3B0), "../lib/ultralib/src/io/piacs"),
                TestSection::Bss(Vram::new(0x8001A3C0), "../lib/ultralib/src/io/sptask"),
                TestSection::Bss(Vram::new(0x8001A400), "../lib/ultralib/src/io/siacs"),
                TestSection::Bss(Vram::new(0x8001A410), "../lib/ultralib/src/io/vimgr"),
                TestSection::Bss(Vram::new(0x8001B630), "libkmc/rand"),
                TestSection::Bss(Vram::new(0x8001B640), "boot/COMMON"),
            ],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "dma_table",
            rom: RomAddress::new(0x011A60),
            vram: Vram::new(0x80029C40),
            noload_size: None,
            sections: vec![TestSection::Text(RomAddress::new(0x011A60), "dma_table")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "main_segment",
            rom: RomAddress::new(0x011A70),
            vram: Vram::new(0x80029C50),
            noload_size: Some(Size::new(0x70FD0)),
            sections: vec![
                TestSection::Text(RomAddress::new(0x011A70), "main_segment/main"),
                TestSection::Text(RomAddress::new(0x011CF0), "main_segment/nnsched"),
                TestSection::Text(RomAddress::new(0x0123D0), "main_segment/joy"),
                TestSection::Text(RomAddress::new(0x0128A0), "main_segment/audio/sound"),
                TestSection::Text(RomAddress::new(0x0132B0), "main_segment/graphic"),
                TestSection::Text(RomAddress::new(0x013B40), "main_segment/graphics/static"),
                TestSection::Text(RomAddress::new(0x014F90), "main_segment/audio/music_driver"),
                TestSection::Text(RomAddress::new(0x015940), "main_segment/main1x"),
                TestSection::Text(RomAddress::new(0x016650), "main_segment/vr_init"),
                TestSection::Text(RomAddress::new(0x016920), "main_segment/aiset"),
                TestSection::Text(RomAddress::new(0x01E3D0), "main_segment/record"),
                TestSection::Text(RomAddress::new(0x020D10), "main_segment/unused/020D10"),
                TestSection::Text(RomAddress::new(0x0227C0), "main_segment/game_etc"),
                TestSection::Text(
                    RomAddress::new(0x024BC0),
                    "main_segment/screen_print/printer",
                ),
                TestSection::Text(
                    RomAddress::new(0x024C40),
                    "main_segment/screen_print/printf_impl",
                ),
                TestSection::Text(
                    RomAddress::new(0x026000),
                    "main_segment/screen_print/debug_print",
                ),
                TestSection::Text(RomAddress::new(0x026550), "main_segment/debug_menu"),
                TestSection::Text(RomAddress::new(0x028820), "main_segment/bg_tasks"),
                TestSection::Text(RomAddress::new(0x0289D0), "main_segment/tex_func"),
                TestSection::Text(RomAddress::new(0x02DBF0), "main_segment/main_menu"),
                TestSection::Text(RomAddress::new(0x043610), "main_segment/font"),
                TestSection::Text(RomAddress::new(0x044D40), "main_segment/msgwnd"),
                TestSection::Text(RomAddress::new(0x045F40), "main_segment/char_anime"),
                TestSection::Text(RomAddress::new(0x046D70), "main_segment/dm_virus_init"),
                TestSection::Text(RomAddress::new(0x048010), "main_segment/dm_game_main"),
                TestSection::Text(RomAddress::new(0x059D10), "main_segment/dm_manual_main"),
                TestSection::Text(RomAddress::new(0x05DD50), "main_segment/dm_title_main"),
                TestSection::Text(RomAddress::new(0x05E700), "main_segment/main_story"),
                TestSection::Text(RomAddress::new(0x063470), "main_segment/lws"),
                TestSection::Text(RomAddress::new(0x063A40), "main_segment/calc"),
                TestSection::Text(RomAddress::new(0x065080), "main_segment/unused/065080"),
                TestSection::Text(RomAddress::new(0x066080), "main_segment/unused/066080"),
                TestSection::Text(RomAddress::new(0x066100), "main_segment/replay"),
                TestSection::Text(RomAddress::new(0x066580), "main_segment/066580"),
                TestSection::Text(RomAddress::new(0x066840), "main_segment/066840"),
                TestSection::Text(RomAddress::new(0x0676B0), "main_segment/unused/055C40"),
                TestSection::Text(RomAddress::new(0x067820), "../lib/libmus/src/player"),
                TestSection::Text(RomAddress::new(0x06B100), "../lib/libmus/src/player_fx"),
                TestSection::Text(RomAddress::new(0x06B9E0), "../lib/libmus/src/aud_dma"),
                TestSection::Text(RomAddress::new(0x06BE40), "../lib/libmus/src/aud_sched"),
                TestSection::Text(RomAddress::new(0x06BFF0), "../lib/libmus/src/aud_thread"),
                TestSection::Text(RomAddress::new(0x06C2B0), "../lib/libmus/src/lib_memory"),
                TestSection::Text(RomAddress::new(0x06C3F0), "../lib/libmus/src/aud_samples"),
                TestSection::Bin(RomAddress::new(0x06C4B0), "rsp/rspboot"),
                TestSection::Bin(RomAddress::new(0x06C580), "rsp/aspMain"),
                TestSection::Bin(RomAddress::new(0x06D3A0), "rsp/f3dex2"),
                TestSection::Bin(RomAddress::new(0x06E730), "rsp/s2dex"),
                TestSection::Data(RomAddress::new(0x06FF20), "main_segment/nnsched"),
                TestSection::Data(RomAddress::new(0x06FF30), "main_segment/graphic"),
                TestSection::Data(RomAddress::new(0x06FF70), "main_segment/graphics/static"),
                TestSection::Data(RomAddress::new(0x070210), "main_segment/audio/music_driver"),
                TestSection::Data(RomAddress::new(0x070220), "main_segment/main1x"),
                TestSection::Data(RomAddress::new(0x0702A0), "main_segment/vr_init"),
                TestSection::Data(RomAddress::new(0x0702B0), "main_segment/aiset"),
                TestSection::Data(RomAddress::new(0x0715C0), "main_segment/record"),
                TestSection::Data(RomAddress::new(0x074DC0), "main_segment/unused/020D10"),
                TestSection::Data(RomAddress::new(0x074E10), "main_segment/game_etc"),
                TestSection::Data(
                    RomAddress::new(0x075000),
                    "main_segment/screen_print/printf_impl",
                ),
                TestSection::Data(
                    RomAddress::new(0x0750C0),
                    "main_segment/screen_print/debug_print",
                ),
                TestSection::Data(RomAddress::new(0x076190), "main_segment/debug_menu"),
                TestSection::Data(RomAddress::new(0x0763F0), "main_segment/bg_tasks"),
                TestSection::Data(RomAddress::new(0x076400), "main_segment/tex_func"),
                TestSection::Data(RomAddress::new(0x076590), "main_segment/main_menu"),
                TestSection::Data(RomAddress::new(0x0766E0), "main_segment/font"),
                TestSection::Data(RomAddress::new(0x08EDA0), "main_segment/dm_game_main"),
                TestSection::Data(RomAddress::new(0x08F1B0), "main_segment/dm_title_main"),
                TestSection::Data(RomAddress::new(0x08F1C0), "main_segment/main_story"),
                TestSection::Data(RomAddress::new(0x092C80), "main_segment/calc"),
                TestSection::Data(RomAddress::new(0x092F90), "main_segment/066580"),
                TestSection::Data(RomAddress::new(0x093140), "main_segment/066840"),
                TestSection::Data(RomAddress::new(0x093A70), "../lib/libmus/src/player"),
                TestSection::Data(RomAddress::new(0x093B30), "../lib/libmus/src/player_fx"),
                TestSection::Data(RomAddress::new(0x093D40), "../lib/libmus/src/aud_sched"),
                TestSection::Data(RomAddress::new(0x093D50), "../lib/libmus/src/aud_thread"),
                TestSection::Bin(RomAddress::new(0x093D60), "rsp/aspMain"),
                TestSection::Bin(RomAddress::new(0x094020), "rsp/f3dex2"),
                TestSection::Bin(RomAddress::new(0x094440), "rsp/s2dex"),
                TestSection::Rodata(RomAddress::new(0x094800), "main_segment/main"),
                TestSection::Rodata(RomAddress::new(0x094830), "main_segment/audio/sound"),
                TestSection::Rodata(RomAddress::new(0x094B20), "main_segment/graphic"),
                TestSection::Rodata(RomAddress::new(0x094B40), "main_segment/main1x"),
                TestSection::Rodata(RomAddress::new(0x094CD0), "main_segment/aiset"),
                TestSection::Rodata(RomAddress::new(0x094D60), "main_segment/record"),
                TestSection::Rodata(RomAddress::new(0x094DD0), "main_segment/unused/020D10"),
                TestSection::Rodata(RomAddress::new(0x094F70), "main_segment/game_etc"),
                TestSection::Rodata(
                    RomAddress::new(0x094FB0),
                    "main_segment/screen_print/printf_impl",
                ),
                TestSection::Rodata(RomAddress::new(0x095100), "main_segment/debug_menu"),
                TestSection::Rodata(RomAddress::new(0x095960), "main_segment/tex_func"),
                TestSection::Rodata(RomAddress::new(0x095A30), "main_segment/main_menu"),
                TestSection::Rodata(RomAddress::new(0x099770), "main_segment/font"),
                TestSection::Rodata(RomAddress::new(0x0997B0), "main_segment/msgwnd"),
                TestSection::Rodata(RomAddress::new(0x099880), "main_segment/char_anime"),
                TestSection::Rodata(RomAddress::new(0x0999C0), "main_segment/dm_virus_init"),
                TestSection::Rodata(RomAddress::new(0x099A50), "main_segment/dm_game_main"),
                TestSection::Rodata(RomAddress::new(0x09A220), "main_segment/dm_manual_main"),
                TestSection::Rodata(RomAddress::new(0x09B060), "main_segment/dm_title_main"),
                TestSection::Rodata(RomAddress::new(0x09B180), "main_segment/main_story"),
                TestSection::Rodata(RomAddress::new(0x09B270), "main_segment/lws"),
                TestSection::Rodata(RomAddress::new(0x09B2C0), "main_segment/calc"),
                TestSection::Rodata(RomAddress::new(0x09B310), "main_segment/unused/065080"),
                TestSection::Rodata(RomAddress::new(0x09B370), "../lib/libmus/src/player"),
                TestSection::Rodata(RomAddress::new(0x09B450), "../lib/libmus/src/player_fx"),
                TestSection::Bss(Vram::new(0x800B3640), "main_segment/audio/sound"),
                TestSection::Bss(Vram::new(0x800E4640), "main_segment/vr_init"),
                TestSection::Bss(Vram::new(0x800E5000), "main_segment/aiset"),
                TestSection::Bss(Vram::new(0x800E51B0), "main_segment/record"),
                TestSection::Bss(Vram::new(0x800E53B0), "main_segment/unused/020D10"),
                TestSection::Bss(Vram::new(0x800E53C0), "main_segment/game_etc"),
                TestSection::Bss(
                    Vram::new(0x800E5860),
                    "main_segment/screen_print/printf_impl",
                ),
                TestSection::Bss(
                    Vram::new(0x800E5890),
                    "main_segment/screen_print/debug_print",
                ),
                TestSection::Bss(Vram::new(0x800E58B0), "main_segment/debug_menu"),
                TestSection::Bss(Vram::new(0x800E5980), "main_segment/dm_game_main"),
                TestSection::Bss(Vram::new(0x800E5990), "main_segment/dm_manual_main"),
                TestSection::Bss(Vram::new(0x800E59A0), "main_segment/dm_title_main"),
                TestSection::Bss(Vram::new(0x800E59B0), "main_segment/main_story"),
                TestSection::Bss(Vram::new(0x800E87C0), "main_segment/lws"),
                TestSection::Bss(Vram::new(0x800E97C0), "main_segment/calc"),
                TestSection::Bss(Vram::new(0x800E98B0), "main_segment/replay"),
                TestSection::Bss(Vram::new(0x800E9910), "../lib/libmus/src/player"),
                TestSection::Bss(Vram::new(0x800E9980), "../lib/libmus/src/aud_dma"),
                TestSection::Bss(Vram::new(0x800E99C0), "../lib/libmus/src/aud_sched"),
                TestSection::Bss(Vram::new(0x800E99D0), "../lib/libmus/src/aud_thread"),
                TestSection::Bss(Vram::new(0x800E9B90), "../lib/libmus/src/lib_memory"),
                TestSection::Bss(Vram::new(0x800E9BA0), "../lib/libmus/src/aud_samples"),
                TestSection::Bss(Vram::new(0x800E9BB0), "main_segment/COMMON"),
            ],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "buffer1",
            rom: RomAddress::new(0x09B460),
            vram: Vram::new(0x80124610),
            noload_size: Some(Size::new(0xE09F0)),
            sections: vec![TestSection::Bss(Vram::new(0x80124610), "buffers/buffer1")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "buffer2",
            rom: RomAddress::new(0x09B460),
            vram: Vram::new(0x80205000),
            noload_size: Some(Size::new(0x1B0000)),
            sections: vec![TestSection::Bss(Vram::new(0x80205000), "buffers/buffer2")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "buffer1",
            rom: RomAddress::new(0x09B460),
            vram: Vram::new(0x803B5000),
            noload_size: Some(Size::new(0x4B000)),
            sections: vec![TestSection::Bss(
                Vram::new(0x803B5000),
                "buffers/framebuffer",
            )],
        }),
        TestSegment::EndMarker(RomAddress::new(0x09B460)),
    ];

    let mut rom_start = None;
    let mut rom_end = None;
    let mut vram_start = None;
    let mut vram_end = None;

    for w in drmario64_us_segments.windows(2) {
        let a = &w[0];
        let b = &w[1];

        match (a, b) {
            (TestSegment::EndMarker(..), _) => {
                panic!("Doesn't make sense")
            }
            (TestSegment::Info(x), TestSegment::EndMarker(y)) => {
                assert!(x.rom <= *y);

                rom_start.get_or_insert(x.rom);
                vram_start.get_or_insert(x.vram);

                rom_end = Some(*y);
                vram_end = Some(x.vram_end(*y - x.rom));
            }
            (TestSegment::Info(x), TestSegment::Info(y)) => {
                assert!(x.rom <= y.rom);

                rom_start.get_or_insert(x.rom);
                vram_start.get_or_insert(x.vram);

                rom_end.get_or_insert(y.rom);
                vram_end.get_or_insert(x.vram_end(y.rom - x.rom));
            }
        }
    }

    let rom_bytes = std::fs::read("../../baserom_uncompressed.us.z64").unwrap();

    let global_rom_range = AddressRange::new(rom_start.unwrap(), rom_end.unwrap());
    let global_vram_range = AddressRange::new(vram_start.unwrap(), vram_end.unwrap());

    let mut context = Context::new(
        GlobalConfig::new(InputEndian::Big),
        global_rom_range,
        global_vram_range,
    );

    let mut segments = Vec::new();
    for w in drmario64_us_segments.windows(2) {
        let a = &w[0];
        let b = &w[1];
        match (a, b) {
            (TestSegment::EndMarker(..), _) => panic!(),
            (
                TestSegment::Info(info),
                TestSegment::Info(TestSegmentInfo {
                    rom: segment_rom_end,
                    ..
                })
                | TestSegment::EndMarker(segment_rom_end),
            ) => {
                let mut text_sections = Vec::new();

                let parent_segment_info = ParentSegmentInfo::new(info.rom, None);

                for (i, sect) in info.sections.iter().enumerate() {
                    let rom_end = if i + 1 < info.sections.len() {
                        match info.sections[i + 1] {
                            TestSection::Text(rom, _)
                            | TestSection::Data(rom, _)
                            | TestSection::Rodata(rom, _)
                            | TestSection::Bin(rom, _) => rom,
                            TestSection::Bss(..) => *segment_rom_end,
                        }
                    } else {
                        *segment_rom_end
                    };

                    match sect {
                        TestSection::Text(rom, name) => {
                            let text_settings =
                                SectionTextSettings::new(InstructionFlags::default());
                            text_sections.push(
                                SectionText::new(
                                    &mut context,
                                    text_settings,
                                    (*name).into(),
                                    &rom_bytes[rom.inner() as usize..rom_end.inner() as usize],
                                    *rom,
                                    info.vram_from_rom(*rom),
                                    parent_segment_info.clone(),
                                )
                                .unwrap(),
                            );
                        }
                        TestSection::Data(..) => {}
                        TestSection::Rodata(..) => {}
                        TestSection::Bss(..) => {}
                        TestSection::Bin(..) => {}
                    }
                }
                segments.push(SegmentData {
                    name: info.name.into(),
                    text_sections,
                });
            }
        }
    }

    assert_eq!(context.global_segment().symbols().len(), 1402);

    /*
    for seg in &segments {
        for sect in &seg.text_sections {
            println!("{} {} {}", seg.name, sect.name(), sect.functions().len());
        }
    }
    */
    let function_count: usize = segments
        .iter()
        .map(|x| {
            x.text_sections
                .iter()
                .map(|y| y.functions().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(function_count, 1402);
}
