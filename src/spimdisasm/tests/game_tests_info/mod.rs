/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![allow(dead_code)]

use std::collections::BTreeMap;

use spimdisasm::{
    addresses::{Rom, Size, Vram},
    context::Context,
    metadata::{RodataMigrationBehavior, SymbolType},
    sections::{
        before_proc::{DataSection, ExecutableSection, NoloadSection},
        processed::{DataSectionProcessed, ExecutableSectionProcessed, NoloadSectionProcessed},
        SectionPostProcessError,
    },
};

pub enum TestSection {
    Text(Rom, &'static str),
    Data(Rom, &'static str),
    Rodata(Rom, &'static str),
    Bss(Vram, &'static str),

    Bin(Rom, &'static str),
}

pub struct TestSegmentInfo {
    pub name: &'static str,
    pub rom: Rom,
    pub vram: Vram,
    pub noload_size: Option<Size>,
    pub sections: Vec<TestSection>,
}
pub enum TestSegment {
    Info(TestSegmentInfo),
    EndMarker(Rom),
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
    pub fn vram_from_rom(&self, rom: Rom) -> Vram {
        (rom - self.rom) + self.vram
    }
}

pub struct SegmentData {
    pub name: String,
    pub text_sections: Vec<ExecutableSection>,
    pub data_sections: Vec<DataSection>,
    pub rodata_sections: Vec<DataSection>,
    pub bss_sections: Vec<NoloadSection>,
}

impl SegmentData {
    pub fn post_process(self, context: &mut Context) -> SegmentDataProcessed {
        let user_relocs = BTreeMap::new();
        SegmentDataProcessed {
            name: self.name,
            text_sections: self
                .text_sections
                .into_iter()
                .map(|x| x.post_process(context, &user_relocs))
                .collect::<Result<Vec<ExecutableSectionProcessed>, SectionPostProcessError>>()
                .unwrap(),
            data_sections: self
                .data_sections
                .into_iter()
                .map(|x| x.post_process(context, &user_relocs))
                .collect::<Result<Vec<DataSectionProcessed>, SectionPostProcessError>>()
                .unwrap(),
            rodata_sections: self
                .rodata_sections
                .into_iter()
                .map(|x| x.post_process(context, &user_relocs))
                .collect::<Result<Vec<DataSectionProcessed>, SectionPostProcessError>>()
                .unwrap(),
            bss_sections: self
                .bss_sections
                .into_iter()
                .map(|x| x.post_process(context))
                .collect::<Result<Vec<NoloadSectionProcessed>, SectionPostProcessError>>()
                .unwrap(),
        }
    }
}

pub struct SegmentDataProcessed {
    pub name: String,
    pub text_sections: Vec<ExecutableSectionProcessed>,
    pub data_sections: Vec<DataSectionProcessed>,
    pub rodata_sections: Vec<DataSectionProcessed>,
    pub bss_sections: Vec<NoloadSectionProcessed>,
}

pub struct UserSymbolInfo {
    pub name: String,
    pub vram: Vram,
    pub rom: Option<Rom>,
    pub name_end: Option<String>,
    pub size: Option<Size>,
    pub typ: Option<SymbolType>,
    pub migration_behavior: RodataMigrationBehavior,
    pub dont_allow_addend: bool,
}
pub enum UserSymbol {
    Info(UserSymbolInfo),
    Ignored(Vram, Option<Size>),
}

impl UserSymbol {
    pub fn new(name: String, vram: Vram) -> Self {
        UserSymbol::Info(UserSymbolInfo {
            name,
            vram,
            rom: None,
            name_end: None,
            size: None,
            typ: None,
            migration_behavior: RodataMigrationBehavior::Default(),
            dont_allow_addend: false,
        })
    }
    pub fn new_ignored(vram: Vram) -> Self {
        UserSymbol::Ignored(vram, None)
    }

    pub fn with_rom(self, rom: Rom) -> Self {
        match self {
            UserSymbol::Info(s) => UserSymbol::Info(UserSymbolInfo {
                rom: Some(rom),
                ..s
            }),
            UserSymbol::Ignored(_vram, _) => panic!("no"),
        }
    }
    pub fn with_name_end(self, name_end: String) -> Self {
        match self {
            UserSymbol::Info(s) => UserSymbol::Info(UserSymbolInfo {
                name_end: Some(name_end),
                ..s
            }),
            UserSymbol::Ignored(..) => panic!("no"),
        }
    }
    pub fn with_size(self, size: Size) -> Self {
        match self {
            UserSymbol::Info(s) => UserSymbol::Info(UserSymbolInfo {
                size: Some(size),
                ..s
            }),
            UserSymbol::Ignored(vram, _) => UserSymbol::Ignored(vram, Some(size)),
        }
    }
    pub fn with_typ(self, typ: SymbolType) -> Self {
        match self {
            UserSymbol::Info(s) => UserSymbol::Info(UserSymbolInfo {
                typ: Some(typ),
                ..s
            }),
            UserSymbol::Ignored(..) => panic!("no"),
        }
    }
    pub fn with_migration_behavior(self, migration_behavior: RodataMigrationBehavior) -> Self {
        match self {
            UserSymbol::Info(s) => UserSymbol::Info(UserSymbolInfo {
                migration_behavior,
                ..s
            }),
            UserSymbol::Ignored(..) => panic!("no"),
        }
    }
    pub fn with_dont_allow_addend(self) -> Self {
        match self {
            UserSymbol::Info(s) => UserSymbol::Info(UserSymbolInfo {
                dont_allow_addend: true,
                ..s
            }),
            UserSymbol::Ignored(..) => panic!("no"),
        }
    }
}

pub fn create_drmario64_us_segments() -> Vec<TestSegment> {
    vec![
        TestSegment::Info(TestSegmentInfo {
            name: "header",
            rom: Rom::new(0x000000),
            vram: Vram::new(0x80000000),
            noload_size: None,
            sections: vec![TestSection::Bin(Rom::new(0x000000), "rom_header")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "ipl3",
            rom: Rom::new(0x000040),
            vram: Vram::new(0x80000040),
            noload_size: None,
            sections: vec![TestSection::Bin(Rom::new(0x000040), "ipl3")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "entry",
            rom: Rom::new(0x001000),
            vram: Vram::new(0x80000400),
            noload_size: None,
            sections: vec![TestSection::Text(Rom::new(0x001000), "entry")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "boot",
            rom: Rom::new(0x001060),
            vram: Vram::new(0x80000460),
            noload_size: Some(Size::new(0x18DE0)),
            sections: vec![
                TestSection::Text(Rom::new(0x001060), "boot/boot_main"),
                TestSection::Text(Rom::new(0x001220), "boot/dmadata"),
                TestSection::Text(Rom::new(0x001320), "gzip/inflate"),
                TestSection::Text(Rom::new(0x002B90), "gzip/unzip"),
                TestSection::Text(Rom::new(0x002F80), "boot/util"),
                TestSection::Text(Rom::new(0x0030A0), "../lib/ultralib/src/io/conteepprobe"),
                TestSection::Text(
                    Rom::new(0x003120),
                    "../lib/ultralib/src/io/conteeplongwrite",
                ),
                TestSection::Text(Rom::new(0x003210), "../lib/ultralib/src/io/conteeplongread"),
                TestSection::Text(Rom::new(0x003280), "../lib/ultralib/src/io/pimgr"),
                TestSection::Text(Rom::new(0x003410), "../lib/ultralib/src/io/epirawdma"),
                TestSection::Text(Rom::new(0x003600), "../lib/ultralib/src/io/epidma"),
                TestSection::Text(Rom::new(0x0036A0), "../lib/ultralib/src/io/cartrominit"),
                TestSection::Text(Rom::new(0x003820), "../lib/ultralib/src/io/devmgr"),
                TestSection::Text(Rom::new(0x003BE0), "../lib/ultralib/src/io/piacs"),
                TestSection::Text(Rom::new(0x003CD0), "../lib/ultralib/src/io/pidma"),
                TestSection::Text(Rom::new(0x003D80), "../lib/ultralib/src/io/aigetlen"),
                TestSection::Text(Rom::new(0x003D90), "../lib/ultralib/src/io/aigetstat"),
                TestSection::Text(Rom::new(0x003DA0), "../lib/ultralib/src/io/aisetfreq"),
                TestSection::Text(Rom::new(0x003EC0), "../lib/ultralib/src/io/aisetnextbuf"),
                TestSection::Text(Rom::new(0x003F60), "../lib/ultralib/src/audio/env"),
                TestSection::Text(Rom::new(0x004C10), "../lib/ultralib/src/audio/sl"),
                TestSection::Text(Rom::new(0x004CC0), "../lib/ultralib/src/audio/heapinit"),
                TestSection::Text(Rom::new(0x004D00), "../lib/ultralib/src/audio/synthesizer"),
                TestSection::Text(Rom::new(0x005340), "../lib/ultralib/src/audio/syndelete"),
                TestSection::Text(Rom::new(0x005350), "../lib/ultralib/src/audio/synaddplayer"),
                TestSection::Text(
                    Rom::new(0x0053A0),
                    "../lib/ultralib/src/audio/synallocvoice",
                ),
                TestSection::Text(Rom::new(0x005580), "../lib/ultralib/src/audio/synstopvoice"),
                TestSection::Text(
                    Rom::new(0x005600),
                    "../lib/ultralib/src/audio/synstartvoice",
                ),
                TestSection::Text(Rom::new(0x0056A0), "../lib/ultralib/src/audio/synsetpitch"),
                TestSection::Text(Rom::new(0x005730), "../lib/ultralib/src/audio/synsetvol"),
                TestSection::Text(Rom::new(0x0057F0), "../lib/ultralib/src/audio/synsetfxmix"),
                TestSection::Text(Rom::new(0x005890), "../lib/ultralib/src/audio/synsetpan"),
                TestSection::Text(Rom::new(0x005920), "../lib/ultralib/src/audio/synallocfx"),
                TestSection::Text(Rom::new(0x0059C0), "../lib/ultralib/src/os/invaldcache"),
                TestSection::Text(
                    Rom::new(0x005A70),
                    "../lib/ultralib/src/os/writebackdcacheall",
                ),
                TestSection::Text(Rom::new(0x005AA0), "../lib/ultralib/src/io/contquery"),
                TestSection::Text(Rom::new(0x005B40), "../lib/ultralib/src/io/contreaddata"),
                TestSection::Text(Rom::new(0x005D20), "../lib/ultralib/src/io/controller"),
                TestSection::Text(Rom::new(0x006030), "../lib/ultralib/src/io/contsetch"),
                TestSection::Text(
                    Rom::new(0x006090),
                    "../lib/ultralib/src/os/virtualtophysical",
                ),
                TestSection::Text(Rom::new(0x0060F0), "../lib/ultralib/src/gu/sqrtf"),
                TestSection::Text(Rom::new(0x006100), "../lib/ultralib/src/gu/cosf"),
                TestSection::Text(Rom::new(0x006250), "../lib/ultralib/src/gu/ortho"),
                TestSection::Text(Rom::new(0x0064D0), "../lib/ultralib/src/gu/perspective"),
                TestSection::Text(Rom::new(0x006850), "../lib/ultralib/src/gu/rotaterpy"),
                TestSection::Text(Rom::new(0x006B70), "../lib/ultralib/src/gu/sinf"),
                TestSection::Text(Rom::new(0x006D10), "../lib/ultralib/src/gu/sins"),
                TestSection::Text(Rom::new(0x006D70), "../lib/ultralib/src/libc/bcmp"),
                TestSection::Text(Rom::new(0x006E80), "../lib/ultralib/src/libc/bcopy"),
                TestSection::Text(Rom::new(0x0071A0), "../lib/ultralib/src/libc/bzero"),
                TestSection::Text(Rom::new(0x007240), "../lib/ultralib/src/libc/string"),
                TestSection::Text(Rom::new(0x0072D0), "../lib/ultralib/src/os/createmesgqueue"),
                TestSection::Text(Rom::new(0x007300), "../lib/ultralib/src/os/jammesg"),
                TestSection::Text(Rom::new(0x007440), "../lib/ultralib/src/os/recvmesg"),
                TestSection::Text(Rom::new(0x007570), "../lib/ultralib/src/os/sendmesg"),
                TestSection::Text(Rom::new(0x0076A0), "../lib/ultralib/src/os/seteventmesg"),
                TestSection::Text(Rom::new(0x007750), "../lib/ultralib/src/io/sptask"),
                TestSection::Text(Rom::new(0x007990), "../lib/ultralib/src/io/sptaskyield"),
                TestSection::Text(Rom::new(0x0079B0), "../lib/ultralib/src/io/sptaskyielded"),
                TestSection::Text(Rom::new(0x007A00), "../lib/ultralib/src/sched/sched"),
                TestSection::Text(Rom::new(0x008410), "../lib/ultralib/src/io/sirawdma"),
                TestSection::Text(Rom::new(0x0084C0), "../lib/ultralib/src/io/siacs"),
                TestSection::Text(Rom::new(0x0085B0), "../lib/ultralib/src/os/createthread"),
                TestSection::Text(Rom::new(0x008680), "../lib/ultralib/src/os/getthreadpri"),
                TestSection::Text(Rom::new(0x0086A0), "../lib/ultralib/src/os/setthreadpri"),
                TestSection::Text(Rom::new(0x008770), "../lib/ultralib/src/os/startthread"),
                TestSection::Text(Rom::new(0x008890), "../lib/ultralib/src/os/stopthread"),
                TestSection::Text(Rom::new(0x008950), "../lib/ultralib/src/os/thread"),
                TestSection::Text(Rom::new(0x008990), "../lib/ultralib/src/os/yieldthread"),
                TestSection::Text(Rom::new(0x0089E0), "../lib/ultralib/src/os/gettime"),
                TestSection::Text(Rom::new(0x008A70), "../lib/ultralib/src/os/settime"),
                TestSection::Text(Rom::new(0x008A90), "../lib/ultralib/src/os/settimer"),
                TestSection::Text(Rom::new(0x008BC0), "../lib/ultralib/src/os/timerintr"),
                TestSection::Text(Rom::new(0x008EE0), "../lib/ultralib/src/os/probetlb"),
                TestSection::Text(
                    Rom::new(0x008FA0),
                    "../lib/ultralib/src/io/vigetcurrframebuf",
                ),
                TestSection::Text(
                    Rom::new(0x008FE0),
                    "../lib/ultralib/src/io/vigetnextframebuf",
                ),
                TestSection::Text(Rom::new(0x009020), "../lib/ultralib/src/io/vimgr"),
                TestSection::Text(Rom::new(0x009360), "../lib/ultralib/src/io/visetevent"),
                TestSection::Text(Rom::new(0x0093C0), "../lib/ultralib/src/io/visetmode"),
                TestSection::Text(Rom::new(0x009410), "../lib/ultralib/src/io/visetspecial"),
                TestSection::Text(Rom::new(0x009580), "../lib/ultralib/src/io/visetyscale"),
                TestSection::Text(Rom::new(0x0095D0), "../lib/ultralib/src/io/viswapbuf"),
                TestSection::Text(Rom::new(0x009620), "../lib/ultralib/src/io/viswapcontext"),
                TestSection::Text(Rom::new(0x009930), "../lib/ultralib/src/io/viblack"),
                TestSection::Text(Rom::new(0x0099A0), "../lib/ultralib/src/mgu/mtxident"),
                TestSection::Text(Rom::new(0x009A00), "../lib/ultralib/src/mgu/mtxidentf"),
                TestSection::Text(Rom::new(0x009A60), "../lib/ultralib/src/mgu/mtxf2l"),
                TestSection::Text(Rom::new(0x009AE0), "../lib/ultralib/src/mgu/mtxl2f"),
                TestSection::Text(Rom::new(0x009B60), "../lib/ultralib/src/mgu/mtxcatf"),
                TestSection::Text(Rom::new(0x009C60), "../lib/ultralib/src/mgu/scale"),
                TestSection::Text(Rom::new(0x009D20), "../lib/ultralib/src/mgu/scalef"),
                TestSection::Text(Rom::new(0x009D80), "../lib/ultralib/src/mgu/translate"),
                TestSection::Text(Rom::new(0x009E60), "../lib/ultralib/src/mgu/translatef"),
                TestSection::Text(Rom::new(0x009EC0), "../lib/ultralib/src/os/initialize"),
                TestSection::Text(Rom::new(0x00A260), "../lib/ultralib/src/io/conteepread"),
                TestSection::Text(Rom::new(0x00A480), "../lib/ultralib/src/io/conteepwrite"),
                TestSection::Text(Rom::new(0x00A830), "../lib/ultralib/src/os/exceptasm"),
                TestSection::Text(Rom::new(0x00B1A0), "../lib/ultralib/src/os/interrupt"),
                TestSection::Text(Rom::new(0x00B230), "../lib/ultralib/src/os/setintmask"),
                TestSection::Text(
                    Rom::new(0x00B2D0),
                    "../lib/ultralib/src/os/setglobalintmask",
                ),
                TestSection::Text(
                    Rom::new(0x00B310),
                    "../lib/ultralib/src/os/resetglobalintmask",
                ),
                TestSection::Text(Rom::new(0x00B360), "../lib/ultralib/src/io/pirawdma"),
                TestSection::Text(Rom::new(0x00B430), "../lib/ultralib/src/io/pigetcmdq"),
                TestSection::Text(Rom::new(0x00B450), "../lib/ultralib/src/io/epirawread"),
                TestSection::Text(Rom::new(0x00B5C0), "../lib/ultralib/src/io/epirawwrite"),
                TestSection::Text(Rom::new(0x00B730), "../lib/ultralib/src/io/ai"),
                TestSection::Text(Rom::new(0x00B750), "../lib/ultralib/src/audio/drvrnew"),
                TestSection::Text(Rom::new(0x00BF80), "../lib/ultralib/src/audio/load"),
                TestSection::Text(Rom::new(0x00CA90), "../lib/ultralib/src/audio/auxbus"),
                TestSection::Text(Rom::new(0x00CBA0), "../lib/ultralib/src/audio/filter"),
                TestSection::Text(Rom::new(0x00CBC0), "../lib/ultralib/src/audio/mainbus"),
                TestSection::Text(Rom::new(0x00CD30), "../lib/ultralib/src/audio/resample"),
                TestSection::Text(Rom::new(0x00CFE0), "../lib/ultralib/src/audio/reverb"),
                TestSection::Text(Rom::new(0x00DB80), "../lib/ultralib/src/audio/save"),
                TestSection::Text(Rom::new(0x00DC50), "../lib/ultralib/src/audio/heapalloc"),
                TestSection::Text(Rom::new(0x00DCA0), "../lib/ultralib/src/audio/copy"),
                TestSection::Text(Rom::new(0x00DCE0), "../lib/ultralib/src/os/invalicache"),
                TestSection::Text(Rom::new(0x00DD60), "../lib/ultralib/src/os/writebackdcache"),
                TestSection::Text(Rom::new(0x00DDE0), "../lib/ultralib/src/io/dpsetnextbuf"),
                TestSection::Text(Rom::new(0x00DE80), "../lib/ultralib/src/os/getcause"),
                TestSection::Text(Rom::new(0x00DE90), "../lib/ultralib/src/os/getcount"),
                TestSection::Text(Rom::new(0x00DEA0), "../lib/ultralib/src/os/getsr"),
                TestSection::Text(Rom::new(0x00DEB0), "../lib/ultralib/src/os/setcompare"),
                TestSection::Text(Rom::new(0x00DEC0), "../lib/ultralib/src/os/setfpccsr"),
                TestSection::Text(Rom::new(0x00DED0), "../lib/ultralib/src/os/setsr"),
                TestSection::Text(Rom::new(0x00DEE0), "../lib/ultralib/src/os/setwatchlo"),
                TestSection::Text(Rom::new(0x00DEF0), "../lib/ultralib/src/io/sp"),
                TestSection::Text(Rom::new(0x00DF10), "../lib/ultralib/src/io/spgetstat"),
                TestSection::Text(Rom::new(0x00DF20), "../lib/ultralib/src/io/spsetstat"),
                TestSection::Text(Rom::new(0x00DF30), "../lib/ultralib/src/io/spsetpc"),
                TestSection::Text(Rom::new(0x00DF60), "../lib/ultralib/src/io/sprawdma"),
                TestSection::Text(Rom::new(0x00DFF0), "../lib/ultralib/src/io/sirawread"),
                TestSection::Text(Rom::new(0x00E040), "../lib/ultralib/src/io/sirawwrite"),
                TestSection::Text(Rom::new(0x00E090), "../lib/ultralib/src/os/destroythread"),
                TestSection::Text(Rom::new(0x00E170), "../lib/ultralib/src/os/maptlbrdb"),
                TestSection::Text(Rom::new(0x00E1D0), "../lib/ultralib/src/os/unmaptlball"),
                TestSection::Text(Rom::new(0x00E220), "../lib/ultralib/src/io/vi"),
                TestSection::Text(
                    Rom::new(0x00E330),
                    "../lib/ultralib/src/io/vigetcurrcontext",
                ),
                TestSection::Text(Rom::new(0x00E340), "../lib/ultralib/src/os/parameters"),
                TestSection::Text(Rom::new(0x00E3A0), "../lib/ultralib/src/io/dp"),
                TestSection::Text(Rom::new(0x00E3C0), "../lib/ultralib/src/io/si"),
                TestSection::Text(Rom::new(0x00E3E0), "libkmc/fmod"),
                TestSection::Text(Rom::new(0x00E5C0), "libkmc/memmove"),
                TestSection::Text(Rom::new(0x00E730), "libkmc/memset"),
                TestSection::Text(Rom::new(0x00E800), "libkmc/modf"),
                TestSection::Text(Rom::new(0x00E8B0), "libkmc/rand"),
                TestSection::Text(Rom::new(0x00E8F0), "libkmc/strcpy"),
                TestSection::Text(Rom::new(0x00E9C0), "libkmc/mmuldi3"),
                TestSection::Data(Rom::new(0x00ED90), "boot/boot_main"),
                TestSection::Data(Rom::new(0x00EDA0), "boot/dmadata"),
                TestSection::Data(Rom::new(0x00EDB0), "gzip/inflate"),
                TestSection::Data(Rom::new(0x00EF30), "gzip/unzip"),
                TestSection::Data(Rom::new(0x00F340), "boot/rom_offsets"),
                TestSection::Data(Rom::new(0x00F650), "../lib/ultralib/src/io/pimgr"),
                TestSection::Data(Rom::new(0x00F680), "../lib/ultralib/src/io/cartrominit"),
                TestSection::Data(Rom::new(0x00F690), "../lib/ultralib/src/io/piacs"),
                TestSection::Data(Rom::new(0x00F6A0), "../lib/ultralib/src/io/aisetnextbuf"),
                TestSection::Data(Rom::new(0x00F6B0), "../lib/ultralib/src/audio/env"),
                TestSection::Data(Rom::new(0x00F7B0), "../lib/ultralib/src/audio/sl"),
                TestSection::Data(Rom::new(0x00F7C0), "../lib/ultralib/src/io/controller"),
                TestSection::Data(Rom::new(0x00F7D0), "../lib/ultralib/src/gu/rotaterpy"),
                TestSection::Data(Rom::new(0x00F7E0), "../lib/ultralib/src/gu/sins"),
                TestSection::Data(Rom::new(0x00FFE0), "../lib/ultralib/src/os/seteventmesg"),
                TestSection::Data(Rom::new(0x00FFF0), "../lib/ultralib/src/sched/sched"),
                TestSection::Data(Rom::new(0x010000), "../lib/ultralib/src/io/siacs"),
                TestSection::Data(Rom::new(0x010010), "../lib/ultralib/src/os/thread"),
                TestSection::Data(Rom::new(0x010030), "../lib/ultralib/src/os/timerintr"),
                TestSection::Data(Rom::new(0x010040), "../lib/ultralib/src/io/vimgr"),
                TestSection::Data(Rom::new(0x010060), "../lib/ultralib/src/io/vitbl"),
                TestSection::Data(Rom::new(0x0111E0), "../lib/ultralib/src/os/initialize"),
                TestSection::Data(Rom::new(0x011200), "../lib/ultralib/src/os/exceptasm"),
                TestSection::Data(Rom::new(0x011230), "../lib/ultralib/src/audio/drvrnew"),
                TestSection::Data(Rom::new(0x0113C0), "../lib/ultralib/src/audio/reverb"),
                TestSection::Data(Rom::new(0x0113E0), "../lib/ultralib/src/io/vi"),
                TestSection::Data(
                    Rom::new(0x011450),
                    "../lib/ultralib/src/vimodes/vimodentsclan1",
                ),
                TestSection::Data(
                    Rom::new(0x0114A0),
                    "../lib/ultralib/src/vimodes/vimodepallan1",
                ),
                TestSection::Data(
                    Rom::new(0x0114F0),
                    "../lib/ultralib/src/vimodes/vimodempallan1",
                ),
                TestSection::Data(Rom::new(0x011540), "libkmc/ctype"),
                TestSection::Rodata(Rom::new(0x011640), "../lib/ultralib/src/io/devmgr"),
                TestSection::Rodata(Rom::new(0x011660), "../lib/ultralib/src/io/aisetfreq"),
                TestSection::Rodata(Rom::new(0x011670), "../lib/ultralib/src/audio/env"),
                TestSection::Rodata(Rom::new(0x011760), "../lib/ultralib/src/audio/synthesizer"),
                TestSection::Rodata(Rom::new(0x011780), "../lib/ultralib/src/gu/cosf"),
                TestSection::Rodata(Rom::new(0x0117E0), "../lib/ultralib/src/gu/perspective"),
                TestSection::Rodata(Rom::new(0x011820), "../lib/ultralib/src/gu/sinf"),
                TestSection::Rodata(Rom::new(0x011880), "../lib/ultralib/src/sched/sched"),
                TestSection::Rodata(Rom::new(0x0118A0), "../lib/ultralib/src/io/viswapcontext"),
                TestSection::Rodata(Rom::new(0x0118B0), "../lib/ultralib/src/os/exceptasm"),
                TestSection::Rodata(Rom::new(0x011900), "../lib/ultralib/src/os/setintmask"),
                TestSection::Rodata(Rom::new(0x011980), "../lib/ultralib/src/audio/drvrnew"),
                TestSection::Rodata(Rom::new(0x0119C0), "../lib/ultralib/src/audio/resample"),
                TestSection::Rodata(Rom::new(0x0119F0), "../lib/ultralib/src/audio/reverb"),
                TestSection::Rodata(Rom::new(0x011A40), "../lib/ultralib/src/gu/libm_vals"),
                TestSection::Rodata(Rom::new(0x011A50), "libkmc/fmod"),
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
            rom: Rom::new(0x011A60),
            vram: Vram::new(0x80029C40),
            noload_size: None,
            sections: vec![TestSection::Text(Rom::new(0x011A60), "dma_table")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "main_segment",
            rom: Rom::new(0x011A70),
            vram: Vram::new(0x80029C50),
            noload_size: Some(Size::new(0x70FD0)),
            sections: vec![
                TestSection::Text(Rom::new(0x011A70), "main_segment/main"),
                TestSection::Text(Rom::new(0x011CF0), "main_segment/nnsched"),
                TestSection::Text(Rom::new(0x0123D0), "main_segment/joy"),
                TestSection::Text(Rom::new(0x0128A0), "main_segment/audio/sound"),
                TestSection::Text(Rom::new(0x0132B0), "main_segment/graphic"),
                TestSection::Text(Rom::new(0x013B40), "main_segment/graphics/static"),
                TestSection::Text(Rom::new(0x014F90), "main_segment/audio/music_driver"),
                TestSection::Text(Rom::new(0x015940), "main_segment/main1x"),
                TestSection::Text(Rom::new(0x016650), "main_segment/vr_init"),
                TestSection::Text(Rom::new(0x016920), "main_segment/aiset"),
                TestSection::Text(Rom::new(0x01E3D0), "main_segment/record"),
                TestSection::Text(Rom::new(0x020D10), "main_segment/unused/020D10"),
                TestSection::Text(Rom::new(0x0227C0), "main_segment/game_etc"),
                TestSection::Text(Rom::new(0x024BC0), "main_segment/screen_print/printer"),
                TestSection::Text(Rom::new(0x024C40), "main_segment/screen_print/printf_impl"),
                TestSection::Text(Rom::new(0x026000), "main_segment/screen_print/debug_print"),
                TestSection::Text(Rom::new(0x026550), "main_segment/debug_menu"),
                TestSection::Text(Rom::new(0x028820), "main_segment/bg_tasks"),
                TestSection::Text(Rom::new(0x0289D0), "main_segment/tex_func"),
                TestSection::Text(Rom::new(0x02DBF0), "main_segment/main_menu"),
                TestSection::Text(Rom::new(0x043610), "main_segment/font"),
                TestSection::Text(Rom::new(0x044D40), "main_segment/msgwnd"),
                TestSection::Text(Rom::new(0x045F40), "main_segment/char_anime"),
                TestSection::Text(Rom::new(0x046D70), "main_segment/dm_virus_init"),
                TestSection::Text(Rom::new(0x048010), "main_segment/dm_game_main"),
                TestSection::Text(Rom::new(0x059D10), "main_segment/dm_manual_main"),
                TestSection::Text(Rom::new(0x05DD50), "main_segment/dm_title_main"),
                TestSection::Text(Rom::new(0x05E700), "main_segment/main_story"),
                TestSection::Text(Rom::new(0x063470), "main_segment/lws"),
                TestSection::Text(Rom::new(0x063A40), "main_segment/calc"),
                TestSection::Text(Rom::new(0x065080), "main_segment/unused/065080"),
                TestSection::Text(Rom::new(0x066080), "main_segment/unused/066080"),
                TestSection::Text(Rom::new(0x066100), "main_segment/replay"),
                TestSection::Text(Rom::new(0x066580), "main_segment/066580"),
                TestSection::Text(Rom::new(0x066840), "main_segment/066840"),
                TestSection::Text(Rom::new(0x0676B0), "main_segment/unused/055C40"),
                TestSection::Text(Rom::new(0x067820), "../lib/libmus/src/player"),
                TestSection::Text(Rom::new(0x06B100), "../lib/libmus/src/player_fx"),
                TestSection::Text(Rom::new(0x06B9E0), "../lib/libmus/src/aud_dma"),
                TestSection::Text(Rom::new(0x06BE40), "../lib/libmus/src/aud_sched"),
                TestSection::Text(Rom::new(0x06BFF0), "../lib/libmus/src/aud_thread"),
                TestSection::Text(Rom::new(0x06C2B0), "../lib/libmus/src/lib_memory"),
                TestSection::Text(Rom::new(0x06C3F0), "../lib/libmus/src/aud_samples"),
                TestSection::Bin(Rom::new(0x06C4B0), "rsp/rspboot"),
                TestSection::Bin(Rom::new(0x06C580), "rsp/aspMain"),
                TestSection::Bin(Rom::new(0x06D3A0), "rsp/f3dex2"),
                TestSection::Bin(Rom::new(0x06E730), "rsp/s2dex"),
                TestSection::Data(Rom::new(0x06FF20), "main_segment/nnsched"),
                TestSection::Data(Rom::new(0x06FF30), "main_segment/graphic"),
                TestSection::Data(Rom::new(0x06FF70), "main_segment/graphics/static"),
                TestSection::Data(Rom::new(0x070210), "main_segment/audio/music_driver"),
                TestSection::Data(Rom::new(0x070220), "main_segment/main1x"),
                TestSection::Data(Rom::new(0x0702A0), "main_segment/vr_init"),
                TestSection::Data(Rom::new(0x0702B0), "main_segment/aiset"),
                TestSection::Data(Rom::new(0x0715C0), "main_segment/record"),
                TestSection::Data(Rom::new(0x074DC0), "main_segment/unused/020D10"),
                TestSection::Data(Rom::new(0x074E10), "main_segment/game_etc"),
                TestSection::Data(Rom::new(0x075000), "main_segment/screen_print/printf_impl"),
                TestSection::Data(Rom::new(0x0750C0), "main_segment/screen_print/debug_print"),
                TestSection::Data(Rom::new(0x076190), "main_segment/debug_menu"),
                TestSection::Data(Rom::new(0x0763F0), "main_segment/bg_tasks"),
                TestSection::Data(Rom::new(0x076400), "main_segment/tex_func"),
                TestSection::Data(Rom::new(0x076590), "main_segment/main_menu"),
                TestSection::Data(Rom::new(0x0766E0), "main_segment/font"),
                TestSection::Data(Rom::new(0x08EDA0), "main_segment/dm_game_main"),
                TestSection::Data(Rom::new(0x08F1B0), "main_segment/dm_title_main"),
                TestSection::Data(Rom::new(0x08F1C0), "main_segment/main_story"),
                TestSection::Data(Rom::new(0x092C80), "main_segment/calc"),
                TestSection::Data(Rom::new(0x092F90), "main_segment/066580"),
                TestSection::Data(Rom::new(0x093140), "main_segment/066840"),
                TestSection::Data(Rom::new(0x093A70), "../lib/libmus/src/player"),
                TestSection::Data(Rom::new(0x093B30), "../lib/libmus/src/player_fx"),
                TestSection::Data(Rom::new(0x093D40), "../lib/libmus/src/aud_sched"),
                TestSection::Data(Rom::new(0x093D50), "../lib/libmus/src/aud_thread"),
                TestSection::Bin(Rom::new(0x093D60), "rsp/aspMain"),
                TestSection::Bin(Rom::new(0x094020), "rsp/f3dex2"),
                TestSection::Bin(Rom::new(0x094440), "rsp/s2dex"),
                TestSection::Rodata(Rom::new(0x094800), "main_segment/main"),
                TestSection::Rodata(Rom::new(0x094830), "main_segment/audio/sound"),
                TestSection::Rodata(Rom::new(0x094B20), "main_segment/graphic"),
                TestSection::Rodata(Rom::new(0x094B40), "main_segment/main1x"),
                TestSection::Rodata(Rom::new(0x094CD0), "main_segment/aiset"),
                TestSection::Rodata(Rom::new(0x094D60), "main_segment/record"),
                TestSection::Rodata(Rom::new(0x094DD0), "main_segment/unused/020D10"),
                TestSection::Rodata(Rom::new(0x094F70), "main_segment/game_etc"),
                TestSection::Rodata(Rom::new(0x094FB0), "main_segment/screen_print/printf_impl"),
                TestSection::Rodata(Rom::new(0x095100), "main_segment/debug_menu"),
                TestSection::Rodata(Rom::new(0x095960), "main_segment/tex_func"),
                TestSection::Rodata(Rom::new(0x095A30), "main_segment/main_menu"),
                TestSection::Rodata(Rom::new(0x099770), "main_segment/font"),
                TestSection::Rodata(Rom::new(0x0997B0), "main_segment/msgwnd"),
                TestSection::Rodata(Rom::new(0x099880), "main_segment/char_anime"),
                TestSection::Rodata(Rom::new(0x0999C0), "main_segment/dm_virus_init"),
                TestSection::Rodata(Rom::new(0x099A50), "main_segment/dm_game_main"),
                TestSection::Rodata(Rom::new(0x09A220), "main_segment/dm_manual_main"),
                TestSection::Rodata(Rom::new(0x09B060), "main_segment/dm_title_main"),
                TestSection::Rodata(Rom::new(0x09B180), "main_segment/main_story"),
                TestSection::Rodata(Rom::new(0x09B270), "main_segment/lws"),
                TestSection::Rodata(Rom::new(0x09B2C0), "main_segment/calc"),
                TestSection::Rodata(Rom::new(0x09B310), "main_segment/unused/065080"),
                TestSection::Rodata(Rom::new(0x09B370), "../lib/libmus/src/player"),
                TestSection::Rodata(Rom::new(0x09B450), "../lib/libmus/src/player_fx"),
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
            rom: Rom::new(0x09B460),
            vram: Vram::new(0x80124610),
            noload_size: Some(Size::new(0xE09F0)),
            sections: vec![TestSection::Bss(Vram::new(0x80124610), "buffers/buffer1")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "buffer2",
            rom: Rom::new(0x09B460),
            vram: Vram::new(0x80205000),
            noload_size: Some(Size::new(0x1B0000)),
            sections: vec![TestSection::Bss(Vram::new(0x80205000), "buffers/buffer2")],
        }),
        TestSegment::Info(TestSegmentInfo {
            name: "buffer1",
            rom: Rom::new(0x09B460),
            vram: Vram::new(0x803B5000),
            noload_size: Some(Size::new(0x4B000)),
            sections: vec![TestSection::Bss(
                Vram::new(0x803B5000),
                "buffers/framebuffer",
            )],
        }),
        TestSegment::EndMarker(Rom::new(0x09B460)),
    ]
}

pub fn create_drmario64_us_symbols() -> Vec<UserSymbol> {
    vec![
        UserSymbol::new("entrypoint".into(), Vram::new(0x80000400)),
        UserSymbol::new("Idle_Nop".into(), Vram::new(0x80000460)),
        UserSymbol::new("Main_StopThread".into(), Vram::new(0x80000468)),
        UserSymbol::new("func_80000488".into(), Vram::new(0x80000488)),
        UserSymbol::new("bootproc".into(), Vram::new(0x80000494)),
        UserSymbol::new("Main_ThreadEntry".into(), Vram::new(0x800004F0)),
        UserSymbol::new("Idle_ThreadEntry".into(), Vram::new(0x80000580)),
        UserSymbol::new("DmaData_RomToRam".into(), Vram::new(0x80000620)),
        UserSymbol::new("func_80000720".into(), Vram::new(0x80000720)),
        UserSymbol::new("func_8000075C".into(), Vram::new(0x8000075C)),
        UserSymbol::new("huft_build".into(), Vram::new(0x80000778)),
        UserSymbol::new("huft_free".into(), Vram::new(0x80000D0C)),
        UserSymbol::new("inflate_codes".into(), Vram::new(0x80000D48)),
        UserSymbol::new("inflate_stored".into(), Vram::new(0x80001260)),
        UserSymbol::new("inflate_fixed".into(), Vram::new(0x80001480)),
        UserSymbol::new("inflate_dynamic".into(), Vram::new(0x80001620)),
        UserSymbol::new("inflate_block".into(), Vram::new(0x80001D4C)),
        UserSymbol::new("inflate".into(), Vram::new(0x80001EB4)),
        UserSymbol::new("expand_gzip".into(), Vram::new(0x80001F90)),
        UserSymbol::new("auRomDataRead".into(), Vram::new(0x80001FD8)),
        UserSymbol::new("data_write".into(), Vram::new(0x80002064)),
        UserSymbol::new("unzip".into(), Vram::new(0x800020A0)),
        UserSymbol::new("updcrc".into(), Vram::new(0x80002148)),
        UserSymbol::new("clear_bufs".into(), Vram::new(0x800021A0)),
        UserSymbol::new("fill_inbuf".into(), Vram::new(0x800021CC)),
        UserSymbol::new("func_800022A8".into(), Vram::new(0x800022A8)),
        UserSymbol::new("flush_window".into(), Vram::new(0x80002300)),
        UserSymbol::new("DecompressRomToRam".into(), Vram::new(0x80002380)),
        UserSymbol::new("WrapI".into(), Vram::new(0x800023B4)),
        UserSymbol::new("WrapF".into(), Vram::new(0x80002400)),
        UserSymbol::new("osEepromProbe".into(), Vram::new(0x800024A0)).with_size(Size::new(0x74)),
        UserSymbol::new("osEepromLongWrite".into(), Vram::new(0x80002520))
            .with_size(Size::new(0xE4)),
        UserSymbol::new("osEepromLongRead".into(), Vram::new(0x80002610))
            .with_size(Size::new(0x70)),
        UserSymbol::new("osCreatePiManager".into(), Vram::new(0x80002680))
            .with_size(Size::new(0x190)),
        UserSymbol::new("__osEPiRawStartDma".into(), Vram::new(0x80002810))
            .with_size(Size::new(0x1EC)),
        UserSymbol::new("osEPiStartDma".into(), Vram::new(0x80002A00)).with_size(Size::new(0x94)),
        UserSymbol::new("osCartRomInit".into(), Vram::new(0x80002AA0)).with_size(Size::new(0x178)),
        UserSymbol::new("__osDevMgrMain".into(), Vram::new(0x80002C20)).with_size(Size::new(0x3B8)),
        UserSymbol::new("__osPiCreateAccessQueue".into(), Vram::new(0x80002FE0)),
        UserSymbol::new("__osPiGetAccess".into(), Vram::new(0x80003034)),
        UserSymbol::new("__osPiRelAccess".into(), Vram::new(0x800030A0)),
        UserSymbol::new("osPiStartDma".into(), Vram::new(0x800030D0)).with_size(Size::new(0xA8)),
        UserSymbol::new("osAiGetLength".into(), Vram::new(0x80003180)).with_size(Size::new(0x10)),
        UserSymbol::new("osAiGetStatus".into(), Vram::new(0x80003190)).with_size(Size::new(0x10)),
        UserSymbol::new("osAiSetFrequency".into(), Vram::new(0x800031A0))
            .with_size(Size::new(0x118)),
        UserSymbol::new("osAiSetNextBuffer".into(), Vram::new(0x800032C0))
            .with_size(Size::new(0x94)),
        UserSymbol::new("alEnvmixerPull".into(), Vram::new(0x80003360)).with_size(Size::new(0x500)),
        UserSymbol::new("alEnvmixerParam".into(), Vram::new(0x80003860)).with_size(Size::new(0xD4)),
        UserSymbol::new("_pullSubFrame".into(), Vram::new(0x80003934)).with_size(Size::new(0x2D4)),
        UserSymbol::new("_frexpf".into(), Vram::new(0x80003C08)).with_size(Size::new(0xE0)),
        UserSymbol::new("_ldexpf".into(), Vram::new(0x80003CE8)).with_size(Size::new(0x24)),
        UserSymbol::new("_getRate".into(), Vram::new(0x80003D0C)).with_size(Size::new(0x280)),
        UserSymbol::new("_getVol".into(), Vram::new(0x80003F8C)).with_size(Size::new(0x84)),
        UserSymbol::new("alInit".into(), Vram::new(0x80004010)).with_size(Size::new(0x2C)),
        UserSymbol::new("alClose".into(), Vram::new(0x8000403C)).with_size(Size::new(0x30)),
        UserSymbol::new("alLink".into(), Vram::new(0x8000406C)).with_size(Size::new(0x20)),
        UserSymbol::new("alUnlink".into(), Vram::new(0x8000408C)).with_size(Size::new(0x30)),
        UserSymbol::new("alHeapInit".into(), Vram::new(0x800040C0)).with_size(Size::new(0x34)),
        UserSymbol::new("alSynNew".into(), Vram::new(0x80004100)).with_size(Size::new(0x2CC)),
        UserSymbol::new("alAudioFrame".into(), Vram::new(0x800043CC)).with_size(Size::new(0x1B0)),
        UserSymbol::new("__allocParam".into(), Vram::new(0x8000457C)).with_size(Size::new(0x2C)),
        UserSymbol::new("__freeParam".into(), Vram::new(0x800045A8)).with_size(Size::new(0x18)),
        UserSymbol::new("_collectPVoices".into(), Vram::new(0x800045C0)).with_size(Size::new(0x54)),
        UserSymbol::new("_freePVoice".into(), Vram::new(0x80004614)).with_size(Size::new(0x40)),
        UserSymbol::new("_timeToSamplesNoRound".into(), Vram::new(0x80004654))
            .with_size(Size::new(0x48)),
        UserSymbol::new("_timeToSamples".into(), Vram::new(0x8000469C)).with_size(Size::new(0x4C)),
        UserSymbol::new("__nextSampleTime".into(), Vram::new(0x800046E8))
            .with_size(Size::new(0x54)),
        UserSymbol::new("alSynDelete".into(), Vram::new(0x80004740)).with_size(Size::new(0x8)),
        UserSymbol::new("alSynAddPlayer".into(), Vram::new(0x80004750)).with_size(Size::new(0x50)),
        UserSymbol::new("alSynAllocVoice".into(), Vram::new(0x800047A0))
            .with_size(Size::new(0x128)),
        UserSymbol::new("_allocatePVoice".into(), Vram::new(0x800048C8)).with_size(Size::new(0xB8)),
        UserSymbol::new("alSynStopVoice".into(), Vram::new(0x80004980)).with_size(Size::new(0x7C)),
        UserSymbol::new("alSynStartVoice".into(), Vram::new(0x80004A00)).with_size(Size::new(0x94)),
        UserSymbol::new("alSynSetPitch".into(), Vram::new(0x80004AA0)).with_size(Size::new(0x90)),
        UserSymbol::new("alSynSetVol".into(), Vram::new(0x80004B30)).with_size(Size::new(0xB8)),
        UserSymbol::new("alSynSetFXMix".into(), Vram::new(0x80004BF0)).with_size(Size::new(0xA0)),
        UserSymbol::new("alSynSetPan".into(), Vram::new(0x80004C90)).with_size(Size::new(0x90)),
        UserSymbol::new("alSynAllocFX".into(), Vram::new(0x80004D20)).with_size(Size::new(0x94)),
        UserSymbol::new("osInvalDCache".into(), Vram::new(0x80004DC0)),
        UserSymbol::new("osWritebackDCacheAll".into(), Vram::new(0x80004E70)),
        UserSymbol::new("osContStartQuery".into(), Vram::new(0x80004EA0))
            .with_size(Size::new(0x80)),
        UserSymbol::new("osContGetQuery".into(), Vram::new(0x80004F20)).with_size(Size::new(0x20)),
        UserSymbol::new("osContStartReadData".into(), Vram::new(0x80004F40))
            .with_size(Size::new(0x88)),
        UserSymbol::new("osContGetReadData".into(), Vram::new(0x80004FC8))
            .with_size(Size::new(0x9C)),
        UserSymbol::new("__osPackReadData".into(), Vram::new(0x80005064))
            .with_size(Size::new(0xB8)),
        UserSymbol::new("osContInit".into(), Vram::new(0x80005120)).with_size(Size::new(0x19C)),
        UserSymbol::new("__osContGetInitData".into(), Vram::new(0x800052BC))
            .with_size(Size::new(0xB4)),
        UserSymbol::new("__osPackRequestData".into(), Vram::new(0x80005370))
            .with_size(Size::new(0xBC)),
        UserSymbol::new("osContSetCh".into(), Vram::new(0x80005430)).with_size(Size::new(0x60)),
        UserSymbol::new("osVirtualToPhysical".into(), Vram::new(0x80005490))
            .with_size(Size::new(0x54)),
        UserSymbol::new("sqrtf".into(), Vram::new(0x800054F0)),
        UserSymbol::new("cosf".into(), Vram::new(0x80005500)),
        UserSymbol::new("guOrthoF".into(), Vram::new(0x80005650)).with_size(Size::new(0x130)),
        UserSymbol::new("guOrtho".into(), Vram::new(0x80005780)).with_size(Size::new(0x14C)),
        UserSymbol::new("guPerspectiveF".into(), Vram::new(0x800058D0)).with_size(Size::new(0x1B0)),
        UserSymbol::new("guPerspective".into(), Vram::new(0x80005A80)).with_size(Size::new(0x1C8)),
        UserSymbol::new("guRotateRPYF".into(), Vram::new(0x80005C50)).with_size(Size::new(0x188)),
        UserSymbol::new("guRotateRPY".into(), Vram::new(0x80005DD8)).with_size(Size::new(0x18C)),
        UserSymbol::new("sinf".into(), Vram::new(0x80005F70)).with_size(Size::new(0x1A0)),
        UserSymbol::new("sins".into(), Vram::new(0x80006110)).with_size(Size::new(0x54)),
        UserSymbol::new("bcmp".into(), Vram::new(0x80006170)),
        UserSymbol::new("bcopy".into(), Vram::new(0x80006280)),
        UserSymbol::new("bzero".into(), Vram::new(0x800065A0)),
        UserSymbol::new("strchr".into(), Vram::new(0x80006640)).with_size(Size::new(0x40)),
        UserSymbol::new("strlen".into(), Vram::new(0x80006680)).with_size(Size::new(0x24)),
        UserSymbol::new("memcpy".into(), Vram::new(0x800066A4)).with_size(Size::new(0x28)),
        UserSymbol::new("osCreateMesgQueue".into(), Vram::new(0x800066D0))
            .with_size(Size::new(0x24)),
        UserSymbol::new("osJamMesg".into(), Vram::new(0x80006700)).with_size(Size::new(0x134)),
        UserSymbol::new("osRecvMesg".into(), Vram::new(0x80006840)).with_size(Size::new(0x124)),
        UserSymbol::new("osSendMesg".into(), Vram::new(0x80006970)).with_size(Size::new(0x130)),
        UserSymbol::new("osSetEventMesg".into(), Vram::new(0x80006AA0)).with_size(Size::new(0xAC)),
        UserSymbol::new("osSpTaskLoad".into(), Vram::new(0x80006B50)).with_size(Size::new(0x20C)),
        UserSymbol::new("osSpTaskStartGo".into(), Vram::new(0x80006D5C)).with_size(Size::new(0x2C)),
        UserSymbol::new("osSpTaskYield".into(), Vram::new(0x80006D90)).with_size(Size::new(0x1C)),
        UserSymbol::new("osSpTaskYielded".into(), Vram::new(0x80006DB0)).with_size(Size::new(0x4C)),
        UserSymbol::new("osCreateScheduler".into(), Vram::new(0x80006E00))
            .with_size(Size::new(0x148)),
        UserSymbol::new("osScAddClient".into(), Vram::new(0x80006F48)).with_size(Size::new(0x58)),
        UserSymbol::new("osScRemoveClient".into(), Vram::new(0x80006FA0))
            .with_size(Size::new(0x90)),
        UserSymbol::new("osScGetCmdQ".into(), Vram::new(0x80007030)).with_size(Size::new(0x8)),
        UserSymbol::new("__scMain".into(), Vram::new(0x80007038)).with_size(Size::new(0x104)),
        UserSymbol::new("__scHandleRetrace".into(), Vram::new(0x8000713C))
            .with_size(Size::new(0x100)),
        UserSymbol::new("__scHandleRSP".into(), Vram::new(0x8000723C)).with_size(Size::new(0xF0)),
        UserSymbol::new("__scHandleRDP".into(), Vram::new(0x8000732C)).with_size(Size::new(0x90)),
        UserSymbol::new("__scTaskReady".into(), Vram::new(0x800073BC)).with_size(Size::new(0x54)),
        UserSymbol::new("__scTaskComplete".into(), Vram::new(0x80007410))
            .with_size(Size::new(0x8C)),
        UserSymbol::new("__scAppendList".into(), Vram::new(0x8000749C)).with_size(Size::new(0x58)),
        UserSymbol::new("__scExec".into(), Vram::new(0x800074F4)).with_size(Size::new(0xD0)),
        UserSymbol::new("__scYield".into(), Vram::new(0x800075C4)).with_size(Size::new(0x38)),
        UserSymbol::new("__scSchedule".into(), Vram::new(0x800075FC)).with_size(Size::new(0x214)),
        UserSymbol::new("__osSiRawStartDma".into(), Vram::new(0x80007810))
            .with_size(Size::new(0xA4)),
        UserSymbol::new("__osSiCreateAccessQueue".into(), Vram::new(0x800078C0)),
        UserSymbol::new("__osSiGetAccess".into(), Vram::new(0x80007914)),
        UserSymbol::new("__osSiRelAccess".into(), Vram::new(0x80007980)),
        UserSymbol::new("osCreateThread".into(), Vram::new(0x800079B0)).with_size(Size::new(0xD0)),
        UserSymbol::new("osGetThreadPri".into(), Vram::new(0x80007A80)).with_size(Size::new(0x18)),
        UserSymbol::new("osSetThreadPri".into(), Vram::new(0x80007AA0)).with_size(Size::new(0xC8)),
        UserSymbol::new("osStartThread".into(), Vram::new(0x80007B70)).with_size(Size::new(0x118)),
        UserSymbol::new("osStopThread".into(), Vram::new(0x80007C90)).with_size(Size::new(0xB4)),
        UserSymbol::new("__osDequeueThread".into(), Vram::new(0x80007D50))
            .with_size(Size::new(0x34)),
        UserSymbol::new("osYieldThread".into(), Vram::new(0x80007D90)).with_size(Size::new(0x48)),
        UserSymbol::new("osGetTime".into(), Vram::new(0x80007DE0)).with_size(Size::new(0x84)),
        UserSymbol::new("osSetTime".into(), Vram::new(0x80007E70)).with_size(Size::new(0x14)),
        UserSymbol::new("osSetTimer".into(), Vram::new(0x80007E90)).with_size(Size::new(0x12C)),
        UserSymbol::new("__osTimerServicesInit".into(), Vram::new(0x80007FC0))
            .with_size(Size::new(0x54)),
        UserSymbol::new("__osTimerInterrupt".into(), Vram::new(0x80008014))
            .with_size(Size::new(0x13C)),
        UserSymbol::new("__osSetTimerIntr".into(), Vram::new(0x80008150))
            .with_size(Size::new(0x80)),
        UserSymbol::new("__osInsertTimer".into(), Vram::new(0x800081D0))
            .with_size(Size::new(0x108)),
        UserSymbol::new("__osProbeTLB".into(), Vram::new(0x800082E0)),
        UserSymbol::new("osViGetCurrentFramebuffer".into(), Vram::new(0x800083A0)),
        UserSymbol::new("osViGetNextFramebuffer".into(), Vram::new(0x800083E0)),
        UserSymbol::new("osCreateViManager".into(), Vram::new(0x80008420))
            .with_size(Size::new(0x1A0)),
        UserSymbol::new("viMgrMain".into(), Vram::new(0x800085C0)).with_size(Size::new(0x198)),
        UserSymbol::new("osViSetEvent".into(), Vram::new(0x80008760)).with_size(Size::new(0x58)),
        UserSymbol::new("osViSetMode".into(), Vram::new(0x800087C0)).with_size(Size::new(0x4C)),
        UserSymbol::new("osViSetSpecialFeatures".into(), Vram::new(0x80008810))
            .with_size(Size::new(0x164)),
        UserSymbol::new("osViSetYScale".into(), Vram::new(0x80008980)).with_size(Size::new(0x44)),
        UserSymbol::new("osViSwapBuffer".into(), Vram::new(0x800089D0)).with_size(Size::new(0x44)),
        UserSymbol::new("__osViSwapContext".into(), Vram::new(0x80008A20))
            .with_size(Size::new(0x308)),
        UserSymbol::new("osViBlack".into(), Vram::new(0x80008D30)).with_size(Size::new(0x5C)),
        UserSymbol::new("guMtxIdent".into(), Vram::new(0x80008DA0)),
        UserSymbol::new("guMtxIdentF".into(), Vram::new(0x80008E00)),
        UserSymbol::new("guMtxF2L".into(), Vram::new(0x80008E60)),
        UserSymbol::new("guMtxL2F".into(), Vram::new(0x80008EE0)),
        UserSymbol::new("guMtxCatF".into(), Vram::new(0x80008F60)),
        UserSymbol::new("guScale".into(), Vram::new(0x80009060)),
        UserSymbol::new("guScaleF".into(), Vram::new(0x80009120)),
        UserSymbol::new("guTranslate".into(), Vram::new(0x80009180)),
        UserSymbol::new("guTranslateF".into(), Vram::new(0x80009260)),
        UserSymbol::new("__createSpeedParam".into(), Vram::new(0x800092C0))
            .with_size(Size::new(0xB8)),
        UserSymbol::new("__osInitialize_common".into(), Vram::new(0x80009378))
            .with_size(Size::new(0x2D8)),
        UserSymbol::new("__osInitialize_autodetect".into(), Vram::new(0x80009650))
            .with_size(Size::new(0x8)),
        UserSymbol::new("osEepromRead".into(), Vram::new(0x80009660)),
        UserSymbol::new("__osPackEepReadData".into(), Vram::new(0x800097F8)),
        UserSymbol::new("osEepromWrite".into(), Vram::new(0x80009880)).with_size(Size::new(0x168)),
        UserSymbol::new("__osPackEepWriteData".into(), Vram::new(0x800099E8))
            .with_size(Size::new(0xA8)),
        UserSymbol::new("__osEepStatus".into(), Vram::new(0x80009A90)).with_size(Size::new(0x19C)),
        UserSymbol::new("__osExceptionPreamble".into(), Vram::new(0x80009C30)),
        UserSymbol::new("__osException".into(), Vram::new(0x80009C40)),
        UserSymbol::new("handle_interrupt".into(), Vram::new(0x80009E48))
            .with_typ(SymbolType::BranchLabel),
        UserSymbol::new("send_mesg".into(), Vram::new(0x8000A1BC)),
        UserSymbol::new("__osEnqueueAndYield".into(), Vram::new(0x8000A2AC)),
        UserSymbol::new("__osEnqueueThread".into(), Vram::new(0x8000A3B4)),
        UserSymbol::new("__osPopThread".into(), Vram::new(0x8000A3FC)),
        UserSymbol::new("__osNop".into(), Vram::new(0x8000A40C)),
        UserSymbol::new("__osDispatchThread".into(), Vram::new(0x8000A414)),
        UserSymbol::new("__osCleanupThread".into(), Vram::new(0x8000A590)),
        UserSymbol::new("__osDisableInt".into(), Vram::new(0x8000A5A0)),
        UserSymbol::new("__osRestoreInt".into(), Vram::new(0x8000A610)),
        UserSymbol::new("osSetIntMask".into(), Vram::new(0x8000A630)),
        UserSymbol::new("__osSetGlobalIntMask".into(), Vram::new(0x8000A6D0))
            .with_size(Size::new(0x40)),
        UserSymbol::new("__osResetGlobalIntMask".into(), Vram::new(0x8000A710))
            .with_size(Size::new(0x48)),
        UserSymbol::new("__osPiRawStartDma".into(), Vram::new(0x8000A760))
            .with_size(Size::new(0xD0)),
        UserSymbol::new("osPiGetCmdQueue".into(), Vram::new(0x8000A830)).with_size(Size::new(0x20)),
        UserSymbol::new("__osEPiRawReadIo".into(), Vram::new(0x8000A850))
            .with_size(Size::new(0x168)),
        UserSymbol::new("__osEPiRawWriteIo".into(), Vram::new(0x8000A9C0))
            .with_size(Size::new(0x164)),
        UserSymbol::new("__osAiDeviceBusy".into(), Vram::new(0x8000AB30))
            .with_size(Size::new(0x14)),
        UserSymbol::new("_init_lpfilter".into(), Vram::new(0x8000AB50)).with_size(Size::new(0xA4)),
        UserSymbol::new("alFxNew".into(), Vram::new(0x8000ABF4)).with_size(Size::new(0x498)),
        UserSymbol::new("alEnvmixerNew".into(), Vram::new(0x8000B08C)).with_size(Size::new(0xB8)),
        UserSymbol::new("alLoadNew".into(), Vram::new(0x8000B144)).with_size(Size::new(0xB0)),
        UserSymbol::new("alResampleNew".into(), Vram::new(0x8000B1F4)).with_size(Size::new(0x8C)),
        UserSymbol::new("alAuxBusNew".into(), Vram::new(0x8000B280)).with_size(Size::new(0x5C)),
        UserSymbol::new("alMainBusNew".into(), Vram::new(0x8000B2DC)).with_size(Size::new(0x5C)),
        UserSymbol::new("alSaveNew".into(), Vram::new(0x8000B338)).with_size(Size::new(0x44)),
        UserSymbol::new("alAdpcmPull".into(), Vram::new(0x8000B380)).with_size(Size::new(0x444)),
        UserSymbol::new("alRaw16Pull".into(), Vram::new(0x8000B7C4)).with_size(Size::new(0x39C)),
        UserSymbol::new("alLoadParam".into(), Vram::new(0x8000BB60)).with_size(Size::new(0x1AC)),
        UserSymbol::new("_decodeChunk".into(), Vram::new(0x8000BD0C)).with_size(Size::new(0x178)),
        UserSymbol::new("alAuxBusPull".into(), Vram::new(0x8000BE90)).with_size(Size::new(0xDC)),
        UserSymbol::new("alAuxBusParam".into(), Vram::new(0x8000BF6C)).with_size(Size::new(0x30)),
        UserSymbol::new("alFilterNew".into(), Vram::new(0x8000BFA0)).with_size(Size::new(0x1C)),
        UserSymbol::new("alMainBusPull".into(), Vram::new(0x8000BFC0)).with_size(Size::new(0x140)),
        UserSymbol::new("alMainBusParam".into(), Vram::new(0x8000C100)).with_size(Size::new(0x30)),
        UserSymbol::new("alResamplePull".into(), Vram::new(0x8000C130)).with_size(Size::new(0x1EC)),
        UserSymbol::new("alResampleParam".into(), Vram::new(0x8000C31C)).with_size(Size::new(0xC0)),
        UserSymbol::new("alFxPull".into(), Vram::new(0x8000C3E0)).with_size(Size::new(0x374)),
        UserSymbol::new("alFxParam".into(), Vram::new(0x8000C754)).with_size(Size::new(0x14)),
        UserSymbol::new("alFxParamHdl".into(), Vram::new(0x8000C768)).with_size(Size::new(0x1F0)),
        UserSymbol::new("_loadOutputBuffer".into(), Vram::new(0x8000C958))
            .with_size(Size::new(0x218)),
        UserSymbol::new("_loadBuffer".into(), Vram::new(0x8000CB70)).with_size(Size::new(0x170)),
        UserSymbol::new("_saveBuffer".into(), Vram::new(0x8000CCE0)).with_size(Size::new(0x170)),
        UserSymbol::new("_filterBuffer".into(), Vram::new(0x8000CE50)).with_size(Size::new(0x9C)),
        UserSymbol::new("_doModFunc".into(), Vram::new(0x8000CEEC)).with_size(Size::new(0x8C)),
        UserSymbol::new("alSavePull".into(), Vram::new(0x8000CF80)).with_size(Size::new(0xA8)),
        UserSymbol::new("alSaveParam".into(), Vram::new(0x8000D028)).with_size(Size::new(0x28)),
        UserSymbol::new("alHeapDBAlloc".into(), Vram::new(0x8000D050)).with_size(Size::new(0x48)),
        UserSymbol::new("alCopy".into(), Vram::new(0x8000D0A0)).with_size(Size::new(0x34)),
        UserSymbol::new("osInvalICache".into(), Vram::new(0x8000D0E0)),
        UserSymbol::new("osWritebackDCache".into(), Vram::new(0x8000D160)),
        UserSymbol::new("osDpSetNextBuffer".into(), Vram::new(0x8000D1E0))
            .with_size(Size::new(0x98)),
        UserSymbol::new("__osGetCause".into(), Vram::new(0x8000D280)),
        UserSymbol::new("osGetCount".into(), Vram::new(0x8000D290)),
        UserSymbol::new("__osGetSR".into(), Vram::new(0x8000D2A0)),
        UserSymbol::new("__osSetCompare".into(), Vram::new(0x8000D2B0)),
        UserSymbol::new("__osSetFpcCsr".into(), Vram::new(0x8000D2C0)),
        UserSymbol::new("__osSetSR".into(), Vram::new(0x8000D2D0)),
        UserSymbol::new("__osSetWatchLo".into(), Vram::new(0x8000D2E0)),
        UserSymbol::new("__osSpDeviceBusy".into(), Vram::new(0x8000D2F0))
            .with_size(Size::new(0x18)),
        UserSymbol::new("__osSpGetStatus".into(), Vram::new(0x8000D310)).with_size(Size::new(0x10)),
        UserSymbol::new("__osSpSetStatus".into(), Vram::new(0x8000D320)).with_size(Size::new(0x10)),
        UserSymbol::new("__osSpSetPc".into(), Vram::new(0x8000D330)).with_size(Size::new(0x30)),
        UserSymbol::new("__osSpRawStartDma".into(), Vram::new(0x8000D360))
            .with_size(Size::new(0x8C)),
        UserSymbol::new("__osSiRawReadIo".into(), Vram::new(0x8000D3F0)),
        UserSymbol::new("__osSiRawWriteIo".into(), Vram::new(0x8000D440)),
        UserSymbol::new("osDestroyThread".into(), Vram::new(0x8000D490)).with_size(Size::new(0xD8)),
        UserSymbol::new("osMapTLBRdb".into(), Vram::new(0x8000D570)),
        UserSymbol::new("osUnmapTLBAll".into(), Vram::new(0x8000D5D0)),
        UserSymbol::new("__osViInit".into(), Vram::new(0x8000D620)).with_size(Size::new(0x104)),
        UserSymbol::new("__osViGetCurrentContext".into(), Vram::new(0x8000D730)),
        UserSymbol::new("__osDpDeviceBusy".into(), Vram::new(0x8000D7A0))
            .with_size(Size::new(0x18)),
        UserSymbol::new("__osSiDeviceBusy".into(), Vram::new(0x8000D7C0))
            .with_size(Size::new(0x18)),
        UserSymbol::new("__fint".into(), Vram::new(0x8000D7E0)),
        UserSymbol::new("fmod".into(), Vram::new(0x8000D868)),
        UserSymbol::new("ceil".into(), Vram::new(0x8000D8C4)),
        UserSymbol::new("floor".into(), Vram::new(0x8000D940)),
        UserSymbol::new("_memmover".into(), Vram::new(0x8000D9C0)),
        UserSymbol::new("memmove".into(), Vram::new(0x8000DAC4)),
        UserSymbol::new("movmem".into(), Vram::new(0x8000DB0C)),
        UserSymbol::new("memset".into(), Vram::new(0x8000DB30)),
        UserSymbol::new("setmem".into(), Vram::new(0x8000DBD0)),
        UserSymbol::new("modf".into(), Vram::new(0x8000DC00)),
        UserSymbol::new("rand".into(), Vram::new(0x8000DCB0)),
        UserSymbol::new("srand".into(), Vram::new(0x8000DCE4)),
        UserSymbol::new("strcpy".into(), Vram::new(0x8000DCF0)),
        UserSymbol::new("__muldi3".into(), Vram::new(0x8000DDC0)),
        UserSymbol::new("div64_64".into(), Vram::new(0x8000DDF8)),
        UserSymbol::new("div64_32".into(), Vram::new(0x8000DEF4)),
        UserSymbol::new("__divdi3".into(), Vram::new(0x8000DFC0)),
        UserSymbol::new("__moddi3".into(), Vram::new(0x8000E070)),
        UserSymbol::new("__udivdi3".into(), Vram::new(0x8000E130)),
        UserSymbol::new("__umoddi3".into(), Vram::new(0x8000E150)).with_typ(SymbolType::Function),
        UserSymbol::new("D_8000E190".into(), Vram::new(0x8000E190)),
        UserSymbol::new("sDmaDataNeedsInitialization".into(), Vram::new(0x8000E1A0))
            .with_size(Size::new(0x1)),
        UserSymbol::new("border".into(), Vram::new(0x8000E1B0)),
        UserSymbol::new("cplens".into(), Vram::new(0x8000E1FC)),
        UserSymbol::new("cplext".into(), Vram::new(0x8000E23C)),
        UserSymbol::new("cpdist".into(), Vram::new(0x8000E27C)),
        UserSymbol::new("cpdext".into(), Vram::new(0x8000E2B8)),
        UserSymbol::new("mask_bits".into(), Vram::new(0x8000E2F4)),
        UserSymbol::new("lbits".into(), Vram::new(0x8000E318)),
        UserSymbol::new("dbits".into(), Vram::new(0x8000E31C)),
        UserSymbol::new("gzip_malloc_addr".into(), Vram::new(0x8000E320)),
        UserSymbol::new("gzip_malloc_tmp".into(), Vram::new(0x8000E324)),
        UserSymbol::new("D_8000E330".into(), Vram::new(0x8000E330)).with_typ(SymbolType::Word),
        UserSymbol::new("crc_32_tab".into(), Vram::new(0x8000E338)),
        UserSymbol::new("crc_132".into(), Vram::new(0x8000E738)),
        UserSymbol::new("storyRomData".into(), Vram::new(0x8000E740)).with_size(Size::new(0x38)),
        UserSymbol::new("bgRomData".into(), Vram::new(0x8000E778)).with_size(Size::new(0xC0)),
        UserSymbol::new("_romDataTbl".into(), Vram::new(0x8000E838)).with_size(Size::new(0x218)),
        UserSymbol::new("__osPiDevMgr".into(), Vram::new(0x8000EA50)).with_size(Size::new(0x1C)),
        UserSymbol::new("__osPiTable".into(), Vram::new(0x8000EA6C)),
        UserSymbol::new("__osCurrentHandle".into(), Vram::new(0x8000EA70)),
        UserSymbol::new(
            "osCartRomInit$in_function_static$osCartRomInit".into(),
            Vram::new(0x8000EA80),
        )
        .with_size(Size::new(0x4))
        .with_typ(SymbolType::Word),
        UserSymbol::new("__osPiAccessQueueEnabled".into(), Vram::new(0x8000EA90)),
        UserSymbol::new("hdwrBugFlag".into(), Vram::new(0x8000EAA0)),
        UserSymbol::new("eqpower".into(), Vram::new(0x8000EAB0)).with_size(Size::new(0x100)),
        UserSymbol::new("alGlobals".into(), Vram::new(0x8000EBB0)),
        UserSymbol::new("__osContinitialized".into(), Vram::new(0x8000EBC0)),
        UserSymbol::new("D_8000EBD0".into(), Vram::new(0x8000EBD0)),
        UserSymbol::new("sintable".into(), Vram::new(0x8000EBE0)),
        UserSymbol::new("__osPreNMI".into(), Vram::new(0x8000F3E0)),
        UserSymbol::new("dp_busy".into(), Vram::new(0x8000F3F4)),
        UserSymbol::new("dpCount".into(), Vram::new(0x8000F3F8)),
        UserSymbol::new("D_8000F3FC".into(), Vram::new(0x8000F3FC)),
        UserSymbol::new("__osSiAccessQueueEnabled".into(), Vram::new(0x8000F400)),
        UserSymbol::new("__osThreadTail".into(), Vram::new(0x8000F410)).with_size(Size::new(0x8)),
        UserSymbol::new("__osRunQueue".into(), Vram::new(0x8000F418)),
        UserSymbol::new("__osActiveQueue".into(), Vram::new(0x8000F41C)),
        UserSymbol::new("__osRunningThread".into(), Vram::new(0x8000F420)),
        UserSymbol::new("__osFaultedThread".into(), Vram::new(0x8000F424)),
        UserSymbol::new("__osTimerList".into(), Vram::new(0x8000F430)),
        UserSymbol::new("__osViDevMgr".into(), Vram::new(0x8000F440)).with_size(Size::new(0x1C)),
        UserSymbol::new("__additional_scanline".into(), Vram::new(0x8000F45C)),
        UserSymbol::new("osViModeTable".into(), Vram::new(0x8000F460)).with_size(Size::new(0x10A0)),
        UserSymbol::new("osClockRate".into(), Vram::new(0x800105E0)).with_size(Size::new(0x8)),
        UserSymbol::new("osViClock".into(), Vram::new(0x800105E8)),
        UserSymbol::new("__osShutdown".into(), Vram::new(0x800105EC)),
        UserSymbol::new("__OSGlobalIntMask".into(), Vram::new(0x800105F0)),
        UserSymbol::new("__osHwIntTable".into(), Vram::new(0x80010600)),
        UserSymbol::new("__osPiIntTable".into(), Vram::new(0x80010628)),
        UserSymbol::new("SMALLROOM_PARAMS".into(), Vram::new(0x80010630)),
        UserSymbol::new("BIGROOM_PARAMS".into(), Vram::new(0x80010698)),
        UserSymbol::new("ECHO_PARAMS".into(), Vram::new(0x80010720)),
        UserSymbol::new("CHORUS_PARAMS".into(), Vram::new(0x80010748)),
        UserSymbol::new("FLANGE_PARAMS".into(), Vram::new(0x80010770)),
        UserSymbol::new("NULL_PARAMS".into(), Vram::new(0x80010798)),
        UserSymbol::new("L_INC".into(), Vram::new(0x800107C0)).with_size(Size::new(0xC)),
        UserSymbol::new(
            "_loadOutputBuffer$in_function_static$val".into(),
            Vram::new(0x800107CC),
        )
        .with_typ(SymbolType::Float32),
        UserSymbol::new(
            "_loadOutputBuffer$in_function_static$lastval".into(),
            Vram::new(0x800107D0),
        )
        .with_typ(SymbolType::Float32),
        UserSymbol::new(
            "_loadOutputBuffer$in_function_static$blob".into(),
            Vram::new(0x800107D4),
        )
        .with_typ(SymbolType::Float32),
        UserSymbol::new("vi".into(), Vram::new(0x800107E0)).with_size(Size::new(0x60)),
        UserSymbol::new("__osViCurr".into(), Vram::new(0x80010840)),
        UserSymbol::new("__osViNext".into(), Vram::new(0x80010844)),
        UserSymbol::new("osViModeNtscLan1".into(), Vram::new(0x80010850)),
        UserSymbol::new("osViModePalLan1".into(), Vram::new(0x800108A0)),
        UserSymbol::new("osViModeMpalLan1".into(), Vram::new(0x800108F0))
            .with_size(Size::new(0x50)),
        UserSymbol::new("__ctype_map".into(), Vram::new(0x80010940))
            .with_size(Size::new(0x100))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("__osIntOffTable".into(), Vram::new(0x80010CB0)),
        UserSymbol::new("__osIntTable".into(), Vram::new(0x80010CD0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("__osRcpImTable".into(), Vram::new(0x80010D00)),
        UserSymbol::new("__libm_qnan_f".into(), Vram::new(0x80010E40)),
        UserSymbol::new("sIdleThread".into(), Vram::new(0x80010E60)),
        UserSymbol::new("sMainThread".into(), Vram::new(0x80011010)),
        UserSymbol::new("sIdleStack".into(), Vram::new(0x800111C0)).with_size(Size::new(0x2000)),
        UserSymbol::new("sMainStack".into(), Vram::new(0x800131C0)).with_size(Size::new(0x2000)),
        UserSymbol::new("B_800151C0".into(), Vram::new(0x800151C0)),
        UserSymbol::new("B_800151D8".into(), Vram::new(0x800151D8)),
        UserSymbol::new("gzip_mem_buff".into(), Vram::new(0x800151E0)),
        UserSymbol::new("piThread".into(), Vram::new(0x800191E0)).with_size(Size::new(0x1B0)),
        UserSymbol::new("piThreadStack".into(), Vram::new(0x80019390)),
        UserSymbol::new("piEventQueue".into(), Vram::new(0x8001A390)),
        UserSymbol::new("piEventBuf".into(), Vram::new(0x8001A3A8)),
        UserSymbol::new("piAccessBuf".into(), Vram::new(0x8001A3B0)).with_size(Size::new(0x4)),
        UserSymbol::new("tmp_task".into(), Vram::new(0x8001A3C0))
            .with_size(Size::new(0x40))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("siAccessBuf".into(), Vram::new(0x8001A400)).with_size(Size::new(0x4)),
        UserSymbol::new(
            "viMgrMain$in_function_static$retrace".into(),
            Vram::new(0x8001A410),
        )
        .with_size(Size::new(0x2))
        .with_typ(SymbolType::Short),
        UserSymbol::new("viThread".into(), Vram::new(0x8001A418)).with_size(Size::new(0x1B0)),
        UserSymbol::new("viThreadStack".into(), Vram::new(0x8001A5D0)),
        UserSymbol::new("viEventQueue".into(), Vram::new(0x8001B5D0)),
        UserSymbol::new("viEventBuf".into(), Vram::new(0x8001B5E8)),
        UserSymbol::new("viRetraceMsg".into(), Vram::new(0x8001B600)).with_size(Size::new(0x18)),
        UserSymbol::new("viCounterMsg".into(), Vram::new(0x8001B618)).with_size(Size::new(0x18)),
        UserSymbol::new("sRandNext".into(), Vram::new(0x8001B630)),
        UserSymbol::new("inbuf".into(), Vram::new(0x8001B640)),
        UserSymbol::new("bk".into(), Vram::new(0x8001D640)),
        UserSymbol::new("__osBaseCounter".into(), Vram::new(0x8001D644)),
        UserSymbol::new("__osThreadSave".into(), Vram::new(0x8001D648)),
        UserSymbol::new("ofd".into(), Vram::new(0x8001D7F8)).with_size(Size::new(0x8)),
        UserSymbol::new("__osContPifRam".into(), Vram::new(0x8001D800)).with_size(Size::new(0x40)),
        UserSymbol::new("__Dom2SpeedParam".into(), Vram::new(0x8001D840))
            .with_size(Size::new(0x74)),
        UserSymbol::new("gBootThreadStack".into(), Vram::new(0x8001D8C0))
            .with_size(Size::new(0x2000)),
        UserSymbol::new("sPiMgrCmdBuff".into(), Vram::new(0x8001F8C0)).with_size(Size::new(0xC8)),
        UserSymbol::new("__osViIntrCount".into(), Vram::new(0x8001F988)),
        UserSymbol::new("insize".into(), Vram::new(0x8001F98C)),
        UserSymbol::new("outcnt".into(), Vram::new(0x8001F990)),
        UserSymbol::new("__osMaxControllers".into(), Vram::new(0x8001F994))
            .with_size(Size::new(0x1)),
        UserSymbol::new("bb".into(), Vram::new(0x8001F998)).with_size(Size::new(0x4)),
        UserSymbol::new("__osCurrentTime".into(), Vram::new(0x8001F9A0)).with_size(Size::new(0x8)),
        UserSymbol::new("hufts".into(), Vram::new(0x8001F9A8)).with_size(Size::new(0x4)),
        UserSymbol::new("__CartRomHandle".into(), Vram::new(0x8001F9B0)).with_size(Size::new(0x74)),
        UserSymbol::new("__osEepPifRam".into(), Vram::new(0x8001FA30))
            .with_size(Size::new(0x40))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("__osPiAccessQueue".into(), Vram::new(0x8001FA70)),
        UserSymbol::new("__Dom1SpeedParam".into(), Vram::new(0x8001FA88))
            .with_size(Size::new(0x74)),
        UserSymbol::new("bytes_in".into(), Vram::new(0x8001FAFC)),
        UserSymbol::new("bytes_out".into(), Vram::new(0x8001FB00)),
        UserSymbol::new("__osContLastCmd".into(), Vram::new(0x8001FB04)).with_size(Size::new(0x1)),
        UserSymbol::new("__osEepromTimerMsg".into(), Vram::new(0x8001FB08))
            .with_size(Size::new(0x4)),
        UserSymbol::new("__osBaseTimer".into(), Vram::new(0x8001FB10)).with_size(Size::new(0x20)),
        UserSymbol::new("__osTimerCounter".into(), Vram::new(0x8001FB30)).with_size(Size::new(0x4)),
        UserSymbol::new("D_8001FB40".into(), Vram::new(0x8001FB40)).with_size(Size::new(0x2000)),
        UserSymbol::new("__osEepromTimer".into(), Vram::new(0x80021B40)),
        UserSymbol::new("__osEventStateTab".into(), Vram::new(0x80021B60))
            .with_size(Size::new(0x78)),
        UserSymbol::new("__osFinalrom".into(), Vram::new(0x80021BD8)).with_size(Size::new(0x4)),
        UserSymbol::new("window".into(), Vram::new(0x80021BE0)),
        UserSymbol::new("inptr".into(), Vram::new(0x80029BE0)).with_size(Size::new(0x4)),
        UserSymbol::new("__osEepromTimerQ".into(), Vram::new(0x80029BE8)),
        UserSymbol::new("ifd".into(), Vram::new(0x80029C00)).with_size(Size::new(0x8)),
        UserSymbol::new("sPiMgrCmdQueue".into(), Vram::new(0x80029C08)),
        UserSymbol::new("__osSiAccessQueue".into(), Vram::new(0x80029C20)),
        UserSymbol::new("Heap_bufferp".into(), Vram::new(0x80124610)).with_size(Size::new(0xE09F0)),
        UserSymbol::new("gfx_freebuf".into(), Vram::new(0x80205000)).with_size(Size::new(0x1B0000)),
        UserSymbol::new("gFramebuffers".into(), Vram::new(0x803B5000))
            .with_size(Size::new(0x4B000)),
        UserSymbol::new("gMainSegmentDmaInfo".into(), Vram::new(0x80029C40))
            .with_size(Size::new(0x10)),
        UserSymbol::new("mainproc".into(), Vram::new(0x80029C50)),
        UserSymbol::new("nnScCreateScheduler".into(), Vram::new(0x80029ED0)),
        UserSymbol::new("nnScGetAudioMQ".into(), Vram::new(0x8002A0CC)),
        UserSymbol::new("nnScGetGfxMQ".into(), Vram::new(0x8002A0D4)),
        UserSymbol::new("nnScEventHandler".into(), Vram::new(0x8002A0DC)),
        UserSymbol::new("nnScAddClient".into(), Vram::new(0x8002A184)),
        UserSymbol::new("nnScRemoveClient".into(), Vram::new(0x8002A1DC)),
        UserSymbol::new("nnScEventBroadcast".into(), Vram::new(0x8002A26C)),
        UserSymbol::new("nnScExecuteAudio".into(), Vram::new(0x8002A2B8)),
        UserSymbol::new("func_8002A3F4".into(), Vram::new(0x8002A3F4)),
        UserSymbol::new("nnScExecuteGraphics".into(), Vram::new(0x8002A4D8)),
        UserSymbol::new("nnScWaitTaskReady".into(), Vram::new(0x8002A51C)),
        UserSymbol::new("joyInit".into(), Vram::new(0x8002A5B0)),
        UserSymbol::new("joyProcCore".into(), Vram::new(0x8002A700)),
        UserSymbol::new("joyCursorFastSet".into(), Vram::new(0x8002A8F8)),
        UserSymbol::new("joyResponseCheck".into(), Vram::new(0x8002A954)),
        UserSymbol::new("func_8002AA80".into(), Vram::new(0x8002AA80)),
        UserSymbol::new("func_8002AAD8".into(), Vram::new(0x8002AAD8)),
        UserSymbol::new("func_8002AAE8".into(), Vram::new(0x8002AAE8)),
        UserSymbol::new("func_8002AB28".into(), Vram::new(0x8002AB28)),
        UserSymbol::new("func_8002ABC0".into(), Vram::new(0x8002ABC0)),
        UserSymbol::new("func_8002AC64".into(), Vram::new(0x8002AC64)),
        UserSymbol::new("dm_audio_set_stereo".into(), Vram::new(0x8002ACE0)),
        UserSymbol::new("dm_audio_init_driver".into(), Vram::new(0x8002AD38)),
        UserSymbol::new("dm_audio_update".into(), Vram::new(0x8002AE58)),
        UserSymbol::new("dm_audio_stop".into(), Vram::new(0x8002AF7C)),
        UserSymbol::new("dm_audio_is_stopped".into(), Vram::new(0x8002AFA4)),
        UserSymbol::new("dm_seq_play".into(), Vram::new(0x8002AFC4)),
        UserSymbol::new("_dm_seq_play".into(), Vram::new(0x8002AFE4)),
        UserSymbol::new("dm_seq_play_fade".into(), Vram::new(0x8002B000)),
        UserSymbol::new("_dm_seq_play_fade".into(), Vram::new(0x8002B028)),
        UserSymbol::new("dm_seq_play_in_game".into(), Vram::new(0x8002B078)),
        UserSymbol::new("_dm_seq_play_in_game".into(), Vram::new(0x8002B098)),
        UserSymbol::new("dm_seq_stop".into(), Vram::new(0x8002B0E4)),
        UserSymbol::new("_dm_seq_stop".into(), Vram::new(0x8002B100)),
        UserSymbol::new("dm_seq_set_volume".into(), Vram::new(0x8002B13C)),
        UserSymbol::new("_dm_seq_set_volume".into(), Vram::new(0x8002B15C)),
        UserSymbol::new("func_8002B178".into(), Vram::new(0x8002B178)),
        UserSymbol::new("func_8002B194".into(), Vram::new(0x8002B194)),
        UserSymbol::new("dm_snd_play".into(), Vram::new(0x8002B1B4)),
        UserSymbol::new("dm_snd_play_in_game".into(), Vram::new(0x8002B344)),
        UserSymbol::new("func_8002B370".into(), Vram::new(0x8002B370)),
        UserSymbol::new("dm_snd_play_strange_sound".into(), Vram::new(0x8002B394)),
        UserSymbol::new("gfxInit".into(), Vram::new(0x8002B490)),
        UserSymbol::new("gfxproc".into(), Vram::new(0x8002B4BC)),
        UserSymbol::new("gfxproc_onRetrace".into(), Vram::new(0x8002B5E4)),
        UserSymbol::new("func_8002B710".into(), Vram::new(0x8002B710)),
        UserSymbol::new("func_8002B728".into(), Vram::new(0x8002B728)),
        UserSymbol::new("func_8002B754".into(), Vram::new(0x8002B754)),
        UserSymbol::new("gfxCreateGraphicThread".into(), Vram::new(0x8002B7A8)),
        UserSymbol::new("gfxWaitMessage".into(), Vram::new(0x8002B800)),
        UserSymbol::new("gfxTaskStart".into(), Vram::new(0x8002B834)),
        UserSymbol::new("F3RCPinitRtn".into(), Vram::new(0x8002B9D8)),
        UserSymbol::new("F3ClearFZRtn".into(), Vram::new(0x8002BAB8)),
        UserSymbol::new("S2RDPinitRtn".into(), Vram::new(0x8002BBD8)),
        UserSymbol::new("S2ClearCFBRtn".into(), Vram::new(0x8002BC58)),
        UserSymbol::new("func_8002BD20".into(), Vram::new(0x8002BD20)),
        UserSymbol::new("load_TexPal".into(), Vram::new(0x8002BEE4)),
        UserSymbol::new("func_8002BF90".into(), Vram::new(0x8002BF90)),
        UserSymbol::new("load_TexBlock_4b".into(), Vram::new(0x8002C048)),
        UserSymbol::new("func_8002C1E8".into(), Vram::new(0x8002C1E8)),
        UserSymbol::new("load_TexTile_4b".into(), Vram::new(0x8002C388)),
        UserSymbol::new("func_8002C4EC".into(), Vram::new(0x8002C4EC)),
        UserSymbol::new("func_8002C688".into(), Vram::new(0x8002C688)),
        UserSymbol::new("draw_Tex".into(), Vram::new(0x8002C7C0)),
        UserSymbol::new("func_8002C914".into(), Vram::new(0x8002C914)),
        UserSymbol::new("func_8002CA68".into(), Vram::new(0x8002CA68)),
        UserSymbol::new("func_8002CD08".into(), Vram::new(0x8002CD08)),
        UserSymbol::new("func_8002CF04".into(), Vram::new(0x8002CF04)),
        UserSymbol::new("InitMusicDriver".into(), Vram::new(0x8002D170)),
        UserSymbol::new("func_8002D3B0".into(), Vram::new(0x8002D3B0)),
        UserSymbol::new("func_8002D428".into(), Vram::new(0x8002D428)),
        UserSymbol::new("func_8002D4A4".into(), Vram::new(0x8002D4A4)),
        UserSymbol::new("func_8002D4F8".into(), Vram::new(0x8002D4F8)),
        UserSymbol::new("func_8002D51C".into(), Vram::new(0x8002D51C)),
        UserSymbol::new("func_8002D554".into(), Vram::new(0x8002D554)),
        UserSymbol::new("func_8002D58C".into(), Vram::new(0x8002D58C)),
        UserSymbol::new("func_8002D5C4".into(), Vram::new(0x8002D5C4)),
        UserSymbol::new("func_8002D5FC".into(), Vram::new(0x8002D5FC)),
        UserSymbol::new("func_8002D634".into(), Vram::new(0x8002D634)),
        UserSymbol::new("func_8002D66C".into(), Vram::new(0x8002D66C)),
        UserSymbol::new("func_8002D6A4".into(), Vram::new(0x8002D6A4)),
        UserSymbol::new("func_8002D710".into(), Vram::new(0x8002D710)),
        UserSymbol::new("func_8002D720".into(), Vram::new(0x8002D720)),
        UserSymbol::new("func_8002D768".into(), Vram::new(0x8002D768)),
        UserSymbol::new("func_8002D7C4".into(), Vram::new(0x8002D7C4)),
        UserSymbol::new("func_8002D7E0".into(), Vram::new(0x8002D7E0)),
        UserSymbol::new("func_8002D810".into(), Vram::new(0x8002D810)),
        UserSymbol::new("func_8002D840".into(), Vram::new(0x8002D840)),
        UserSymbol::new("func_8002D870".into(), Vram::new(0x8002D870)),
        UserSymbol::new("func_8002D8A0".into(), Vram::new(0x8002D8A0)),
        UserSymbol::new("DmaRomToRam".into(), Vram::new(0x8002D8D0)),
        UserSymbol::new("NnSchedInstall".into(), Vram::new(0x8002D984)),
        UserSymbol::new("NnSchedWaitFrame".into(), Vram::new(0x8002D9E4)),
        UserSymbol::new("NnSchedDoTask".into(), Vram::new(0x8002DA48)),
        UserSymbol::new("main11".into(), Vram::new(0x8002DB20)),
        UserSymbol::new("adjust_story_ai".into(), Vram::new(0x8002DC10)),
        UserSymbol::new("main12".into(), Vram::new(0x8002E230)),
        UserSymbol::new("sgenrand".into(), Vram::new(0x8002E830)),
        UserSymbol::new("genrand".into(), Vram::new(0x8002E8B0)),
        UserSymbol::new("func_8002EB00".into(), Vram::new(0x8002EB00)),
        UserSymbol::new("aifMakeBlkWork".into(), Vram::new(0x8002EB74)),
        UserSymbol::new("aifMakeFlagSet".into(), Vram::new(0x8002ED14)),
        UserSymbol::new("aifGameInit".into(), Vram::new(0x8002ED2C)),
        UserSymbol::new("aifFirstInit".into(), Vram::new(0x8002EDDC)),
        UserSymbol::new("aifMake".into(), Vram::new(0x8002EFF0)),
        UserSymbol::new("aifMake2".into(), Vram::new(0x8002F180)),
        UserSymbol::new("aiHiruSideLineEraser".into(), Vram::new(0x8002F2EC)),
        UserSymbol::new("func_8002F924".into(), Vram::new(0x8002F924)),
        UserSymbol::new("aifEraseLineCore".into(), Vram::new(0x8002F9CC)),
        UserSymbol::new("aifRensaCheckCore".into(), Vram::new(0x8002FDF0)),
        UserSymbol::new("aifRensaCheck".into(), Vram::new(0x80030810)),
        UserSymbol::new("aifSearchLineCore".into(), Vram::new(0x800308FC)),
        UserSymbol::new("aifMiniPointK3".into(), Vram::new(0x800310C8)),
        UserSymbol::new("aifMiniAloneCapNumber".into(), Vram::new(0x80031218)),
        UserSymbol::new("aifMiniAloneCapNumberW".into(), Vram::new(0x8003135C)),
        UserSymbol::new("flash_virus".into(), Vram::new(0x800313F0)),
        UserSymbol::new("search_Vflash".into(), Vram::new(0x8003145C)),
        UserSymbol::new("func_8003151C".into(), Vram::new(0x8003151C)),
        UserSymbol::new("aifSearchLineMS".into(), Vram::new(0x8003157C)),
        UserSymbol::new("aiHiruAllPriSet".into(), Vram::new(0x800325AC)),
        UserSymbol::new("aiSetCharacter".into(), Vram::new(0x80033474)),
        UserSymbol::new("aifFieldCopy".into(), Vram::new(0x80034200)),
        UserSymbol::new("func_80034310".into(), Vram::new(0x80034310)),
        UserSymbol::new("aifPlaceSearch".into(), Vram::new(0x80034380)),
        UserSymbol::new("aifMoveCheck".into(), Vram::new(0x80034798)),
        UserSymbol::new("aifTRecur".into(), Vram::new(0x80034AB0)),
        UserSymbol::new("aifTRecurUP".into(), Vram::new(0x80034D54)),
        UserSymbol::new("aifYRecur".into(), Vram::new(0x80034E80)),
        UserSymbol::new("aifYRecurUP".into(), Vram::new(0x8003510C)),
        UserSymbol::new("aifReMoveCheck".into(), Vram::new(0x80035234)),
        UserSymbol::new("aifKeyMake".into(), Vram::new(0x80035494)),
        UserSymbol::new("aifKeyOut".into(), Vram::new(0x80035704)),
        UserSymbol::new("search_flash_3".into(), Vram::new(0x80035CF4)),
        UserSymbol::new("flash_special".into(), Vram::new(0x800362A0)),
        UserSymbol::new("aiCOM_MissTake".into(), Vram::new(0x80036560)),
        UserSymbol::new("func_800365B0".into(), Vram::new(0x800365B0)),
        UserSymbol::new("func_800365C8".into(), Vram::new(0x800365C8)),
        UserSymbol::new("BitField_PutBit".into(), Vram::new(0x8003661C)),
        UserSymbol::new("BitField_GetBit".into(), Vram::new(0x800366A4)),
        UserSymbol::new("dm_init_config_save".into(), Vram::new(0x80036718)),
        UserSymbol::new("func_8003678C".into(), Vram::new(0x8003678C)),
        UserSymbol::new("func_800367E0".into(), Vram::new(0x800367E0)),
        UserSymbol::new("func_800367F0".into(), Vram::new(0x800367F0)),
        UserSymbol::new("func_800367FC".into(), Vram::new(0x800367FC)),
        UserSymbol::new("func_80036808".into(), Vram::new(0x80036808)),
        UserSymbol::new("dm_init_save_mem".into(), Vram::new(0x80036818)),
        UserSymbol::new("dm_init_system_mem".into(), Vram::new(0x800369BC)),
        UserSymbol::new("dm_story_sort_set".into(), Vram::new(0x80036A64)),
        UserSymbol::new("dm_level_sort_set".into(), Vram::new(0x80036B6C)),
        UserSymbol::new("dm_taiQ_sort_set".into(), Vram::new(0x80036BC8)),
        UserSymbol::new("dm_timeAt_sort_set".into(), Vram::new(0x80036C58)),
        UserSymbol::new("dm_vscom_set".into(), Vram::new(0x80036CFC)),
        UserSymbol::new("dm_vc_fl_set".into(), Vram::new(0x80036D58)),
        UserSymbol::new("dm_vsman_set".into(), Vram::new(0x80036DB4)),
        UserSymbol::new("dm_vm_fl_set".into(), Vram::new(0x80036E10)),
        UserSymbol::new("dm_vm_ta_set".into(), Vram::new(0x80036E6C)),
        UserSymbol::new("func_80036EC8".into(), Vram::new(0x80036EC8)),
        UserSymbol::new("func_80036F1C".into(), Vram::new(0x80036F1C)),
        UserSymbol::new("_get1PLess".into(), Vram::new(0x80036F70)),
        UserSymbol::new("_sort1PMode".into(), Vram::new(0x80037068)),
        UserSymbol::new("dm_data_mode_story_sort".into(), Vram::new(0x80037248)),
        UserSymbol::new("dm_data_mode_level_sort".into(), Vram::new(0x80037294)),
        UserSymbol::new("dm_data_mode_taiQ_sort".into(), Vram::new(0x800372E0)),
        UserSymbol::new("dm_data_mode_timeAt_sort".into(), Vram::new(0x8003732C)),
        UserSymbol::new("func_80037378".into(), Vram::new(0x80037378)),
        UserSymbol::new("func_800373C8".into(), Vram::new(0x800373C8)),
        UserSymbol::new("func_80037418".into(), Vram::new(0x80037418)),
        UserSymbol::new("func_80037468".into(), Vram::new(0x80037468)),
        UserSymbol::new("_sortVsMode".into(), Vram::new(0x800374B8)),
        UserSymbol::new("dm_data_vscom_sort".into(), Vram::new(0x80037710)),
        UserSymbol::new("dm_data_vc_fl_sort".into(), Vram::new(0x8003772C)),
        UserSymbol::new("dm_data_vsman_sort".into(), Vram::new(0x80037748)),
        UserSymbol::new("dm_data_vm_fl_sort".into(), Vram::new(0x80037764)),
        UserSymbol::new("dm_data_vm_ta_sort".into(), Vram::new(0x80037780)),
        UserSymbol::new("func_8003779C".into(), Vram::new(0x8003779C)),
        UserSymbol::new("func_80037808".into(), Vram::new(0x80037808)),
        UserSymbol::new("func_800378B0".into(), Vram::new(0x800378B0)),
        UserSymbol::new("func_80037950".into(), Vram::new(0x80037950)),
        UserSymbol::new("func_800379D4".into(), Vram::new(0x800379D4)),
        UserSymbol::new("RecStory_Compress".into(), Vram::new(0x80037A5C)),
        UserSymbol::new("RecStory_Extract".into(), Vram::new(0x80037B74)),
        UserSymbol::new("func_80037C7C".into(), Vram::new(0x80037C7C)),
        UserSymbol::new("func_80037D40".into(), Vram::new(0x80037D40)),
        UserSymbol::new("func_80037DF0".into(), Vram::new(0x80037DF0)),
        UserSymbol::new("func_80037EB4".into(), Vram::new(0x80037EB4)),
        UserSymbol::new("func_80037F64".into(), Vram::new(0x80037F64)),
        UserSymbol::new("func_80038038".into(), Vram::new(0x80038038)),
        UserSymbol::new("func_800380F0".into(), Vram::new(0x800380F0)),
        UserSymbol::new("func_80038168".into(), Vram::new(0x80038168)),
        UserSymbol::new("func_800381DC".into(), Vram::new(0x800381DC)),
        UserSymbol::new("func_80038254".into(), Vram::new(0x80038254)),
        UserSymbol::new("func_800382C8".into(), Vram::new(0x800382C8)),
        UserSymbol::new("func_80038340".into(), Vram::new(0x80038340)),
        UserSymbol::new("func_800383B4".into(), Vram::new(0x800383B4)),
        UserSymbol::new("func_8003842C".into(), Vram::new(0x8003842C)),
        UserSymbol::new("func_800384A0".into(), Vram::new(0x800384A0)),
        UserSymbol::new("func_80038518".into(), Vram::new(0x80038518)),
        UserSymbol::new("RecAll_Compress".into(), Vram::new(0x8003858C)),
        UserSymbol::new("RecAll_Extract".into(), Vram::new(0x800386AC)),
        UserSymbol::new("EepRom_Init".into(), Vram::new(0x80038824)),
        UserSymbol::new("EepRom_InitFirst".into(), Vram::new(0x80038878)),
        UserSymbol::new("EepRom_InitVars".into(), Vram::new(0x800388E0)),
        UserSymbol::new("func_80038938".into(), Vram::new(0x80038938)),
        UserSymbol::new("EepRom_WriteDif".into(), Vram::new(0x800389A8)),
        UserSymbol::new("EepRom_ReadAll".into(), Vram::new(0x80038A94)),
        UserSymbol::new("EepRom_WriteAll".into(), Vram::new(0x80038B18)),
        UserSymbol::new("EepRom_DumpErrMes".into(), Vram::new(0x80038BD0)),
        UserSymbol::new("EepRom_DumpDataSize".into(), Vram::new(0x80038BD8)),
        UserSymbol::new("RecWritingMsg_init".into(), Vram::new(0x80038BE0)),
        UserSymbol::new("RecWritingMsg_setStr".into(), Vram::new(0x80038C38)),
        UserSymbol::new("RecWritingMsg_calc".into(), Vram::new(0x80038C7C)),
        UserSymbol::new("RecWritingMsg_draw".into(), Vram::new(0x80038CBC)),
        UserSymbol::new("RecWritingMsg_start".into(), Vram::new(0x80038E1C)),
        UserSymbol::new("RecWritingMsg_end".into(), Vram::new(0x80038E24)),
        UserSymbol::new("RecWritingMsg_isEnd".into(), Vram::new(0x80038E30)),
        UserSymbol::new("RecWritingMsg_setPos".into(), Vram::new(0x80038E44)),
        UserSymbol::new("setSleepTimer".into(), Vram::new(0x80038E50)),
        UserSymbol::new("func_80038EF0".into(), Vram::new(0x80038EF0)),
        UserSymbol::new("func_8003901C".into(), Vram::new(0x8003901C)),
        UserSymbol::new("func_800393DC".into(), Vram::new(0x800393DC)),
        UserSymbol::new("func_800394A0".into(), Vram::new(0x800394A0)),
        UserSymbol::new("func_8003974C".into(), Vram::new(0x8003974C)),
        UserSymbol::new("func_80039BE0".into(), Vram::new(0x80039BE0)),
        UserSymbol::new("func_80039D08".into(), Vram::new(0x80039D08)),
        UserSymbol::new("func_80039E14".into(), Vram::new(0x80039E14)),
        UserSymbol::new("func_80039F74".into(), Vram::new(0x80039F74)),
        UserSymbol::new("func_8003A1B4".into(), Vram::new(0x8003A1B4)),
        UserSymbol::new("func_8003A26C".into(), Vram::new(0x8003A26C)),
        UserSymbol::new("func_8003A618".into(), Vram::new(0x8003A618)),
        UserSymbol::new("func_8003A7E4".into(), Vram::new(0x8003A7E4)),
        UserSymbol::new("func_8003A89C".into(), Vram::new(0x8003A89C)),
        UserSymbol::new("func_8003A8F4".into(), Vram::new(0x8003A8F4)),
        UserSymbol::new("func_8003A950".into(), Vram::new(0x8003A950)),
        UserSymbol::new("initEtcWork".into(), Vram::new(0x8003A9A0)),
        UserSymbol::new("init_pause_disp".into(), Vram::new(0x8003ACB4)),
        UserSymbol::new("disp_logo_setup".into(), Vram::new(0x8003AD88)),
        UserSymbol::new("disp_count_logo".into(), Vram::new(0x8003AE08)),
        UserSymbol::new("disp_clear_logo".into(), Vram::new(0x8003AEE4)),
        UserSymbol::new("disp_allclear_logo".into(), Vram::new(0x8003B054)),
        UserSymbol::new("disp_win_logo".into(), Vram::new(0x8003B1C4)),
        UserSymbol::new("disp_lose_logo".into(), Vram::new(0x8003B30C)),
        UserSymbol::new("disp_draw_logo".into(), Vram::new(0x8003B454)),
        UserSymbol::new("disp_pause_logo".into(), Vram::new(0x8003B59C)),
        UserSymbol::new("etc_continue_logo".into(), Vram::new(0x8003BA98)),
        UserSymbol::new("disp_continue_logo".into(), Vram::new(0x8003BEF8)),
        UserSymbol::new("disp_continue_logo_score".into(), Vram::new(0x8003BFD8)),
        UserSymbol::new("disp_gameover_logo".into(), Vram::new(0x8003C094)),
        UserSymbol::new("disp_timeover_logo".into(), Vram::new(0x8003C1A4)),
        UserSymbol::new("disp_retire_logo".into(), Vram::new(0x8003C2B4)),
        UserSymbol::new("disp_timestop_logo".into(), Vram::new(0x8003C3C0)),
        UserSymbol::new("add_attack_effect".into(), Vram::new(0x8003C600)),
        UserSymbol::new("disp_attack_effect".into(), Vram::new(0x8003C6F4)),
        UserSymbol::new("Printer_Open".into(), Vram::new(0x8003CDA0)),
        UserSymbol::new("Printer_Close".into(), Vram::new(0x8003CDD4)),
        UserSymbol::new("Printer_PutChars".into(), Vram::new(0x8003CDF8)),
        UserSymbol::new("cvt_radix".into(), Vram::new(0x8003CE20)),
        UserSymbol::new("cvtl_radix".into(), Vram::new(0x8003CE5C)),
        UserSymbol::new("round_asc".into(), Vram::new(0x8003CF2C)),
        UserSymbol::new("eprt_sub".into(), Vram::new(0x8003CFA8)),
        UserSymbol::new("ecvt".into(), Vram::new(0x8003D110)),
        UserSymbol::new("fcvt".into(), Vram::new(0x8003D41C)),
        UserSymbol::new("gcvt".into(), Vram::new(0x8003D4C8)),
        UserSymbol::new("_kmcprt".into(), Vram::new(0x8003D6D0)),
        UserSymbol::new("DebugPrint_8003E1E0".into(), Vram::new(0x8003E1E0)),
        UserSymbol::new("DebugPrint_SetColor".into(), Vram::new(0x8003E208)),
        UserSymbol::new("DebugPrint_8003E278".into(), Vram::new(0x8003E278)),
        UserSymbol::new("DebugPrint_DrawCharacter".into(), Vram::new(0x8003E3F0)),
        UserSymbol::new("DebugPrint_CallbackOpen".into(), Vram::new(0x8003E430)),
        UserSymbol::new("DebugPrint_CallbackClose".into(), Vram::new(0x8003E4AC)),
        UserSymbol::new("DebugPrint_CallbackPutChars".into(), Vram::new(0x8003E4B4)),
        UserSymbol::new("DebugPrint_Open".into(), Vram::new(0x8003E69C)),
        UserSymbol::new("DebugPrint_Close".into(), Vram::new(0x8003E6D8)),
        UserSymbol::new("DebugPrint_Printf".into(), Vram::new(0x8003E6F8)),
        UserSymbol::new("DebugMenu_8003E730".into(), Vram::new(0x8003E730)),
        UserSymbol::new(
            "DebugMenu_Page_SpeedMenu_Draw".into(),
            Vram::new(0x8003E8C8),
        ),
        UserSymbol::new(
            "DebugMenu_Page_LogicMenu_RetrieveStats".into(),
            Vram::new(0x8003EB20),
        ),
        UserSymbol::new(
            "DebugMenu_Page_LogicMenu_ApplyStats".into(),
            Vram::new(0x8003ECDC),
        ),
        UserSymbol::new("DebugMenu_8003EEA4".into(), Vram::new(0x8003EEA4)),
        UserSymbol::new(
            "DebugMenu_Page_LogicMenu_Draw".into(),
            Vram::new(0x8003F050),
        ),
        UserSymbol::new(
            "DebugMenu_Page_CharacterMenu_RetrieveStats".into(),
            Vram::new(0x8003F360),
        ),
        UserSymbol::new(
            "DebugMenu_Page_CharacterMenu_ApplyStats".into(),
            Vram::new(0x8003F474),
        ),
        UserSymbol::new("DebugMenu_8003F568".into(), Vram::new(0x8003F568)),
        UserSymbol::new(
            "DebugMenu_Page_CharacterMenu_Draw".into(),
            Vram::new(0x8003F7DC),
        ),
        UserSymbol::new("DebugMenu_Settings_Update".into(), Vram::new(0x8003FB00)),
        UserSymbol::new("DebugMenu_Settings_Draw".into(), Vram::new(0x8003FD0C)),
        UserSymbol::new(
            "DebugMenu_Page_CharacterEdit_Callback".into(),
            Vram::new(0x8003FD88),
        ),
        UserSymbol::new(
            "DebugMenu_GetCharacterStrength".into(),
            Vram::new(0x8003FEE4),
        ),
        UserSymbol::new(
            "DebugMenu_Page_CharacterEdit_RetrieveStats".into(),
            Vram::new(0x8004015C),
        ),
        UserSymbol::new(
            "DebugMenu_Page_CharacterEdit_ApplyStats".into(),
            Vram::new(0x80040238),
        ),
        UserSymbol::new(
            "DebugMenu_Page_CharacterEdit_UpdateController".into(),
            Vram::new(0x80040578),
        ),
        UserSymbol::new(
            "DebugMenu_Page_CharacterEdit_Draw".into(),
            Vram::new(0x80040624),
        ),
        UserSymbol::new(
            "DebugMenu_CharacterEdit_Update".into(),
            Vram::new(0x800409A0),
        ),
        UserSymbol::new("DebugMenu_CharacterEdit_Draw".into(), Vram::new(0x800409DC)),
        UserSymbol::new("BgTasksManager_ThreadEntry".into(), Vram::new(0x80040A00)),
        UserSymbol::new("BgTasksManager_IsInitialized".into(), Vram::new(0x80040A58)),
        UserSymbol::new("BgTasksManager_Init".into(), Vram::new(0x80040A64)),
        UserSymbol::new("BgTasksManager_Destroy".into(), Vram::new(0x80040AE4)),
        UserSymbol::new("BgTasksManager_SendTask".into(), Vram::new(0x80040B10)),
        UserSymbol::new(
            "BgTasksManager_GetRemainingTasks".into(),
            Vram::new(0x80040BA4),
        ),
        UserSymbol::new("gfxSetScissor".into(), Vram::new(0x80040BB0)),
        UserSymbol::new("func_80040D34".into(), Vram::new(0x80040D34)),
        UserSymbol::new("FillRectRGBA".into(), Vram::new(0x80040E3C)),
        UserSymbol::new("CopyTexBlock".into(), Vram::new(0x80040F34)),
        UserSymbol::new("CopyTexBlock4_LoadTex".into(), Vram::new(0x8004113C)),
        UserSymbol::new("CopyTexBlock4".into(), Vram::new(0x80041334)),
        UserSymbol::new("CopyTexBlock8_LoadTex".into(), Vram::new(0x80041480)),
        UserSymbol::new("CopyTexBlock8".into(), Vram::new(0x80041668)),
        UserSymbol::new("CopyTexBlock16_LoadTex".into(), Vram::new(0x800417B4)),
        UserSymbol::new("CopyTexBlock16".into(), Vram::new(0x8004199C)),
        UserSymbol::new("StretchTexBlock".into(), Vram::new(0x80041A54)),
        UserSymbol::new("StretchTexBlock_ScisRect".into(), Vram::new(0x80041DE0)),
        UserSymbol::new(
            "StretchAlphaTexBlock_LoadColorTex".into(),
            Vram::new(0x80041F94),
        ),
        UserSymbol::new(
            "StretchAlphaTexBlock_LoadAlphaTex".into(),
            Vram::new(0x8004216C),
        ),
        UserSymbol::new("StretchAlphaTexBlock".into(), Vram::new(0x80042364)),
        UserSymbol::new("StretchTexBlock4_LoadTex".into(), Vram::new(0x80042468)),
        UserSymbol::new("StretchTexBlock4".into(), Vram::new(0x80042650)),
        UserSymbol::new("StretchTexBlock8_LoadTex".into(), Vram::new(0x800427E0)),
        UserSymbol::new("StretchTexBlock8".into(), Vram::new(0x800429B8)),
        UserSymbol::new("StretchTexBlock16_LoadTex".into(), Vram::new(0x80042B48)),
        UserSymbol::new("StretchTexBlock16".into(), Vram::new(0x80042D20)),
        UserSymbol::new("StretchTexBlock4i_LoadTex".into(), Vram::new(0x80042E04)),
        UserSymbol::new("StretchTexBlock4i".into(), Vram::new(0x80042FEC)),
        UserSymbol::new("func_800430D0".into(), Vram::new(0x800430D0)),
        UserSymbol::new("func_800432A8".into(), Vram::new(0x800432A8)),
        UserSymbol::new("StretchTexTile".into(), Vram::new(0x8004338C)),
        UserSymbol::new("StretchTexTile_ScisRect".into(), Vram::new(0x8004371C)),
        UserSymbol::new("RectTexTile_ScisRect".into(), Vram::new(0x800438D0)),
        UserSymbol::new(
            "StretchAlphaTexTile_LoadColorTex".into(),
            Vram::new(0x80043A14),
        ),
        UserSymbol::new(
            "StretchAlphaTexTile_LoadAlphaTex".into(),
            Vram::new(0x80043BEC),
        ),
        UserSymbol::new("StretchAlphaTexTile".into(), Vram::new(0x80043DDC)),
        UserSymbol::new("RectAlphaTexTile".into(), Vram::new(0x80043F18)),
        UserSymbol::new("StretchTexTile4_LoadTex".into(), Vram::new(0x80044058)),
        UserSymbol::new("StretchTexTile4".into(), Vram::new(0x80044238)),
        UserSymbol::new("StretchTexTile8_LoadTex".into(), Vram::new(0x800443F4)),
        UserSymbol::new("StretchTexTile8".into(), Vram::new(0x800445C4)),
        UserSymbol::new("RectTexTile8".into(), Vram::new(0x80044780)),
        UserSymbol::new("StretchTexTile16_LoadTex".into(), Vram::new(0x80044940)),
        UserSymbol::new("StretchTexTile16".into(), Vram::new(0x80044B18)),
        UserSymbol::new("StretchTexTile4i_LoadTex".into(), Vram::new(0x80044C28)),
        UserSymbol::new("StretchTexTile4i".into(), Vram::new(0x80044E08)),
        UserSymbol::new("RectTexTile4i".into(), Vram::new(0x80044F18)),
        UserSymbol::new("tiMappingAddr".into(), Vram::new(0x8004502C)),
        UserSymbol::new("tiLoadTexData".into(), Vram::new(0x80045098)),
        UserSymbol::new("func_80045110".into(), Vram::new(0x80045110)),
        UserSymbol::new("tiCopyTexBlock".into(), Vram::new(0x800451C4)),
        UserSymbol::new("tiStretchTexBlock".into(), Vram::new(0x800452B0)),
        UserSymbol::new("tiStretchTexTile".into(), Vram::new(0x800453D8)),
        UserSymbol::new("tiStretchTexItem".into(), Vram::new(0x8004554C)),
        UserSymbol::new("tiStretchAlphaTexItem".into(), Vram::new(0x80045748)),
        UserSymbol::new("func_80045914".into(), Vram::new(0x80045914)),
        UserSymbol::new("drawCursorPattern".into(), Vram::new(0x80045AAC)),
        UserSymbol::new("func_80045DD0".into(), Vram::new(0x80045DD0)),
        UserSymbol::new("func_80045E18".into(), Vram::new(0x80045E18)),
        UserSymbol::new("func_80045E48".into(), Vram::new(0x80045E48)),
        UserSymbol::new("func_80045E94".into(), Vram::new(0x80045E94)),
        UserSymbol::new("transFunc_curve".into(), Vram::new(0x80045EF4)),
        UserSymbol::new("transFunc_finger".into(), Vram::new(0x80045F68)),
        UserSymbol::new("func_80046008".into(), Vram::new(0x80046008)),
        UserSymbol::new("scaleFunc_curve".into(), Vram::new(0x80046068)),
        UserSymbol::new("func_800460DC".into(), Vram::new(0x800460DC)),
        UserSymbol::new("colorFunc_curve".into(), Vram::new(0x8004613C)),
        UserSymbol::new("colorFunc_cursor".into(), Vram::new(0x800461B0)),
        UserSymbol::new("menuItem_init".into(), Vram::new(0x80046250)),
        UserSymbol::new("menuItem_updateTransScale".into(), Vram::new(0x80046368)),
        UserSymbol::new("menuItem_updateColor".into(), Vram::new(0x80046408)),
        UserSymbol::new("func_800464BC".into(), Vram::new(0x800464BC)),
        UserSymbol::new("func_800464F8".into(), Vram::new(0x800464F8)),
        UserSymbol::new("func_8004655C".into(), Vram::new(0x8004655C)),
        UserSymbol::new("func_800465B8".into(), Vram::new(0x800465B8)),
        UserSymbol::new("func_80046614".into(), Vram::new(0x80046614)),
        UserSymbol::new("func_80046670".into(), Vram::new(0x80046670)),
        UserSymbol::new("func_8004667C".into(), Vram::new(0x8004667C)),
        UserSymbol::new("func_80046688".into(), Vram::new(0x80046688)),
        UserSymbol::new("func_80046694".into(), Vram::new(0x80046694)),
        UserSymbol::new("func_800466A0".into(), Vram::new(0x800466A0)),
        UserSymbol::new("func_800466B8".into(), Vram::new(0x800466B8)),
        UserSymbol::new("func_800466D0".into(), Vram::new(0x800466D0)),
        UserSymbol::new("func_80046734".into(), Vram::new(0x80046734)),
        UserSymbol::new("func_800467E0".into(), Vram::new(0x800467E0)),
        UserSymbol::new("func_80046844".into(), Vram::new(0x80046844)),
        UserSymbol::new("menuItem_outOfScreen".into(), Vram::new(0x80046A3C)),
        UserSymbol::new("menuItem_drawTex".into(), Vram::new(0x80046B18)),
        UserSymbol::new("func_80046C74".into(), Vram::new(0x80046C74)),
        UserSymbol::new("menuItem_drawAlphaTex".into(), Vram::new(0x80046D3C)),
        UserSymbol::new("func_80046F58".into(), Vram::new(0x80046F58)),
        UserSymbol::new("func_80047074".into(), Vram::new(0x80047074)),
        UserSymbol::new("func_8004714C".into(), Vram::new(0x8004714C)),
        UserSymbol::new("func_800472D0".into(), Vram::new(0x800472D0)),
        UserSymbol::new("menuTitle_setTitle".into(), Vram::new(0x80047420)),
        UserSymbol::new("func_800474EC".into(), Vram::new(0x800474EC)),
        UserSymbol::new("func_80047584".into(), Vram::new(0x80047584)),
        UserSymbol::new("menuTitle_draw".into(), Vram::new(0x800475A8)),
        UserSymbol::new("func_80047720".into(), Vram::new(0x80047720)),
        UserSymbol::new("func_800477BC".into(), Vram::new(0x800477BC)),
        UserSymbol::new("menuCursor_init2".into(), Vram::new(0x80047828)),
        UserSymbol::new("func_800479A8".into(), Vram::new(0x800479A8)),
        UserSymbol::new("menuCursor_update".into(), Vram::new(0x800479EC)),
        UserSymbol::new("menuCursor_draw1".into(), Vram::new(0x80047AA4)),
        UserSymbol::new("menuCursor_draw2".into(), Vram::new(0x80048444)),
        UserSymbol::new("func_80048634".into(), Vram::new(0x80048634)),
        UserSymbol::new("func_80048680".into(), Vram::new(0x80048680)),
        UserSymbol::new("func_800486C8".into(), Vram::new(0x800486C8)),
        UserSymbol::new("menuBottle_draw".into(), Vram::new(0x80048708)),
        UserSymbol::new("func_80048B8C".into(), Vram::new(0x80048B8C)),
        UserSymbol::new("func_80048C48".into(), Vram::new(0x80048C48)),
        UserSymbol::new("func_80048CC8".into(), Vram::new(0x80048CC8)),
        UserSymbol::new("menuLvGauge_draw1".into(), Vram::new(0x80048D34)),
        UserSymbol::new("func_80048FA0".into(), Vram::new(0x80048FA0)),
        UserSymbol::new("func_80049034".into(), Vram::new(0x80049034)),
        UserSymbol::new("func_80049080".into(), Vram::new(0x80049080)),
        UserSymbol::new("func_800490B8".into(), Vram::new(0x800490B8)),
        UserSymbol::new("menuYN_init".into(), Vram::new(0x800490D4)),
        UserSymbol::new("menuYN_input".into(), Vram::new(0x80049224)),
        UserSymbol::new("func_800492D8".into(), Vram::new(0x800492D8)),
        UserSymbol::new("menuYN_draw".into(), Vram::new(0x80049348)),
        UserSymbol::new("func_80049540".into(), Vram::new(0x80049540)),
        UserSymbol::new("func_80049578".into(), Vram::new(0x80049578)),
        UserSymbol::new("menuMes_init".into(), Vram::new(0x80049594)),
        UserSymbol::new("func_8004970C".into(), Vram::new(0x8004970C)),
        UserSymbol::new("func_80049754".into(), Vram::new(0x80049754)),
        UserSymbol::new("func_800497D0".into(), Vram::new(0x800497D0)),
        UserSymbol::new("func_80049894".into(), Vram::new(0x80049894)),
        UserSymbol::new("func_800498C4".into(), Vram::new(0x800498C4)),
        UserSymbol::new("menuSpeedAsk_draw".into(), Vram::new(0x800498E0)),
        UserSymbol::new("menuSpeedItem_init".into(), Vram::new(0x80049A5C)),
        UserSymbol::new("func_80049C54".into(), Vram::new(0x80049C54)),
        UserSymbol::new("menuSpeedItem_update".into(), Vram::new(0x80049CD4)),
        UserSymbol::new("menuSpeedItem_draw1".into(), Vram::new(0x80049EA4)),
        UserSymbol::new("func_8004A160".into(), Vram::new(0x8004A160)),
        UserSymbol::new("func_8004A1F4".into(), Vram::new(0x8004A1F4)),
        UserSymbol::new("menuMusicItem_init".into(), Vram::new(0x8004A240)),
        UserSymbol::new("menuMusicItem_input".into(), Vram::new(0x8004A3F4)),
        UserSymbol::new("menuMusicItem_update".into(), Vram::new(0x8004A4CC)),
        UserSymbol::new("menuMusicItem_draw1".into(), Vram::new(0x8004A680)),
        UserSymbol::new("menuMusicItem_draw2".into(), Vram::new(0x8004A780)),
        UserSymbol::new("func_8004A814".into(), Vram::new(0x8004A814)),
        UserSymbol::new("func_8004A860".into(), Vram::new(0x8004A860)),
        UserSymbol::new("menuNumber_update".into(), Vram::new(0x8004A8D8)),
        UserSymbol::new("menuNumber_draw".into(), Vram::new(0x8004A94C)),
        UserSymbol::new("func_8004AC98".into(), Vram::new(0x8004AC98)),
        UserSymbol::new("func_8004ACF4".into(), Vram::new(0x8004ACF4)),
        UserSymbol::new("func_8004AD10".into(), Vram::new(0x8004AD10)),
        UserSymbol::new("func_8004AD3C".into(), Vram::new(0x8004AD3C)),
        UserSymbol::new("func_8004AD84".into(), Vram::new(0x8004AD84)),
        UserSymbol::new("menuComLvPanel_draw".into(), Vram::new(0x8004ADC4)),
        UserSymbol::new("menuCont_setFade".into(), Vram::new(0x8004B114)),
        UserSymbol::new("func_8004B2C8".into(), Vram::new(0x8004B2C8)),
        UserSymbol::new("menuCont_init".into(), Vram::new(0x8004B318)),
        UserSymbol::new("func_8004B43C".into(), Vram::new(0x8004B43C)),
        UserSymbol::new("func_8004B488".into(), Vram::new(0x8004B488)),
        UserSymbol::new("menuCont_draw".into(), Vram::new(0x8004B500)),
        UserSymbol::new("func_8004B774".into(), Vram::new(0x8004B774)),
        UserSymbol::new("menuMainPanel_init".into(), Vram::new(0x8004B7B8)),
        UserSymbol::new("menuMainPanel_input".into(), Vram::new(0x8004B8CC)),
        UserSymbol::new("func_8004B98C".into(), Vram::new(0x8004B98C)),
        UserSymbol::new("menuMainPanel_draw".into(), Vram::new(0x8004B9F0)),
        UserSymbol::new("func_8004BB14".into(), Vram::new(0x8004BB14)),
        UserSymbol::new("menuNameSelPanel_clear".into(), Vram::new(0x8004BB58)),
        UserSymbol::new("menuNameSelPanel_init".into(), Vram::new(0x8004BC2C)),
        UserSymbol::new("func_8004BEB4".into(), Vram::new(0x8004BEB4)),
        UserSymbol::new("menuNameSelPanel_input1".into(), Vram::new(0x8004BF10)),
        UserSymbol::new("menuNameSelPanel_input".into(), Vram::new(0x8004C1F0)),
        UserSymbol::new("menuNameSelPanel_update".into(), Vram::new(0x8004C2E8)),
        UserSymbol::new("menuNameSelPanel_draw".into(), Vram::new(0x8004C4BC)),
        UserSymbol::new("func_8004C820".into(), Vram::new(0x8004C820)),
        UserSymbol::new("menuNameOpPanel_init".into(), Vram::new(0x8004C864)),
        UserSymbol::new("menuNameOpPanel_input".into(), Vram::new(0x8004C974)),
        UserSymbol::new("func_8004CA30".into(), Vram::new(0x8004CA30)),
        UserSymbol::new("menuNameOpPanel_draw".into(), Vram::new(0x8004CAA0)),
        UserSymbol::new("func_8004CCD0".into(), Vram::new(0x8004CCD0)),
        UserSymbol::new("menuSndSelPanel_init".into(), Vram::new(0x8004CD14)),
        UserSymbol::new("menuSndSelPanel_input".into(), Vram::new(0x8004CE48)),
        UserSymbol::new("func_8004CFB8".into(), Vram::new(0x8004CFB8)),
        UserSymbol::new("menuSndSelPanel_draw".into(), Vram::new(0x8004D070)),
        UserSymbol::new("func_8004D258".into(), Vram::new(0x8004D258)),
        UserSymbol::new("menuPlay2Panel_init".into(), Vram::new(0x8004D2A0)),
        UserSymbol::new("menuPlay2Panel_copyConfig".into(), Vram::new(0x8004D6DC)),
        UserSymbol::new("menuPlay2Panel_copyCursor".into(), Vram::new(0x8004D734)),
        UserSymbol::new("menuPlay2Panel_input".into(), Vram::new(0x8004D828)),
        UserSymbol::new("menuPlay2Panel_update".into(), Vram::new(0x8004DA40)),
        UserSymbol::new("menuPlay2Panel_draw".into(), Vram::new(0x8004DD14)),
        UserSymbol::new("func_8004E270".into(), Vram::new(0x8004E270)),
        UserSymbol::new("func_8004E2B4".into(), Vram::new(0x8004E2B4)),
        UserSymbol::new("menuPlay2PanelSub_init".into(), Vram::new(0x8004E2F4)),
        UserSymbol::new("menuPlay2PanelSub_input".into(), Vram::new(0x8004E4F4)),
        UserSymbol::new("menuPlay2PanelSub_update".into(), Vram::new(0x8004E764)),
        UserSymbol::new("menuPlay2PanelSub_draw".into(), Vram::new(0x8004E8E0)),
        UserSymbol::new("menuMain_setMsgStr".into(), Vram::new(0x8004EB94)),
        UserSymbol::new("menuMain_initPanel".into(), Vram::new(0x8004ED70)),
        UserSymbol::new("menuMain_init".into(), Vram::new(0x8004EEC8)),
        UserSymbol::new("func_8004F2D8".into(), Vram::new(0x8004F2D8)),
        UserSymbol::new("func_8004F33C".into(), Vram::new(0x8004F33C)),
        UserSymbol::new("menuMain_input".into(), Vram::new(0x8004F358)),
        UserSymbol::new("menuMain_update".into(), Vram::new(0x800508BC)),
        UserSymbol::new("menuMain_drawKaSaMaRu".into(), Vram::new(0x80050B5C)),
        UserSymbol::new("menuMain_draw".into(), Vram::new(0x800512AC)),
        UserSymbol::new("func_800513F0".into(), Vram::new(0x800513F0)),
        UserSymbol::new("func_80051480".into(), Vram::new(0x80051480)),
        UserSymbol::new("func_800514C4".into(), Vram::new(0x800514C4)),
        UserSymbol::new("menuStory_init".into(), Vram::new(0x80051540)),
        UserSymbol::new("func_80051974".into(), Vram::new(0x80051974)),
        UserSymbol::new("func_800519CC".into(), Vram::new(0x800519CC)),
        UserSymbol::new("func_800519EC".into(), Vram::new(0x800519EC)),
        UserSymbol::new("menuStory_input".into(), Vram::new(0x80051A28)),
        UserSymbol::new("menuStory_update".into(), Vram::new(0x80051E94)),
        UserSymbol::new("menuStory_draw".into(), Vram::new(0x800522CC)),
        UserSymbol::new("func_800529FC".into(), Vram::new(0x800529FC)),
        UserSymbol::new("menuLvSel_init".into(), Vram::new(0x80052A40)),
        UserSymbol::new("func_80052DF0".into(), Vram::new(0x80052DF0)),
        UserSymbol::new("menuLvSel_input".into(), Vram::new(0x80052E48)),
        UserSymbol::new("menuLvSel_update".into(), Vram::new(0x80053194)),
        UserSymbol::new("menuLvSel_draw".into(), Vram::new(0x8005348C)),
        UserSymbol::new("func_8005380C".into(), Vram::new(0x8005380C)),
        UserSymbol::new("menuChSel_init".into(), Vram::new(0x80053850)),
        UserSymbol::new("menuChSel_checkSelected".into(), Vram::new(0x80053C2C)),
        UserSymbol::new("func_80053C84".into(), Vram::new(0x80053C84)),
        UserSymbol::new("menuChSel_input1".into(), Vram::new(0x80053CEC)),
        UserSymbol::new("menuChSel_inputMan".into(), Vram::new(0x80053FBC)),
        UserSymbol::new("menuChSel_inputCom".into(), Vram::new(0x800540B8)),
        UserSymbol::new("menuChSel_input".into(), Vram::new(0x800541CC)),
        UserSymbol::new("menuChSel_update".into(), Vram::new(0x800544C4)),
        UserSymbol::new("menuChSel_draw".into(), Vram::new(0x80054734)),
        UserSymbol::new("func_80054A94".into(), Vram::new(0x80054A94)),
        UserSymbol::new("menuPlay2_init".into(), Vram::new(0x80054B60)),
        UserSymbol::new("func_800550F4".into(), Vram::new(0x800550F4)),
        UserSymbol::new("func_80055154".into(), Vram::new(0x80055154)),
        UserSymbol::new("func_800551BC".into(), Vram::new(0x800551BC)),
        UserSymbol::new("func_80055254".into(), Vram::new(0x80055254)),
        UserSymbol::new("menuPlay2_inputMan".into(), Vram::new(0x800552EC)),
        UserSymbol::new("menuPlay2_inputCpu".into(), Vram::new(0x80055598)),
        UserSymbol::new("menuPlay2_input".into(), Vram::new(0x800556E0)),
        UserSymbol::new("menuPlay2_update".into(), Vram::new(0x80055C08)),
        UserSymbol::new("menuPlay2_draw".into(), Vram::new(0x80055CE0)),
        UserSymbol::new("func_80055DFC".into(), Vram::new(0x80055DFC)),
        UserSymbol::new("menuNmEnt_init".into(), Vram::new(0x80055E40)),
        UserSymbol::new("func_800560D4".into(), Vram::new(0x800560D4)),
        UserSymbol::new("menuNmEnt_input".into(), Vram::new(0x800560F0)),
        UserSymbol::new("menuNmEnt_update".into(), Vram::new(0x80056824)),
        UserSymbol::new("menuNmEnt_draw".into(), Vram::new(0x80056990)),
        UserSymbol::new("func_80056C84".into(), Vram::new(0x80056C84)),
        UserSymbol::new("func_80056CAC".into(), Vram::new(0x80056CAC)),
        UserSymbol::new("menuRankBase_draw".into(), Vram::new(0x80056CC8)),
        UserSymbol::new("func_80056DF0".into(), Vram::new(0x80056DF0)),
        UserSymbol::new("func_80056E1C".into(), Vram::new(0x80056E1C)),
        UserSymbol::new("menuRankNum_draw".into(), Vram::new(0x80056E38)),
        UserSymbol::new("menuRankFig_init".into(), Vram::new(0x80057014)),
        UserSymbol::new("menuRankFig_update".into(), Vram::new(0x8005712C)),
        UserSymbol::new("menuRankFig_draw".into(), Vram::new(0x800574B4)),
        UserSymbol::new("menuRankName_init".into(), Vram::new(0x8005768C)),
        UserSymbol::new("func_80057724".into(), Vram::new(0x80057724)),
        UserSymbol::new("menuRankName_draw".into(), Vram::new(0x80057740)),
        UserSymbol::new("func_80057898".into(), Vram::new(0x80057898)),
        UserSymbol::new("func_800578C8".into(), Vram::new(0x800578C8)),
        UserSymbol::new("menuRankLabel_draw".into(), Vram::new(0x800578E4)),
        UserSymbol::new("func_80057AFC".into(), Vram::new(0x80057AFC)),
        UserSymbol::new("func_80057BE8".into(), Vram::new(0x80057BE8)),
        UserSymbol::new("menuRankHeader_draw".into(), Vram::new(0x80057C50)),
        UserSymbol::new("func_80057D24".into(), Vram::new(0x80057D24)),
        UserSymbol::new("func_80057E68".into(), Vram::new(0x80057E68)),
        UserSymbol::new("func_80057F6C".into(), Vram::new(0x80057F6C)),
        UserSymbol::new("func_8005806C".into(), Vram::new(0x8005806C)),
        UserSymbol::new("func_800581C8".into(), Vram::new(0x800581C8)),
        UserSymbol::new("menuRankPanel_update".into(), Vram::new(0x800582FC)),
        UserSymbol::new("menuRankPanel_draw".into(), Vram::new(0x800583C4)),
        UserSymbol::new("menuRank_setNameBaseScale".into(), Vram::new(0x800585BC)),
        UserSymbol::new("menuRank_setSlide".into(), Vram::new(0x800586A4)),
        UserSymbol::new("menuRank_setFrame".into(), Vram::new(0x80058838)),
        UserSymbol::new("func_80058A24".into(), Vram::new(0x80058A24)),
        UserSymbol::new("menuRank_setPanel".into(), Vram::new(0x80058A9C)),
        UserSymbol::new("menuRank_init".into(), Vram::new(0x8005911C)),
        UserSymbol::new("menuRank_input".into(), Vram::new(0x800592D4)),
        UserSymbol::new("menuRank_update".into(), Vram::new(0x8005954C)),
        UserSymbol::new("menuRank_draw".into(), Vram::new(0x800596B4)),
        UserSymbol::new("_eep_writingCallback".into(), Vram::new(0x800599E0)),
        UserSymbol::new("func_80059A58".into(), Vram::new(0x80059A58)),
        UserSymbol::new("func_80059AA4".into(), Vram::new(0x80059AA4)),
        UserSymbol::new("func_80059AF0".into(), Vram::new(0x80059AF0)),
        UserSymbol::new("_eepWritePlayer".into(), Vram::new(0x80059B5C)),
        UserSymbol::new("_eepErasePlayer".into(), Vram::new(0x80059BC8)),
        UserSymbol::new("_eepEraseData".into(), Vram::new(0x80059C34)),
        UserSymbol::new("_waitRetrace".into(), Vram::new(0x80059CA0)),
        UserSymbol::new("_getMtxPtr".into(), Vram::new(0x80059CC4)),
        UserSymbol::new("_getVtxPtr".into(), Vram::new(0x80059CDC)),
        UserSymbol::new("_getTexChar".into(), Vram::new(0x80059CF4)),
        UserSymbol::new("_getTexCommon".into(), Vram::new(0x80059D04)),
        UserSymbol::new("func_80059D14".into(), Vram::new(0x80059D14)),
        UserSymbol::new("_getTexLevel".into(), Vram::new(0x80059D24)),
        UserSymbol::new("_getTexMain".into(), Vram::new(0x80059D34)),
        UserSymbol::new("_getTexName".into(), Vram::new(0x80059D44)),
        UserSymbol::new("_getTexP2".into(), Vram::new(0x80059D54)),
        UserSymbol::new("_getTexP4".into(), Vram::new(0x80059D64)),
        UserSymbol::new("_getTexRank".into(), Vram::new(0x80059D74)),
        UserSymbol::new("_getTexSetup".into(), Vram::new(0x80059D84)),
        UserSymbol::new("_getTexStory".into(), Vram::new(0x80059D94)),
        UserSymbol::new("_getTexGameAl".into(), Vram::new(0x80059DA4)),
        UserSymbol::new("_getTexGameP1".into(), Vram::new(0x80059DB4)),
        UserSymbol::new("_getTexCont".into(), Vram::new(0x80059DC4)),
        UserSymbol::new("_getTexTutol".into(), Vram::new(0x80059DD4)),
        UserSymbol::new("_getTexKasa".into(), Vram::new(0x80059DE4)),
        UserSymbol::new("_getRootItem".into(), Vram::new(0x80059DF4)),
        UserSymbol::new("_getKeyLvl".into(), Vram::new(0x80059DFC)),
        UserSymbol::new("_getKeyTrg".into(), Vram::new(0x80059E1C)),
        UserSymbol::new("_getKeyRep".into(), Vram::new(0x80059E3C)),
        UserSymbol::new("_getMode".into(), Vram::new(0x80059E5C)),
        UserSymbol::new("_getModeOld".into(), Vram::new(0x80059E6C)),
        UserSymbol::new("_setMode".into(), Vram::new(0x80059E7C)),
        UserSymbol::new("_setNextMain".into(), Vram::new(0x80059E8C)),
        UserSymbol::new("_setTitle".into(), Vram::new(0x80059E9C)),
        UserSymbol::new("_setFadeDir".into(), Vram::new(0x80059EB8)),
        UserSymbol::new("menuAll_init".into(), Vram::new(0x80059F1C)),
        UserSymbol::new("func_8005A2AC".into(), Vram::new(0x8005A2AC)),
        UserSymbol::new("func_8005A2EC".into(), Vram::new(0x8005A2EC)),
        UserSymbol::new("menuAll_changeMenu".into(), Vram::new(0x8005A434)),
        UserSymbol::new("menuAll_input".into(), Vram::new(0x8005A720)),
        UserSymbol::new("menuAll_update".into(), Vram::new(0x8005A974)),
        UserSymbol::new("menuAll_drawBg".into(), Vram::new(0x8005AD30)),
        UserSymbol::new("menuAll_draw".into(), Vram::new(0x8005AEF4)),
        UserSymbol::new("main_menu".into(), Vram::new(0x8005B2D4)),
        UserSymbol::new("graphic_menu".into(), Vram::new(0x8005B658)),
        UserSymbol::new("fontStr_nextChar".into(), Vram::new(0x8005B7F0)),
        UserSymbol::new("fontStr_length".into(), Vram::new(0x8005B858)),
        UserSymbol::new("fontStr_charSize".into(), Vram::new(0x8005B898)),
        UserSymbol::new("func_8005B8D8".into(), Vram::new(0x8005B8D8)),
        UserSymbol::new("font2index".into(), Vram::new(0x8005B8F0)),
        UserSymbol::new("ascii2index".into(), Vram::new(0x8005B940)),
        UserSymbol::new("font16_initDL".into(), Vram::new(0x8005B974)),
        UserSymbol::new("font16_initDL2".into(), Vram::new(0x8005B998)),
        UserSymbol::new("fontXX_draw".into(), Vram::new(0x8005BAAC)),
        UserSymbol::new("fontXX_drawID".into(), Vram::new(0x8005BB24)),
        UserSymbol::new("fontXX_draw2".into(), Vram::new(0x8005BEA0)),
        UserSymbol::new("fontXX_drawID2".into(), Vram::new(0x8005BF18)),
        UserSymbol::new("fontAsc_draw".into(), Vram::new(0x8005C4BC)),
        UserSymbol::new("fontAsc_drawID".into(), Vram::new(0x8005C548)),
        UserSymbol::new("fontAsc_draw2".into(), Vram::new(0x8005C90C)),
        UserSymbol::new("fontAsc_drawID2".into(), Vram::new(0x8005C998)),
        UserSymbol::new("func_8005CF20".into(), Vram::new(0x8005CF20)),
        UserSymbol::new("msgWnd_init".into(), Vram::new(0x8005CF78)),
        UserSymbol::new("msgWnd_init2".into(), Vram::new(0x8005CFD4)),
        UserSymbol::new("msgWnd_clear".into(), Vram::new(0x8005D09C)),
        UserSymbol::new("msgWnd_layout".into(), Vram::new(0x8005D0EC)),
        UserSymbol::new("msgWnd_addStr".into(), Vram::new(0x8005D314)),
        UserSymbol::new("func_8005D3F8".into(), Vram::new(0x8005D3F8)),
        UserSymbol::new("msgWnd_update".into(), Vram::new(0x8005D428)),
        UserSymbol::new("msgWnd_draw".into(), Vram::new(0x8005D78C)),
        UserSymbol::new("msgWnd_isEnd".into(), Vram::new(0x8005E0B4)),
        UserSymbol::new("msgWnd_skip".into(), Vram::new(0x8005E0BC)),
        UserSymbol::new("msgWnd_isSpeaking".into(), Vram::new(0x8005E0D0)),
        UserSymbol::new("msgWnd_isScroll".into(), Vram::new(0x8005E0D8)),
        UserSymbol::new("msgWnd_getWidth".into(), Vram::new(0x8005E0EC)),
        UserSymbol::new("msgWnd_getHeight".into(), Vram::new(0x8005E104)),
        UserSymbol::new("animeSeq_init".into(), Vram::new(0x8005E120)),
        UserSymbol::new("func_8005E154".into(), Vram::new(0x8005E154)),
        UserSymbol::new("animeSeq_update".into(), Vram::new(0x8005E174)),
        UserSymbol::new("animeSeq_isEnd".into(), Vram::new(0x8005E32C)),
        UserSymbol::new("animeState_getDataSize".into(), Vram::new(0x8005E358)),
        UserSymbol::new("animeState_load".into(), Vram::new(0x8005E36C)),
        UserSymbol::new("animeState_init".into(), Vram::new(0x8005E40C)),
        UserSymbol::new("animeState_set".into(), Vram::new(0x8005E48C)),
        UserSymbol::new("animeState_update".into(), Vram::new(0x8005E4A8)),
        UserSymbol::new("animeState_isEnd".into(), Vram::new(0x8005E4E0)),
        UserSymbol::new("animeState_initDL".into(), Vram::new(0x8005E4FC)),
        UserSymbol::new("animeState_initDL2".into(), Vram::new(0x8005E5B0)),
        UserSymbol::new("animeState_initIntensityDL".into(), Vram::new(0x8005E67C)),
        UserSymbol::new("animeState_draw".into(), Vram::new(0x8005E750)),
        UserSymbol::new("func_8005E998".into(), Vram::new(0x8005E998)),
        UserSymbol::new("animeSmog_init".into(), Vram::new(0x8005EAFC)),
        UserSymbol::new("animeSmog_load".into(), Vram::new(0x8005EBA8)),
        UserSymbol::new("animeSmog_start".into(), Vram::new(0x8005EBDC)),
        UserSymbol::new("animeSmog_stop".into(), Vram::new(0x8005EBE4)),
        UserSymbol::new("animeSmog_update".into(), Vram::new(0x8005EC40)),
        UserSymbol::new("animeSmog_draw".into(), Vram::new(0x8005ED74)),
        UserSymbol::new("func_8005EE64".into(), Vram::new(0x8005EE64)),
        UserSymbol::new("loadAnimeSeq".into(), Vram::new(0x8005EE98)),
        UserSymbol::new("init_map_all".into(), Vram::new(0x8005EF50)),
        UserSymbol::new("clear_map".into(), Vram::new(0x8005EFC0)),
        UserSymbol::new("clear_map_all".into(), Vram::new(0x8005F00C)),
        UserSymbol::new("get_map_info".into(), Vram::new(0x8005F074)),
        UserSymbol::new("func_8005F09C".into(), Vram::new(0x8005F09C)),
        UserSymbol::new("func_8005F0E4".into(), Vram::new(0x8005F0E4)),
        UserSymbol::new("get_virus_color_count".into(), Vram::new(0x8005F13C)),
        UserSymbol::new("get_virus_count".into(), Vram::new(0x8005F1E0)),
        UserSymbol::new("set_map".into(), Vram::new(0x8005F204)),
        UserSymbol::new("set_virus".into(), Vram::new(0x8005F25C)),
        UserSymbol::new("func_8005F2B0".into(), Vram::new(0x8005F2B0)),
        UserSymbol::new("func_8005F310".into(), Vram::new(0x8005F310)),
        UserSymbol::new("dm_check_color".into(), Vram::new(0x8005F33C)),
        UserSymbol::new("dm_check_color_2".into(), Vram::new(0x8005F54C)),
        UserSymbol::new("dm_virus_map_copy".into(), Vram::new(0x8005F634)),
        UserSymbol::new("dm_get_first_virus_count".into(), Vram::new(0x8005F698)),
        UserSymbol::new(
            "dm_get_first_virus_count_in_new_mode".into(),
            Vram::new(0x8005F6F0),
        ),
        UserSymbol::new("_dm_virus_init".into(), Vram::new(0x8005F700)),
        UserSymbol::new("dm_virus_init".into(), Vram::new(0x8005FC50)),
        UserSymbol::new("func_8005FC6C".into(), Vram::new(0x8005FC6C)),
        UserSymbol::new("_makeFlash_checkOrdre".into(), Vram::new(0x8005FCC4)),
        UserSymbol::new("func_8005FE68".into(), Vram::new(0x8005FE68)),
        UserSymbol::new("make_flash_virus_pos".into(), Vram::new(0x8005FF00)),
        UserSymbol::new("func_800601F0".into(), Vram::new(0x800601F0)),
        UserSymbol::new("func_80060270".into(), Vram::new(0x80060270)),
        UserSymbol::new("dm_make_score".into(), Vram::new(0x800602B8)),
        UserSymbol::new("dm_attack_se".into(), Vram::new(0x80060348)),
        UserSymbol::new("dm_warning_h_line".into(), Vram::new(0x80060424)),
        UserSymbol::new("set_down_flg".into(), Vram::new(0x800604D8)),
        UserSymbol::new("go_down".into(), Vram::new(0x8006075C)),
        UserSymbol::new("erase_anime".into(), Vram::new(0x80060910)),
        UserSymbol::new("throw_rotate_capsel".into(), Vram::new(0x800609B8)),
        UserSymbol::new("translate_capsel".into(), Vram::new(0x80060A6C)),
        UserSymbol::new("rotate_capsel".into(), Vram::new(0x80060C1C)),
        UserSymbol::new("dm_make_magazine".into(), Vram::new(0x80060E10)),
        UserSymbol::new("func_80060F60".into(), Vram::new(0x80060F60)),
        UserSymbol::new("func_80060FA0".into(), Vram::new(0x80060FA0)),
        UserSymbol::new("dm_set_capsel".into(), Vram::new(0x80060FCC)),
        UserSymbol::new("dm_capsel_speed_up".into(), Vram::new(0x800610E0)),
        UserSymbol::new("dm_check_game_over".into(), Vram::new(0x80061184)),
        UserSymbol::new("update_flash_virus_count".into(), Vram::new(0x80061194)),
        UserSymbol::new("func_8006121C".into(), Vram::new(0x8006121C)),
        UserSymbol::new("dm_set_virus".into(), Vram::new(0x8006126C)),
        UserSymbol::new("dm_virus_anime".into(), Vram::new(0x800613D0)),
        UserSymbol::new("dm_capsel_erase_anime".into(), Vram::new(0x80061490)),
        UserSymbol::new("dm_make_erase_h_line".into(), Vram::new(0x8006151C)),
        UserSymbol::new("dm_h_erase_chack".into(), Vram::new(0x800615BC)),
        UserSymbol::new("dm_h_erase_chack_set".into(), Vram::new(0x80061678)),
        UserSymbol::new("dm_make_erase_w_line".into(), Vram::new(0x800618B0)),
        UserSymbol::new("dm_w_erase_chack".into(), Vram::new(0x80061954)),
        UserSymbol::new("dm_w_erase_chack_set".into(), Vram::new(0x80061A14)),
        UserSymbol::new("dm_h_ball_chack".into(), Vram::new(0x80061C50)),
        UserSymbol::new("dm_w_ball_chack".into(), Vram::new(0x80061D20)),
        UserSymbol::new("dm_black_up".into(), Vram::new(0x80061DB8)),
        UserSymbol::new("dm_broken_set".into(), Vram::new(0x80061E7C)),
        UserSymbol::new("dm_calc_erase_score_pos".into(), Vram::new(0x80061FE4)),
        UserSymbol::new("dm_calc_capsel_pos".into(), Vram::new(0x800620FC)),
        UserSymbol::new("dm_draw_capsel_by_gfx".into(), Vram::new(0x8006258C)),
        UserSymbol::new(
            "dm_draw_capsel_by_cpu_tentative".into(),
            Vram::new(0x800626A4),
        ),
        UserSymbol::new("dm_game_heap_top".into(), Vram::new(0x80062898)),
        UserSymbol::new("get_virus_anime_state".into(), Vram::new(0x800628A4)),
        UserSymbol::new("get_virus_smog_state".into(), Vram::new(0x800628BC)),
        UserSymbol::new("dm_get_mtx_buf".into(), Vram::new(0x800628E4)),
        UserSymbol::new("dm_get_vtx_buf".into(), Vram::new(0x80062900)),
        UserSymbol::new("func_80062920".into(), Vram::new(0x80062920)),
        UserSymbol::new("func_80062978".into(), Vram::new(0x80062978)),
        UserSymbol::new("func_80062990".into(), Vram::new(0x80062990)),
        UserSymbol::new("func_800629AC".into(), Vram::new(0x800629AC)),
        UserSymbol::new("func_800629C8".into(), Vram::new(0x800629C8)),
        UserSymbol::new("func_800629E4".into(), Vram::new(0x800629E4)),
        UserSymbol::new("func_80062A08".into(), Vram::new(0x80062A08)),
        UserSymbol::new("func_80062A24".into(), Vram::new(0x80062A24)),
        UserSymbol::new("func_80062A40".into(), Vram::new(0x80062A40)),
        UserSymbol::new("func_80062A5C".into(), Vram::new(0x80062A5C)),
        UserSymbol::new("func_80062A8C".into(), Vram::new(0x80062A8C)),
        UserSymbol::new("func_80062AA8".into(), Vram::new(0x80062AA8)),
        UserSymbol::new("func_80062AC4".into(), Vram::new(0x80062AC4)),
        UserSymbol::new("func_80062AE0".into(), Vram::new(0x80062AE0)),
        UserSymbol::new("func_80062AFC".into(), Vram::new(0x80062AFC)),
        UserSymbol::new("func_80062B18".into(), Vram::new(0x80062B18)),
        UserSymbol::new("func_80062B34".into(), Vram::new(0x80062B34)),
        UserSymbol::new("func_80062B50".into(), Vram::new(0x80062B50)),
        UserSymbol::new("func_80062B84".into(), Vram::new(0x80062B84)),
        UserSymbol::new("func_80062BC4".into(), Vram::new(0x80062BC4)),
        UserSymbol::new("timeAttackResult_update".into(), Vram::new(0x80062BD4)),
        UserSymbol::new("func_80062DA4".into(), Vram::new(0x80062DA4)),
        UserSymbol::new("func_80062DD8".into(), Vram::new(0x80062DD8)),
        UserSymbol::new("func_80062E84".into(), Vram::new(0x80062E84)),
        UserSymbol::new("func_80062EC0".into(), Vram::new(0x80062EC0)),
        UserSymbol::new("scoreNums_set".into(), Vram::new(0x80062FB8)),
        UserSymbol::new("backup_game_state".into(), Vram::new(0x800630F8)),
        UserSymbol::new("resume_game_state".into(), Vram::new(0x80063238)),
        UserSymbol::new("func_80063378".into(), Vram::new(0x80063378)),
        UserSymbol::new("func_800633C0".into(), Vram::new(0x800633C0)),
        UserSymbol::new("func_800633FC".into(), Vram::new(0x800633FC)),
        UserSymbol::new("dm_warning_h_line_se".into(), Vram::new(0x80063440)),
        UserSymbol::new("dm_play_count_down_se".into(), Vram::new(0x800634E8)),
        UserSymbol::new("dm_capsel_down".into(), Vram::new(0x800635BC)),
        UserSymbol::new("func_80063844".into(), Vram::new(0x80063844)),
        UserSymbol::new("dm_set_attack_2p".into(), Vram::new(0x800638AC)),
        UserSymbol::new("dm_set_attack_4p".into(), Vram::new(0x80063A80)),
        UserSymbol::new("func_80063FF4".into(), Vram::new(0x80063FF4)),
        UserSymbol::new("save_visible_fall_point_flag".into(), Vram::new(0x8006408C)),
        UserSymbol::new("retryMenu_init".into(), Vram::new(0x80064130)),
        UserSymbol::new("func_8006417C".into(), Vram::new(0x8006417C)),
        UserSymbol::new("retryMenu_input".into(), Vram::new(0x80064198)),
        UserSymbol::new("func_80064298".into(), Vram::new(0x80064298)),
        UserSymbol::new("func_8006431C".into(), Vram::new(0x8006431C)),
        UserSymbol::new("set_bottom_up_virus".into(), Vram::new(0x800643E0)),
        UserSymbol::new("bottom_up_bottle_items".into(), Vram::new(0x80064628)),
        UserSymbol::new("add_taiQ_bonus_wait".into(), Vram::new(0x80064748)),
        UserSymbol::new("func_80064848".into(), Vram::new(0x80064848)),
        UserSymbol::new("dm_game_eep_write_callback".into(), Vram::new(0x800648C0)),
        UserSymbol::new("func_80064940".into(), Vram::new(0x80064940)),
        UserSymbol::new("func_8006498C".into(), Vram::new(0x8006498C)),
        UserSymbol::new("dm_save_all".into(), Vram::new(0x800649CC)),
        UserSymbol::new("dm_query_pause_player".into(), Vram::new(0x80064E4C)),
        UserSymbol::new("dm_game_main_cnt_1P".into(), Vram::new(0x80064F18)),
        UserSymbol::new("dm_game_main_cnt".into(), Vram::new(0x80065BC4)),
        UserSymbol::new("dm_set_pause_on".into(), Vram::new(0x800666FC)),
        UserSymbol::new("func_80066808".into(), Vram::new(0x80066808)),
        UserSymbol::new("dm_set_pause_and_volume".into(), Vram::new(0x8006683C)),
        UserSymbol::new("func_800669A0".into(), Vram::new(0x800669A0)),
        UserSymbol::new("dm_calc_big_virus_pos".into(), Vram::new(0x800669C4)),
        UserSymbol::new("dm_game_main_1p".into(), Vram::new(0x80066F68)),
        UserSymbol::new("dm_add_win_2p".into(), Vram::new(0x8006729C)),
        UserSymbol::new("dm_set_win_2p".into(), Vram::new(0x800673FC)),
        UserSymbol::new("dm_set_lose_2p".into(), Vram::new(0x800674EC)),
        UserSymbol::new("func_800675C8".into(), Vram::new(0x800675C8)),
        UserSymbol::new("dm_set_time_attack_result_2p".into(), Vram::new(0x80067668)),
        UserSymbol::new("dm_game_main_2p".into(), Vram::new(0x80067834)),
        UserSymbol::new("dm_game_main_4p".into(), Vram::new(0x80067F58)),
        UserSymbol::new("dm_game_demo_1p".into(), Vram::new(0x80068950)),
        UserSymbol::new("dm_game_demo_2p".into(), Vram::new(0x80068AA0)),
        UserSymbol::new("dm_game_demo_4p".into(), Vram::new(0x80068C24)),
        UserSymbol::new("dm_game_get_capsel_tex".into(), Vram::new(0x80068DC0)),
        UserSymbol::new("dm_game_get_capsel_pal".into(), Vram::new(0x80068DE8)),
        UserSymbol::new("scoreNums_draw".into(), Vram::new(0x80068E24)),
        UserSymbol::new("func_80069160".into(), Vram::new(0x80069160)),
        UserSymbol::new("func_80069188".into(), Vram::new(0x80069188)),
        UserSymbol::new("starForce_draw".into(), Vram::new(0x800691F8)),
        UserSymbol::new("func_800695A8".into(), Vram::new(0x800695A8)),
        UserSymbol::new("draw_4p_attack_guide_panel".into(), Vram::new(0x8006967C)),
        UserSymbol::new("dm_map_draw".into(), Vram::new(0x80069990)),
        UserSymbol::new("func_80069ACC".into(), Vram::new(0x80069ACC)),
        UserSymbol::new("draw_virus_number".into(), Vram::new(0x80069BCC)),
        UserSymbol::new("draw_count_number".into(), Vram::new(0x80069E20)),
        UserSymbol::new("func_8006A098".into(), Vram::new(0x8006A098)),
        UserSymbol::new("draw_time2".into(), Vram::new(0x8006A198)),
        UserSymbol::new("push_any_key_draw".into(), Vram::new(0x8006A2BC)),
        UserSymbol::new("draw_demo_logo".into(), Vram::new(0x8006A480)),
        UserSymbol::new("draw_replay_logo".into(), Vram::new(0x8006A6E8)),
        UserSymbol::new("func_8006A938".into(), Vram::new(0x8006A938)),
        UserSymbol::new("_disp_coin_logo".into(), Vram::new(0x8006A978)),
        UserSymbol::new("draw_flash_virus_light".into(), Vram::new(0x8006AC60)),
        UserSymbol::new("draw_flash_virus_lights".into(), Vram::new(0x8006AE04)),
        UserSymbol::new("func_8006AEFC".into(), Vram::new(0x8006AEFC)),
        UserSymbol::new("draw_story_board".into(), Vram::new(0x8006B0A4)),
        UserSymbol::new("draw_vsmode_board".into(), Vram::new(0x8006B270)),
        UserSymbol::new("_draw_bottle_10".into(), Vram::new(0x8006B8E0)),
        UserSymbol::new("dm_calc_bottle_2p".into(), Vram::new(0x8006BBEC)),
        UserSymbol::new("dm_draw_bottle_2p".into(), Vram::new(0x8006BC5C)),
        UserSymbol::new("dm_draw_big_virus".into(), Vram::new(0x8006BD00)),
        UserSymbol::new("dm_draw_KaSaMaRu".into(), Vram::new(0x8006BDFC)),
        UserSymbol::new("dm_game_graphic_common".into(), Vram::new(0x8006C1D0)),
        UserSymbol::new("dm_game_graphic_p".into(), Vram::new(0x8006C580)),
        UserSymbol::new("dm_game_graphic_1p".into(), Vram::new(0x8006C6FC)),
        UserSymbol::new("dm_game_graphic_effect".into(), Vram::new(0x8006C88C)),
        UserSymbol::new("func_8006D0E8".into(), Vram::new(0x8006D0E8)),
        UserSymbol::new("dm_make_key".into(), Vram::new(0x8006D150)),
        UserSymbol::new("key_control_main".into(), Vram::new(0x8006D254)),
        UserSymbol::new("make_ai_main".into(), Vram::new(0x8006D620)),
        UserSymbol::new("dm_effect_make".into(), Vram::new(0x8006D7B0)),
        UserSymbol::new("dm_game_init_heap".into(), Vram::new(0x8006D870)),
        UserSymbol::new("dm_game_init".into(), Vram::new(0x8006D91C)),
        UserSymbol::new("dm_game_init_static".into(), Vram::new(0x8006E0EC)),
        UserSymbol::new("dm_game_init_snap_bg".into(), Vram::new(0x8006E80C)),
        UserSymbol::new("dm_game_draw_snap_bg".into(), Vram::new(0x8006E884)),
        UserSymbol::new("func_8006F628".into(), Vram::new(0x8006F628)),
        UserSymbol::new("dm_game_main".into(), Vram::new(0x8006F684)),
        UserSymbol::new("dm_game_main2".into(), Vram::new(0x8006F950)),
        UserSymbol::new("dm_game_main3".into(), Vram::new(0x8006FFD0)),
        UserSymbol::new("dm_game_graphic".into(), Vram::new(0x8007023C)),
        UserSymbol::new("dm_game_graphic2".into(), Vram::new(0x80070360)),
        UserSymbol::new("dm_game_graphic_onDoneSawp".into(), Vram::new(0x80071624)),
        UserSymbol::new("func_80071A44".into(), Vram::new(0x80071A44)),
        UserSymbol::new("main_techmes".into(), Vram::new(0x80071A4C)),
        UserSymbol::new("graphic_techmes".into(), Vram::new(0x80071CE0)),
        UserSymbol::new("func_80071EF0".into(), Vram::new(0x80071EF0)),
        UserSymbol::new("func_80071F0C".into(), Vram::new(0x80071F0C)),
        UserSymbol::new("func_80071F14".into(), Vram::new(0x80071F14)),
        UserSymbol::new("func_80071FA0".into(), Vram::new(0x80071FA0)),
        UserSymbol::new("tutolWnd_draw".into(), Vram::new(0x80072028)),
        UserSymbol::new("func_800721A0".into(), Vram::new(0x800721A0)),
        UserSymbol::new("func_800721BC".into(), Vram::new(0x800721BC)),
        UserSymbol::new("func_800721D8".into(), Vram::new(0x800721D8)),
        UserSymbol::new("func_80072204".into(), Vram::new(0x80072204)),
        UserSymbol::new("func_80072230".into(), Vram::new(0x80072230)),
        UserSymbol::new("func_8007224C".into(), Vram::new(0x8007224C)),
        UserSymbol::new("func_80072268".into(), Vram::new(0x80072268)),
        UserSymbol::new("dm_manual_attack_capsel_down".into(), Vram::new(0x80072280)),
        UserSymbol::new("func_800723EC".into(), Vram::new(0x800723EC)),
        UserSymbol::new("dm_manual_update_virus_anime".into(), Vram::new(0x80072428)),
        UserSymbol::new("dm_manual_main_cnt".into(), Vram::new(0x8007248C)),
        UserSymbol::new("dm_manual_make_key".into(), Vram::new(0x80072968)),
        UserSymbol::new("dm_manual_1_main".into(), Vram::new(0x80072AB4)),
        UserSymbol::new("dm_manual_2_main".into(), Vram::new(0x800733AC)),
        UserSymbol::new("dm_manual_3_main".into(), Vram::new(0x800739D8)),
        UserSymbol::new("dm_manual_4_main".into(), Vram::new(0x80074330)),
        UserSymbol::new("draw_AB_guide".into(), Vram::new(0x80074954)),
        UserSymbol::new("func_80074B08".into(), Vram::new(0x80074B08)),
        UserSymbol::new("func_80074EF0".into(), Vram::new(0x80074EF0)),
        UserSymbol::new("disp_cont".into(), Vram::new(0x80074FE0)),
        UserSymbol::new("dm_manual_draw_fg".into(), Vram::new(0x800752B8)),
        UserSymbol::new("dm_manual_all_init".into(), Vram::new(0x800755F0)),
        UserSymbol::new("dm_manual_main".into(), Vram::new(0x80075A2C)),
        UserSymbol::new("dm_manual_graphic".into(), Vram::new(0x80075CF8)),
        UserSymbol::new("func_80075F30".into(), Vram::new(0x80075F30)),
        UserSymbol::new("dm_title_main".into(), Vram::new(0x80075F98)),
        UserSymbol::new("dm_title_graphic".into(), Vram::new(0x8007636C)),
        UserSymbol::new("main_boot_error".into(), Vram::new(0x8007657C)),
        UserSymbol::new("graphic_boot_error".into(), Vram::new(0x800767DC)),
        UserSymbol::new("story_zoomfade".into(), Vram::new(0x800768E0)),
        UserSymbol::new("get_gbi_stat".into(), Vram::new(0x80076CA0)),
        UserSymbol::new("curtain_proc".into(), Vram::new(0x80076CCC)),
        UserSymbol::new("curtain_proc_org".into(), Vram::new(0x80076DB4)),
        UserSymbol::new("func_800770E8".into(), Vram::new(0x800770E8)),
        UserSymbol::new("story_bg_init".into(), Vram::new(0x80077170)),
        UserSymbol::new("story_bg_proc".into(), Vram::new(0x800771EC)),
        UserSymbol::new("func_800773F0".into(), Vram::new(0x800773F0)),
        UserSymbol::new("func_8007744C".into(), Vram::new(0x8007744C)),
        UserSymbol::new("func_8007746C".into(), Vram::new(0x8007746C)),
        UserSymbol::new("func_800774C4".into(), Vram::new(0x800774C4)),
        UserSymbol::new("story_spot".into(), Vram::new(0x80077504)),
        UserSymbol::new("func_800777E8".into(), Vram::new(0x800777E8)),
        UserSymbol::new("init_coffee_break".into(), Vram::new(0x8007780C)),
        UserSymbol::new("init_coffee_break_cnt".into(), Vram::new(0x8007786C)),
        UserSymbol::new("draw_coffee_break".into(), Vram::new(0x800778E8)),
        UserSymbol::new("init_menu_bg".into(), Vram::new(0x80077D68)),
        UserSymbol::new("draw_menu_bg".into(), Vram::new(0x80077E2C)),
        UserSymbol::new("init_title".into(), Vram::new(0x80077FA4)),
        UserSymbol::new("demo_title".into(), Vram::new(0x80078094)),
        UserSymbol::new("func_80078648".into(), Vram::new(0x80078648)),
        UserSymbol::new("func_8007865C".into(), Vram::new(0x8007865C)),
        UserSymbol::new("story_st_new_op".into(), Vram::new(0x8007873C)),
        UserSymbol::new("story_st_new".into(), Vram::new(0x80078F78)),
        UserSymbol::new("story_st_new2_f".into(), Vram::new(0x800791D0)),
        UserSymbol::new("story_st_new2".into(), Vram::new(0x800796F4)),
        UserSymbol::new("story_m_end".into(), Vram::new(0x80079B24)),
        UserSymbol::new("story_st_new_w9".into(), Vram::new(0x8007A154)),
        UserSymbol::new("story_w_end".into(), Vram::new(0x8007A440)),
        UserSymbol::new("func_8007A9DC".into(), Vram::new(0x8007A9DC)),
        UserSymbol::new("main_story".into(), Vram::new(0x8007AA84)),
        UserSymbol::new("init_objMtx".into(), Vram::new(0x8007AEBC)),
        UserSymbol::new("graphic_story".into(), Vram::new(0x8007AEF4)),
        UserSymbol::new("func_8007B62C".into(), Vram::new(0x8007B62C)),
        UserSymbol::new("lws_anim".into(), Vram::new(0x8007B650)),
        UserSymbol::new("func_8007BC20".into(), Vram::new(0x8007BC20)),
        UserSymbol::new("func_8007BC54".into(), Vram::new(0x8007BC54)),
        UserSymbol::new("func_8007BD30".into(), Vram::new(0x8007BD30)),
        UserSymbol::new("func_8007BEEC".into(), Vram::new(0x8007BEEC)),
        UserSymbol::new("func_8007BFE0".into(), Vram::new(0x8007BFE0)),
        UserSymbol::new("func_8007C0C4".into(), Vram::new(0x8007C0C4)),
        UserSymbol::new("func_8007C244".into(), Vram::new(0x8007C244)),
        UserSymbol::new("func_8007C480".into(), Vram::new(0x8007C480)),
        UserSymbol::new("angleF2S".into(), Vram::new(0x8007C4AC)),
        UserSymbol::new("func_8007C4D8".into(), Vram::new(0x8007C4D8)),
        UserSymbol::new("func_8007C540".into(), Vram::new(0x8007C540)),
        UserSymbol::new("func_8007C5A8".into(), Vram::new(0x8007C5A8)),
        UserSymbol::new("func_8007C624".into(), Vram::new(0x8007C624)),
        UserSymbol::new("func_8007C6D8".into(), Vram::new(0x8007C6D8)),
        UserSymbol::new("func_8007C780".into(), Vram::new(0x8007C780)),
        UserSymbol::new("func_8007C7E8".into(), Vram::new(0x8007C7E8)),
        UserSymbol::new("func_8007C894".into(), Vram::new(0x8007C894)),
        UserSymbol::new("func_8007C9C8".into(), Vram::new(0x8007C9C8)),
        UserSymbol::new("func_8007CAFC".into(), Vram::new(0x8007CAFC)),
        UserSymbol::new("func_8007CBE4".into(), Vram::new(0x8007CBE4)),
        UserSymbol::new("func_8007CC68".into(), Vram::new(0x8007CC68)),
        UserSymbol::new("func_8007CCFC".into(), Vram::new(0x8007CCFC)),
        UserSymbol::new("func_8007CD78".into(), Vram::new(0x8007CD78)),
        UserSymbol::new("func_8007CFB4".into(), Vram::new(0x8007CFB4)),
        UserSymbol::new("randomize00".into(), Vram::new(0x8007D078)),
        UserSymbol::new("randomseed".into(), Vram::new(0x8007D108)),
        UserSymbol::new("irandom".into(), Vram::new(0x8007D1C8)),
        UserSymbol::new("random".into(), Vram::new(0x8007D224)),
        UserSymbol::new("func_8007D260".into(), Vram::new(0x8007D260)),
        UserSymbol::new("mult_func_8007D348".into(), Vram::new(0x8007D348)),
        UserSymbol::new("mult_func_8007D410".into(), Vram::new(0x8007D410)),
        UserSymbol::new("mult_func_8007D4CC".into(), Vram::new(0x8007D4CC)),
        UserSymbol::new("mult_func_8007D568".into(), Vram::new(0x8007D568)),
        UserSymbol::new("mult_func_8007D644".into(), Vram::new(0x8007D644)),
        UserSymbol::new("func_8007D74C".into(), Vram::new(0x8007D74C)),
        UserSymbol::new("func_8007D83C".into(), Vram::new(0x8007D83C)),
        UserSymbol::new("func_8007DB78".into(), Vram::new(0x8007DB78)),
        UserSymbol::new("func_8007DD70".into(), Vram::new(0x8007DD70)),
        UserSymbol::new("func_8007DF64".into(), Vram::new(0x8007DF64)),
        UserSymbol::new("func_8007E260".into(), Vram::new(0x8007E260)),
        UserSymbol::new("replay_record_init_buffer".into(), Vram::new(0x8007E2E0)),
        UserSymbol::new("replay_record_init".into(), Vram::new(0x8007E2FC)),
        UserSymbol::new("replay_record".into(), Vram::new(0x8007E418)),
        UserSymbol::new("replay_play_init".into(), Vram::new(0x8007E5C4)),
        UserSymbol::new("replay_play".into(), Vram::new(0x8007E638)),
        UserSymbol::new("func_8007E760".into(), Vram::new(0x8007E760)),
        UserSymbol::new("func_8007EA20".into(), Vram::new(0x8007EA20)),
        UserSymbol::new("func_8007EA38".into(), Vram::new(0x8007EA38)),
        UserSymbol::new("func_8007EA58".into(), Vram::new(0x8007EA58)),
        UserSymbol::new("func_8007EAEC".into(), Vram::new(0x8007EAEC)),
        UserSymbol::new("func_8007EC54".into(), Vram::new(0x8007EC54)),
        UserSymbol::new("func_8007EC98".into(), Vram::new(0x8007EC98)),
        UserSymbol::new("func_8007ECE0".into(), Vram::new(0x8007ECE0)),
        UserSymbol::new("func_8007ED74".into(), Vram::new(0x8007ED74)),
        UserSymbol::new("matrixMulL".into(), Vram::new(0x8007EE18)),
        UserSymbol::new("makeTransrateMatrix".into(), Vram::new(0x8007F004)),
        UserSymbol::new("makeScaleMatrix".into(), Vram::new(0x8007F04C)),
        UserSymbol::new("makeMatrix".into(), Vram::new(0x8007F094)),
        UserSymbol::new("func_8007F214".into(), Vram::new(0x8007F214)),
        UserSymbol::new("func_8007F284".into(), Vram::new(0x8007F284)),
        UserSymbol::new("func_8007F2F4".into(), Vram::new(0x8007F2F4)),
        UserSymbol::new("func_8007F364".into(), Vram::new(0x8007F364)),
        UserSymbol::new("matrixConv".into(), Vram::new(0x8007F420)),
        UserSymbol::new("func_8007F514".into(), Vram::new(0x8007F514)),
        UserSymbol::new("func_8007F550".into(), Vram::new(0x8007F550)),
        UserSymbol::new("func_8007F60C".into(), Vram::new(0x8007F60C)),
        UserSymbol::new("func_8007F670".into(), Vram::new(0x8007F670)),
        UserSymbol::new("func_8007F6C4".into(), Vram::new(0x8007F6C4)),
        UserSymbol::new("func_8007F720".into(), Vram::new(0x8007F720)),
        UserSymbol::new("func_8007F754".into(), Vram::new(0x8007F754)),
        UserSymbol::new("func_8007F798".into(), Vram::new(0x8007F798)),
        UserSymbol::new("func_8007F824".into(), Vram::new(0x8007F824)),
        UserSymbol::new("func_8007F890".into(), Vram::new(0x8007F890)),
        UserSymbol::new("func_8007F990".into(), Vram::new(0x8007F990)),
        UserSymbol::new("func_8007F9D0".into(), Vram::new(0x8007F9D0)),
        UserSymbol::new("Fenvelope".into(), Vram::new(0x8007FA00)),
        UserSymbol::new("MusInitialize".into(), Vram::new(0x8007FB4C)),
        UserSymbol::new("MusSetMasterVolume".into(), Vram::new(0x8007FDE4)),
        UserSymbol::new("MusStartSong".into(), Vram::new(0x8007FE10)),
        UserSymbol::new("MusStartSongFromMarker".into(), Vram::new(0x8007FE40)),
        UserSymbol::new("MusStartEffect".into(), Vram::new(0x80080090)),
        UserSymbol::new("MusStartEffect2".into(), Vram::new(0x80080110)),
        UserSymbol::new("MusStop".into(), Vram::new(0x80080228)),
        UserSymbol::new("MusAsk".into(), Vram::new(0x800802F4)),
        UserSymbol::new("MusHandleStop".into(), Vram::new(0x80080374)),
        UserSymbol::new("MusHandleAsk".into(), Vram::new(0x80080428)),
        UserSymbol::new("MusHandleSetVolume".into(), Vram::new(0x80080480)),
        UserSymbol::new("MusHandleSetPan".into(), Vram::new(0x800804E0)),
        UserSymbol::new("MusHandleSetFreqOffset".into(), Vram::new(0x80080550)),
        UserSymbol::new("MusHandleSetTempo".into(), Vram::new(0x800805C0)),
        UserSymbol::new("MusHandleSetReverb".into(), Vram::new(0x8008064C)),
        UserSymbol::new("MusPtrBankInitialize".into(), Vram::new(0x800806D4)),
        UserSymbol::new("MusPtrBankSetSingle".into(), Vram::new(0x80080710)),
        UserSymbol::new("MusPtrBankSetCurrent".into(), Vram::new(0x80080738)),
        UserSymbol::new("MusPtrBankGetCurrent".into(), Vram::new(0x8008075C)),
        UserSymbol::new("MusHandleGetPtrBank".into(), Vram::new(0x80080768)),
        UserSymbol::new("MusHandlePause".into(), Vram::new(0x800807C4)),
        UserSymbol::new("MusHandleUnPause".into(), Vram::new(0x800807E8)),
        UserSymbol::new("MusSetFxType".into(), Vram::new(0x80080810)),
        UserSymbol::new("MusSetSongFxChange".into(), Vram::new(0x80080840)),
        UserSymbol::new("MusFxBankInitialize".into(), Vram::new(0x8008089C)),
        UserSymbol::new("MusFxBankNumberOfEffects".into(), Vram::new(0x80080920)),
        UserSymbol::new("MusFxBankSetCurrent".into(), Vram::new(0x80080928)),
        UserSymbol::new("MusFxBankSetSingle".into(), Vram::new(0x80080934)),
        UserSymbol::new("MusFxBankGetCurrent".into(), Vram::new(0x80080940)),
        UserSymbol::new("MusFxBankSetPtrBank".into(), Vram::new(0x8008094C)),
        UserSymbol::new("MusFxBankGetPtrBank".into(), Vram::new(0x80080954)),
        UserSymbol::new("MusSetScheduler".into(), Vram::new(0x8008095C)),
        UserSymbol::new("MusHandleWaveCount".into(), Vram::new(0x80080968)),
        UserSymbol::new("MusHandleWaveAddress".into(), Vram::new(0x800809CC)),
        UserSymbol::new("MusSetMarkerCallback".into(), Vram::new(0x80080A44)),
        UserSymbol::new("__MusIntFifoOpen".into(), Vram::new(0x80080A50)),
        UserSymbol::new("__MusIntFifoProcessCommand".into(), Vram::new(0x80080AB4)),
        UserSymbol::new("__MusIntFifoAddCommand".into(), Vram::new(0x80080B34)),
        UserSymbol::new("__MusIntMain".into(), Vram::new(0x80080BCC)),
        UserSymbol::new("__MusIntGetNewNote".into(), Vram::new(0x80080EC4)),
        UserSymbol::new("__MusIntFlushPending".into(), Vram::new(0x80081430)),
        UserSymbol::new("__MusIntSetVolumeAndPan".into(), Vram::new(0x800814BC)),
        UserSymbol::new("__MusIntSetPitch".into(), Vram::new(0x80081620)),
        UserSymbol::new("__MusIntInitEnvelope".into(), Vram::new(0x80081734)),
        UserSymbol::new("__MusIntProcessEnvelope".into(), Vram::new(0x800817A4)),
        UserSymbol::new("__MusIntInitSweep".into(), Vram::new(0x80081980)),
        UserSymbol::new("__MusIntProcessSweep".into(), Vram::new(0x8008199C)),
        UserSymbol::new("__MusIntProcessWobble".into(), Vram::new(0x80081A54)),
        UserSymbol::new("__MusIntProcessVibrato".into(), Vram::new(0x80081AAC)),
        UserSymbol::new(
            "__MusIntProcessContinuousVolume".into(),
            Vram::new(0x80081B10),
        ),
        UserSymbol::new(
            "__MusIntProcessContinuousPitchBend".into(),
            Vram::new(0x80081BD4),
        ),
        UserSymbol::new("__MusIntPowerOf2".into(), Vram::new(0x80081CE8)),
        UserSymbol::new("__MusIntRemapPtrBank".into(), Vram::new(0x80081E7C)),
        UserSymbol::new("__MusIntRandom".into(), Vram::new(0x80082014)),
        UserSymbol::new("__MusIntInitialiseChannel".into(), Vram::new(0x800820A8)),
        UserSymbol::new("__MusIntFindChannel".into(), Vram::new(0x800821E4)),
        UserSymbol::new("__MusIntRemapPtrs".into(), Vram::new(0x800823AC)),
        UserSymbol::new("__MusIntStartEffect".into(), Vram::new(0x800823E4)),
        UserSymbol::new("__MusIntFindChannelAndStart".into(), Vram::new(0x80082494)),
        UserSymbol::new("__MusIntStartSong".into(), Vram::new(0x8008263C)),
        UserSymbol::new("__MusIntHandleSetFlag".into(), Vram::new(0x800828A8)),
        UserSymbol::new("Fstop".into(), Vram::new(0x800828FC)),
        UserSymbol::new("Fwave".into(), Vram::new(0x8008291C)),
        UserSymbol::new("Fport".into(), Vram::new(0x8008294C)),
        UserSymbol::new("Fportoff".into(), Vram::new(0x8008296C)),
        UserSymbol::new("Fdefa".into(), Vram::new(0x80082978)),
        UserSymbol::new("Ftempo".into(), Vram::new(0x80082AA0)),
        UserSymbol::new("Fendit".into(), Vram::new(0x80082B84)),
        UserSymbol::new("Fcutoff".into(), Vram::new(0x80082B98)),
        UserSymbol::new("Fvibup".into(), Vram::new(0x80082BBC)),
        UserSymbol::new("Fvibdown".into(), Vram::new(0x80082C2C)),
        UserSymbol::new("Fviboff".into(), Vram::new(0x80082CA0)),
        UserSymbol::new("Flength".into(), Vram::new(0x80082CB0)),
        UserSymbol::new("Fignore".into(), Vram::new(0x80082CE0)),
        UserSymbol::new("Ftrans".into(), Vram::new(0x80082CF0)),
        UserSymbol::new("Fignore_trans".into(), Vram::new(0x80082D00)),
        UserSymbol::new("Fdistort".into(), Vram::new(0x80082D10)),
        UserSymbol::new("Fenvoff".into(), Vram::new(0x80082D68)),
        UserSymbol::new("Fenvon".into(), Vram::new(0x80082D78)),
        UserSymbol::new("Ftroff".into(), Vram::new(0x80082D84)),
        UserSymbol::new("Ftron".into(), Vram::new(0x80082D94)),
        UserSymbol::new("Ffor".into(), Vram::new(0x80082DA0)),
        UserSymbol::new("Fnext".into(), Vram::new(0x80082E0C)),
        UserSymbol::new("Fwobble".into(), Vram::new(0x80082EA8)),
        UserSymbol::new("Fwobbleoff".into(), Vram::new(0x80082ED0)),
        UserSymbol::new("Fvelon".into(), Vram::new(0x80082EDC)),
        UserSymbol::new("Fveloff".into(), Vram::new(0x80082EEC)),
        UserSymbol::new("Fvelocity".into(), Vram::new(0x80082EF8)),
        UserSymbol::new("Fpan".into(), Vram::new(0x80082F0C)),
        UserSymbol::new("Fstereo".into(), Vram::new(0x80082F20)),
        UserSymbol::new("Fdrums".into(), Vram::new(0x80082F28)),
        UserSymbol::new("Fdrumsoff".into(), Vram::new(0x80082F70)),
        UserSymbol::new("Fprint".into(), Vram::new(0x80082F7C)),
        UserSymbol::new("Fgoto".into(), Vram::new(0x80082F84)),
        UserSymbol::new("Freverb".into(), Vram::new(0x80082FF4)),
        UserSymbol::new("FrandNote".into(), Vram::new(0x80083004)),
        UserSymbol::new("FrandVolume".into(), Vram::new(0x80083054)),
        UserSymbol::new("FrandPan".into(), Vram::new(0x800830A4)),
        UserSymbol::new("Fvolume".into(), Vram::new(0x800830F4)),
        UserSymbol::new("Fstartfx".into(), Vram::new(0x80083104)),
        UserSymbol::new("Fbendrange".into(), Vram::new(0x800831D4)),
        UserSymbol::new("Fsweep".into(), Vram::new(0x80083210)),
        UserSymbol::new("Fchangefx".into(), Vram::new(0x80083220)),
        UserSymbol::new("Fmarker".into(), Vram::new(0x80083264)),
        UserSymbol::new("Flength0".into(), Vram::new(0x800832D4)),
        UserSymbol::new("CustomInit".into(), Vram::new(0x800832E0)),
        UserSymbol::new("CustomSynNew".into(), Vram::new(0x80083338)),
        UserSymbol::new("CustomAllocFX".into(), Vram::new(0x80083604)),
        UserSymbol::new("CustomFxNew".into(), Vram::new(0x80083698)),
        UserSymbol::new("CustomFxSet".into(), Vram::new(0x800837E4)),
        UserSymbol::new("SetForCustom".into(), Vram::new(0x80083AC4)),
        UserSymbol::new("GetFxMostSections".into(), Vram::new(0x80083ADC)),
        UserSymbol::new("GetFxLongest".into(), Vram::new(0x80083B20)),
        UserSymbol::new("ChangeCustomEffect".into(), Vram::new(0x80083B64)),
        UserSymbol::new("__MusIntDmaInit".into(), Vram::new(0x80083BC0)),
        UserSymbol::new("__MusIntDmaProcess".into(), Vram::new(0x80083D34)),
        UserSymbol::new("__CallBackDmaNew".into(), Vram::new(0x80083E04)),
        UserSymbol::new("__CallBackDmaProcess".into(), Vram::new(0x80083E10)),
        UserSymbol::new("__MusIntDmaSample".into(), Vram::new(0x80083E7C)),
        UserSymbol::new("__MusIntSchedInit".into(), Vram::new(0x80084020)),
        UserSymbol::new("__OsSchedInstall".into(), Vram::new(0x8008402C)),
        UserSymbol::new("__OsSchedWaitFrame".into(), Vram::new(0x80084090)),
        UserSymbol::new("__OsSchedDoTask".into(), Vram::new(0x800840F0)),
        UserSymbol::new("__MusIntAudManInit".into(), Vram::new(0x800841D0)),
        UserSymbol::new("__MusIntThreadProcess".into(), Vram::new(0x80084308)),
        UserSymbol::new("__MusIntMemInit".into(), Vram::new(0x80084490)),
        UserSymbol::new("__MusIntMemMalloc".into(), Vram::new(0x800844DC)),
        UserSymbol::new("__MusIntMemRemaining".into(), Vram::new(0x8008450C)),
        UserSymbol::new("__MusIntMemGetHeapAddr".into(), Vram::new(0x80084524)),
        UserSymbol::new("__MusIntMemSet".into(), Vram::new(0x80084530)),
        UserSymbol::new("__MusIntMemMove".into(), Vram::new(0x8008455C)),
        UserSymbol::new("__MusIntSamplesInit".into(), Vram::new(0x800845D0)),
        UserSymbol::new("__MusIntSamplesCurrent".into(), Vram::new(0x80084648)),
        UserSymbol::new("rspbootTextStart".into(), Vram::new(0x80084690))
            .with_name_end("rspbootTextEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("aspMainTextStart".into(), Vram::new(0x80084760))
            .with_name_end("aspMainTextEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("gspF3DEX2_fifoTextStart".into(), Vram::new(0x80085580))
            .with_name_end("gspF3DEX2_fifoTextEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("gspS2DEX_fifoTextStart".into(), Vram::new(0x80086910))
            .with_name_end("gspS2DEX_fifoTextEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("framecont".into(), Vram::new(0x80088100)),
        UserSymbol::new("D_80088104".into(), Vram::new(0x80088104)).with_size(Size::new(0x1)),
        UserSymbol::new("D_80088105".into(), Vram::new(0x80088105)).with_size(Size::new(0x1)),
        UserSymbol::new("gfx_ucode".into(), Vram::new(0x80088110)).with_size(Size::new(0x10)),
        UserSymbol::new("gCurrentFramebufferIndex".into(), Vram::new(0x80088120)),
        UserSymbol::new("graphic_no".into(), Vram::new(0x80088124)),
        UserSymbol::new("pendingGFX".into(), Vram::new(0x80088128)),
        UserSymbol::new("vp".into(), Vram::new(0x80088130)),
        UserSymbol::new("rdpinit_flag_161".into(), Vram::new(0x80088140)),
        UserSymbol::new("S2RDPinit_dl".into(), Vram::new(0x80088150))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("S2ClearCFB_dl".into(), Vram::new(0x800881B8))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("S2Spriteinit_dl".into(), Vram::new(0x800881E0))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("F3RDPinit_dl".into(), Vram::new(0x80088228))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("F3SetupRDP_dl".into(), Vram::new(0x80088308))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("F3SetupRSP_dl".into(), Vram::new(0x80088328))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("D_80088360".into(), Vram::new(0x80088360))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("D_800883A8".into(), Vram::new(0x800883A8))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("nn_mus_sched".into(), Vram::new(0x800883F0)).with_size(Size::new(0xC)),
        UserSymbol::new("evs_stereo".into(), Vram::new(0x80088400)),
        UserSymbol::new("evs_seqence".into(), Vram::new(0x80088401)),
        UserSymbol::new("evs_seqnumb".into(), Vram::new(0x80088402)),
        UserSymbol::new("evs_playcnt".into(), Vram::new(0x80088403)),
        UserSymbol::new("evs_keyrept".into(), Vram::new(0x80088404)).with_size(Size::new(0x2)),
        UserSymbol::new("evs_gamespeed".into(), Vram::new(0x80088406)),
        UserSymbol::new("evs_score_flag".into(), Vram::new(0x80088407)),
        UserSymbol::new("evs_story_flg".into(), Vram::new(0x80088408)),
        UserSymbol::new("evs_story_no".into(), Vram::new(0x80088409)).with_size(Size::new(0x1)),
        UserSymbol::new("evs_story_level".into(), Vram::new(0x8008840A)),
        UserSymbol::new("evs_secret_flg".into(), Vram::new(0x8008840C)).with_size(Size::new(0x2)),
        UserSymbol::new("evs_one_game_flg".into(), Vram::new(0x8008840E)),
        UserSymbol::new("evs_level_21".into(), Vram::new(0x8008840F)),
        UserSymbol::new("evs_manual_no".into(), Vram::new(0x80088410)),
        UserSymbol::new("evs_high_score".into(), Vram::new(0x80088414)),
        UserSymbol::new("evs_vs_count".into(), Vram::new(0x80088418)),
        UserSymbol::new("FlyingCnt".into(), Vram::new(0x8008841C)).with_size(Size::new(0x3)),
        UserSymbol::new("BonusWait".into(), Vram::new(0x80088420)).with_size(Size::new(0x9)),
        UserSymbol::new("D_8008842C".into(), Vram::new(0x8008842C)),
        UserSymbol::new("GameSpeed".into(), Vram::new(0x80088430)).with_size(Size::new(0x4)),
        UserSymbol::new("FallSpeed".into(), Vram::new(0x80088434)).with_size(Size::new(0x38)),
        UserSymbol::new("Score1p".into(), Vram::new(0x8008846C)).with_size(Size::new(0x12)),
        UserSymbol::new("mti".into(), Vram::new(0x80088480)),
        UserSymbol::new("mag01_108".into(), Vram::new(0x80088484)).with_size(Size::new(0x8)),
        UserSymbol::new("D_80088490".into(), Vram::new(0x80088490)),
        UserSymbol::new("aiVirusLevel".into(), Vram::new(0x800884A8)),
        UserSymbol::new("aiDownSpeed".into(), Vram::new(0x800884C0)),
        UserSymbol::new("aiSlideFSpeed".into(), Vram::new(0x800884D8)).with_size(Size::new(0x18)),
        UserSymbol::new("aiSlideSpeed".into(), Vram::new(0x800884F0)),
        UserSymbol::new("aiDebugP1".into(), Vram::new(0x80088508)),
        UserSymbol::new("capsGCnv_122".into(), Vram::new(0x8008850C)).with_size(Size::new(0x16)),
        UserSymbol::new("capsCCnv_123".into(), Vram::new(0x80088524)).with_size(Size::new(0x6)),
        UserSymbol::new("aiLinePri".into(), Vram::new(0x8008852C)),
        UserSymbol::new("srh_466".into(), Vram::new(0x80088534)).with_size(Size::new(0x8)),
        UserSymbol::new("bad_point".into(), Vram::new(0x8008853C)),
        UserSymbol::new("bad_point2".into(), Vram::new(0x8008854C)),
        UserSymbol::new("pri_point".into(), Vram::new(0x8008855C)).with_size(Size::new(0x12)),
        UserSymbol::new("EraseLinP".into(), Vram::new(0x80088570)).with_size(Size::new(0x12)),
        UserSymbol::new("HeiEraseLinRate".into(), Vram::new(0x80088584)),
        UserSymbol::new("WidEraseLinRate".into(), Vram::new(0x80088588)),
        UserSymbol::new("HeiLinesAllp".into(), Vram::new(0x8008858C)).with_size(Size::new(0x12)),
        UserSymbol::new("WidLinesAllp".into(), Vram::new(0x800885A0)).with_size(Size::new(0x12)),
        UserSymbol::new("AloneCapP".into(), Vram::new(0x800885B4)).with_size(Size::new(0xC)),
        UserSymbol::new("AloneCapWP".into(), Vram::new(0x800885C0)).with_size(Size::new(0xC)),
        UserSymbol::new("OnVirusP".into(), Vram::new(0x800885CC)),
        UserSymbol::new("D_800885D0".into(), Vram::new(0x800885D0)),
        UserSymbol::new("RensaP".into(), Vram::new(0x800885D2)),
        UserSymbol::new("RensaMP".into(), Vram::new(0x800885D4)),
        UserSymbol::new("LPriP".into(), Vram::new(0x800885D6)),
        UserSymbol::new("BadLineRate".into(), Vram::new(0x800885D8)),
        UserSymbol::new("WallRate".into(), Vram::new(0x800885F8)),
        UserSymbol::new("tbl_2973".into(), Vram::new(0x80088638)),
        UserSymbol::new("ai_param_org".into(), Vram::new(0x80088660)),
        UserSymbol::new("ai_char_data_org".into(), Vram::new(0x800890E0)),
        UserSymbol::new("mess_panel_tex_size".into(), Vram::new(0x800897A0))
            .with_size(Size::new(0x8)),
        UserSymbol::new("mess_panel_lut".into(), Vram::new(0x800897A8)).with_size(Size::new(0x200)),
        UserSymbol::new("mess_panel_tex".into(), Vram::new(0x800899A8)),
        UserSymbol::new("_cached_1332".into(), Vram::new(0x8008CF90)),
        UserSymbol::new("D_8008CFA0".into(), Vram::new(0x8008CFA0)),
        UserSymbol::new("D_8008CFE0".into(), Vram::new(0x8008CFE0)),
        UserSymbol::new("pause_table".into(), Vram::new(0x8008CFF0)).with_size(Size::new(0x30)),
        UserSymbol::new("cont_table".into(), Vram::new(0x8008D020)).with_size(Size::new(0x60)),
        UserSymbol::new("etc_parts_tbl".into(), Vram::new(0x8008D080)).with_size(Size::new(0x40)),
        UserSymbol::new("x2p".into(), Vram::new(0x8008D0C0)),
        UserSymbol::new("x4p".into(), Vram::new(0x8008D0C8)),
        UserSymbol::new("etc_vp".into(), Vram::new(0x8008D0D8)),
        UserSymbol::new("etc_setup".into(), Vram::new(0x8008D0E8)).with_size(Size::new(0x98)),
        UserSymbol::new("col_prim_434".into(), Vram::new(0x8008D180)).with_size(Size::new(0x30)),
        UserSymbol::new("col_env_435".into(), Vram::new(0x8008D1B0)).with_size(Size::new(0x30)),
        UserSymbol::new("basc".into(), Vram::new(0x8008D1E0)),
        UserSymbol::new("BASC".into(), Vram::new(0x8008D1F4)),
        UserSymbol::new("_div_data".into(), Vram::new(0x8008D208))
            .with_size(Size::new(0x48))
            .with_typ(SymbolType::Float64),
        UserSymbol::new("_mul_data".into(), Vram::new(0x8008D250))
            .with_size(Size::new(0x48))
            .with_typ(SymbolType::Float64),
        UserSymbol::new("D_8008D2A0".into(), Vram::new(0x8008D2A0))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("D_8008D2A8".into(), Vram::new(0x8008D2A8)),
        UserSymbol::new("D_8008D2C8".into(), Vram::new(0x8008D2C8))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("sDebugPrintFontTex".into(), Vram::new(0x8008D2D0))
            .with_size(Size::new(0xF00)),
        UserSymbol::new("sDebugPrintFontMap".into(), Vram::new(0x8008E1D0)),
        UserSymbol::new("D_8008E290".into(), Vram::new(0x8008E290)),
        UserSymbol::new("D_8008E340".into(), Vram::new(0x8008E340)),
        UserSymbol::new("sPrinterCallbacks".into(), Vram::new(0x8008E358)),
        UserSymbol::new("sPrinterInstance".into(), Vram::new(0x8008E364)),
        UserSymbol::new("D_8008E370".into(), Vram::new(0x8008E370)),
        UserSymbol::new("D_8008E380".into(), Vram::new(0x8008E380)),
        UserSymbol::new("D_8008E398".into(), Vram::new(0x8008E398)),
        UserSymbol::new("D_8008E3B8".into(), Vram::new(0x8008E3B8)).with_size(Size::new(0x8)),
        UserSymbol::new("D_8008E3C0".into(), Vram::new(0x8008E3C0)),
        UserSymbol::new("D_8008E400".into(), Vram::new(0x8008E400)),
        UserSymbol::new("D_8008E420".into(), Vram::new(0x8008E420)),
        UserSymbol::new("D_8008E454".into(), Vram::new(0x8008E454)),
        UserSymbol::new("D_8008E480".into(), Vram::new(0x8008E480)),
        UserSymbol::new("D_8008E490".into(), Vram::new(0x8008E490)),
        UserSymbol::new("D_8008E4A0".into(), Vram::new(0x8008E4A0)),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_CharacterNames".into(),
            Vram::new(0x8008E4AC),
        ),
        UserSymbol::new("D_8008E4DC".into(), Vram::new(0x8008E4DC)),
        UserSymbol::new("D_8008E4F0".into(), Vram::new(0x8008E4F0)),
        UserSymbol::new("D_8008E508".into(), Vram::new(0x8008E508)),
        UserSymbol::new("D_8008E538".into(), Vram::new(0x8008E538)),
        UserSymbol::new("D_8008E540".into(), Vram::new(0x8008E540)),
        UserSymbol::new("D_8008E548".into(), Vram::new(0x8008E548)).with_size(Size::new(0x10)),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_RowsPerColumn".into(),
            Vram::new(0x8008E558),
        )
        .with_size(Size::new(0x10)),
        UserSymbol::new("D_8008E568".into(), Vram::new(0x8008E568)),
        UserSymbol::new("D_8008E574".into(), Vram::new(0x8008E574)),
        UserSymbol::new("D_8008E57C".into(), Vram::new(0x8008E57C)),
        UserSymbol::new("D_8008E584".into(), Vram::new(0x8008E584)),
        UserSymbol::new("D_8008E590".into(), Vram::new(0x8008E590)),
        UserSymbol::new("D_8008E598".into(), Vram::new(0x8008E598)),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_ColumnNames".into(),
            Vram::new(0x8008E5A0),
        ),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_DefaultColor".into(),
            Vram::new(0x8008E5B0),
        ),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_ColumnWidths".into(),
            Vram::new(0x8008E5B4),
        ),
        UserSymbol::new("sBgTasksManager_Initialized".into(), Vram::new(0x8008E5D0)),
        UserSymbol::new("copy_texture_init_dl".into(), Vram::new(0x8008E5E0)),
        UserSymbol::new("alpha_texture_init_dl".into(), Vram::new(0x8008E650)),
        UserSymbol::new("normal_texture_init_dl".into(), Vram::new(0x8008E6B8)),
        UserSymbol::new("D_8008E728".into(), Vram::new(0x8008E728)),
        UserSymbol::new("init_dl_155".into(), Vram::new(0x8008E748)),
        UserSymbol::new("_yn_1767".into(), Vram::new(0x8008E770)),
        UserSymbol::new("_type_1949".into(), Vram::new(0x8008E778)),
        UserSymbol::new("_menuMain_lastMode".into(), Vram::new(0x8008E788))
            .with_size(Size::new(0x4)),
        UserSymbol::new("_tblMain_5279".into(), Vram::new(0x8008E78C)),
        UserSymbol::new("_tblPlay1_5280".into(), Vram::new(0x8008E79C)),
        UserSymbol::new("_tblPlay2_5281".into(), Vram::new(0x8008E7B4)).with_size(Size::new(0xC)),
        UserSymbol::new("D_8008E7C0".into(), Vram::new(0x8008E7C0)),
        UserSymbol::new("_tblBaTyp_5282".into(), Vram::new(0x8008E7D0)),
        UserSymbol::new("_tblOpt_5283".into(), Vram::new(0x8008E7DC)),
        UserSymbol::new("_tblTutol_5284".into(), Vram::new(0x8008E7F0)),
        UserSymbol::new("_tblSound_5285".into(), Vram::new(0x8008E804)),
        UserSymbol::new("_tblMisc_5286".into(), Vram::new(0x8008E814)),
        UserSymbol::new("_loopTbl_5287".into(), Vram::new(0x8008E820)).with_size(Size::new(0x40)),
        UserSymbol::new("mes_5577".into(), Vram::new(0x8008E860)).with_size(Size::new(0xC)),
        UserSymbol::new("_nameEntry_charTable".into(), Vram::new(0x8008E86C)),
        UserSymbol::new("_code_9711".into(), Vram::new(0x8008E870))
            .with_size(Size::new(0x26))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("_posX_tbl_9716".into(), Vram::new(0x8008E898)),
        UserSymbol::new("_menuAll_lastMode".into(), Vram::new(0x8008E8B4))
            .with_size(Size::new(0x4)),
        UserSymbol::new("font_a_tex".into(), Vram::new(0x8008E8C0)).with_typ(SymbolType::Byte),
        UserSymbol::new("font_2_tex".into(), Vram::new(0x80094350)).with_typ(SymbolType::Byte),
        UserSymbol::new("font_e_tex".into(), Vram::new(0x8009F870))
            .with_typ(SymbolType::Byte)
            .with_dont_allow_addend(),
        UserSymbol::new("font_e2_tex".into(), Vram::new(0x800A0E78)).with_typ(SymbolType::Byte),
        UserSymbol::new("font_grade_tex".into(), Vram::new(0x800A3A88)).with_typ(SymbolType::Byte),
        UserSymbol::new("D_800A3AD0".into(), Vram::new(0x800A3AD0)).with_typ(SymbolType::Byte),
        UserSymbol::new("D_800A3BD0".into(), Vram::new(0x800A3BD0)).with_typ(SymbolType::Byte),
        UserSymbol::new("char_code_tbl".into(), Vram::new(0x800A3CD0)),
        UserSymbol::new("D_800A6D90".into(), Vram::new(0x800A6D90)),
        UserSymbol::new("_tbl_133".into(), Vram::new(0x800A6F70)),
        UserSymbol::new("virus_anime_table".into(), Vram::new(0x800A6F80))
            .with_size(Size::new(0xC)),
        UserSymbol::new("dm_chaine_se_table_vs_178".into(), Vram::new(0x800A6F8C)),
        UserSymbol::new("dm_chaine_se_table_4p_179".into(), Vram::new(0x800A6F90)),
        UserSymbol::new("rotate_table_474".into(), Vram::new(0x800A6F9C))
            .with_size(Size::new(0x10)),
        UserSymbol::new("rotate_mtx_475".into(), Vram::new(0x800A6FAC)).with_size(Size::new(0x18)),
        UserSymbol::new("D_800A6FC4".into(), Vram::new(0x800A6FC4)),
        UserSymbol::new("visible_fall_point".into(), Vram::new(0x800A6FC8)),
        UserSymbol::new("_mesPassword".into(), Vram::new(0x800A6FD8)).with_typ(SymbolType::CString),
        UserSymbol::new("_mesEasyCont".into(), Vram::new(0x800A6FEC)).with_typ(SymbolType::CString),
        UserSymbol::new("_mesEasyNoCont".into(), Vram::new(0x800A7040))
            .with_typ(SymbolType::CString),
        UserSymbol::new("_mesNormalCont".into(), Vram::new(0x800A709C))
            .with_typ(SymbolType::CString),
        UserSymbol::new("_mesNormalNoCont".into(), Vram::new(0x800A70F4))
            .with_typ(SymbolType::CString),
        UserSymbol::new("_mesHardCont".into(), Vram::new(0x800A7150)).with_typ(SymbolType::CString),
        UserSymbol::new("_mesHardNoCont".into(), Vram::new(0x800A71A4))
            .with_typ(SymbolType::CString),
        UserSymbol::new("_mesSHardCont".into(), Vram::new(0x800A727C))
            .with_typ(SymbolType::CString),
        UserSymbol::new("_mesSHardNoCont".into(), Vram::new(0x800A72D8))
            .with_typ(SymbolType::CString),
        UserSymbol::new("_n_1199".into(), Vram::new(0x800A7354)),
        UserSymbol::new("D_800A7360".into(), Vram::new(0x800A7360)),
        UserSymbol::new("tbl_4589".into(), Vram::new(0x800A7374)),
        UserSymbol::new("col_4590".into(), Vram::new(0x800A7378))
            .with_size(Size::new(0xC))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("title_demo_flg".into(), Vram::new(0x800A7390)),
        UserSymbol::new("title_demo_no".into(), Vram::new(0x800A7394)),
        UserSymbol::new("title_manual_no".into(), Vram::new(0x800A7398)),
        UserSymbol::new("title_exit_flag".into(), Vram::new(0x800A739C)),
        UserSymbol::new("D_800A73A0".into(), Vram::new(0x800A73A0)),
        UserSymbol::new("D_800A73B8".into(), Vram::new(0x800A73B8))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("curtain_alpha_00_tex".into(), Vram::new(0x800A73C0)),
        UserSymbol::new("curtain_00_tex".into(), Vram::new(0x800A76C0)),
        UserSymbol::new("changestar_tex".into(), Vram::new(0x800A82C0)),
        UserSymbol::new("title_bmp_tbl".into(), Vram::new(0x800A8AC0)).with_size(Size::new(0xC)),
        UserSymbol::new("mess_heap".into(), Vram::new(0x800A8ACC)),
        UserSymbol::new("st_staffroll_txt".into(), Vram::new(0x800A8AD0))
            .with_typ(SymbolType::CString),
        UserSymbol::new("mes_mo1_1".into(), Vram::new(0x800A923C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_mo1_2".into(), Vram::new(0x800A9278)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_mo1_data".into(), Vram::new(0x800A92C0)),
        UserSymbol::new("mes_wo1_1".into(), Vram::new(0x800A92D8)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_wo1_2".into(), Vram::new(0x800A9314)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_wo1_3".into(), Vram::new(0x800A935C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_wo1_data".into(), Vram::new(0x800A9390)),
        UserSymbol::new("mes_o2_1".into(), Vram::new(0x800A93B0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o2_2".into(), Vram::new(0x800A93F0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o2_3".into(), Vram::new(0x800A9418)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o2_3a".into(), Vram::new(0x800A9444)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o2_4".into(), Vram::new(0x800A9484)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o2_5".into(), Vram::new(0x800A94D4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o2_data".into(), Vram::new(0x800A9520)),
        UserSymbol::new("mes_o3_1".into(), Vram::new(0x800A9558)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o3_2".into(), Vram::new(0x800A959C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_o3_data".into(), Vram::new(0x800A95D4)),
        UserSymbol::new("mes_01_1".into(), Vram::new(0x800A95EC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_2".into(), Vram::new(0x800A962C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_3".into(), Vram::new(0x800A965C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_4".into(), Vram::new(0x800A96A0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_5".into(), Vram::new(0x800A96D8)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_6".into(), Vram::new(0x800A9720)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_7".into(), Vram::new(0x800A9764)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_01_data".into(), Vram::new(0x800A97AC)),
        UserSymbol::new("mes_m02a_1".into(), Vram::new(0x800A97EC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02a_2".into(), Vram::new(0x800A9838)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02a_data".into(), Vram::new(0x800A9878)),
        UserSymbol::new("mes_w02a_1".into(), Vram::new(0x800A9890)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w02a_2".into(), Vram::new(0x800A98D4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w02a_data".into(), Vram::new(0x800A9918)),
        UserSymbol::new("mes_m02b_1".into(), Vram::new(0x800A9930)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02b_2".into(), Vram::new(0x800A9974)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02b_3".into(), Vram::new(0x800A99B0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02b_4".into(), Vram::new(0x800A99EC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02b_5".into(), Vram::new(0x800A9A18)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02b_6".into(), Vram::new(0x800A9A50)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m02b_data".into(), Vram::new(0x800A9A94)),
        UserSymbol::new("mes_w02b_1".into(), Vram::new(0x800A9ACC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w02b_1a".into(), Vram::new(0x800A9AF4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w02b_2".into(), Vram::new(0x800A9B30)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w02b_3".into(), Vram::new(0x800A9B6C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w02b_data".into(), Vram::new(0x800A9BB8)),
        UserSymbol::new("mes_m03_1".into(), Vram::new(0x800A9BE0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m03_2".into(), Vram::new(0x800A9C18)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m03_3".into(), Vram::new(0x800A9C58)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m03_4".into(), Vram::new(0x800A9C74)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m03_5".into(), Vram::new(0x800A9CA0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m03_data".into(), Vram::new(0x800A9CF0)),
        UserSymbol::new("mes_w03_1".into(), Vram::new(0x800A9D20)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w03_2".into(), Vram::new(0x800A9D5C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w03_3".into(), Vram::new(0x800A9DA0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w03_data".into(), Vram::new(0x800A9DDC)),
        UserSymbol::new("mes_m04_1".into(), Vram::new(0x800A9DFC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m04_2".into(), Vram::new(0x800A9E44)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m04_3".into(), Vram::new(0x800A9E7C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m04_4".into(), Vram::new(0x800A9EB8)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m04_data".into(), Vram::new(0x800A9ED4)),
        UserSymbol::new("mes_w04_1".into(), Vram::new(0x800A9EFC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w04_2".into(), Vram::new(0x800A9F48)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w04_3".into(), Vram::new(0x800A9F7C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w04_data".into(), Vram::new(0x800A9FB4)),
        UserSymbol::new("mes_m05_1".into(), Vram::new(0x800A9FD4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m05_2".into(), Vram::new(0x800AA014)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m05_3".into(), Vram::new(0x800AA040)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m05_4".into(), Vram::new(0x800AA07C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m05_data".into(), Vram::new(0x800AA09C)),
        UserSymbol::new("mes_w05_1".into(), Vram::new(0x800AA0C4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w05_2".into(), Vram::new(0x800AA104)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w05_3".into(), Vram::new(0x800AA130)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w05_4".into(), Vram::new(0x800AA168)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w05_data".into(), Vram::new(0x800AA194)),
        UserSymbol::new("mes_m06_1".into(), Vram::new(0x800AA1BC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m06_2".into(), Vram::new(0x800AA200)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m06_3".into(), Vram::new(0x800AA238)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m06_4".into(), Vram::new(0x800AA26C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m06_data".into(), Vram::new(0x800AA29C)),
        UserSymbol::new("mes_w06_1".into(), Vram::new(0x800AA2C4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w06_2".into(), Vram::new(0x800AA300)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w06_3".into(), Vram::new(0x800AA318)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w06_data".into(), Vram::new(0x800AA334)),
        UserSymbol::new("mes_07_1".into(), Vram::new(0x800AA354)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_07_2".into(), Vram::new(0x800AA398)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_07_3".into(), Vram::new(0x800AA3DC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_07E_3".into(), Vram::new(0x800AA41C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_07_data".into(), Vram::new(0x800AA454)),
        UserSymbol::new("mes_07E_data".into(), Vram::new(0x800AA474)),
        UserSymbol::new("mes_08a_1".into(), Vram::new(0x800AA494)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_08a_data".into(), Vram::new(0x800AA4DC)),
        UserSymbol::new("mes_08Ea_1".into(), Vram::new(0x800AA4EC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_08Ea_data".into(), Vram::new(0x800AA528)),
        UserSymbol::new("mes_08b_1".into(), Vram::new(0x800AA538)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_08b_2".into(), Vram::new(0x800AA570)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_08b_data".into(), Vram::new(0x800AA5AC)),
        UserSymbol::new("mes_m09_2".into(), Vram::new(0x800AA5C4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m09_3".into(), Vram::new(0x800AA604)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m09_4".into(), Vram::new(0x800AA63C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m09_data".into(), Vram::new(0x800AA668)),
        UserSymbol::new("mes_w09_2".into(), Vram::new(0x800AA688)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w09_3".into(), Vram::new(0x800AA6CC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w09_4".into(), Vram::new(0x800AA714)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w09_data".into(), Vram::new(0x800AA758)),
        UserSymbol::new("mes_m0e1_1".into(), Vram::new(0x800AA778)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e1_2".into(), Vram::new(0x800AA7C4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e1_3".into(), Vram::new(0x800AA808)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e1_data".into(), Vram::new(0x800AA844)),
        UserSymbol::new("mes_m0e2_1".into(), Vram::new(0x800AA864)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e2_2".into(), Vram::new(0x800AA894)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e2_3".into(), Vram::new(0x800AA8CC)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e2_4".into(), Vram::new(0x800AA8F0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e2_5".into(), Vram::new(0x800AA928)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e2_6".into(), Vram::new(0x800AA96C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e2_data".into(), Vram::new(0x800AA9A8)),
        UserSymbol::new("mes_m0e_1".into(), Vram::new(0x800AA9E0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e_2".into(), Vram::new(0x800AAA10)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e_3".into(), Vram::new(0x800AAA4C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_m0e_data".into(), Vram::new(0x800AAA80)),
        UserSymbol::new("mes_w0e1_1".into(), Vram::new(0x800AAAA0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w0e1_2".into(), Vram::new(0x800AAAE0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w0e1_3".into(), Vram::new(0x800AAB18)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w0e1_data".into(), Vram::new(0x800AAB5C)),
        UserSymbol::new("mes_w0e2_1".into(), Vram::new(0x800AAB7C)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w0e2_2".into(), Vram::new(0x800AABB4)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_w0e2_data".into(), Vram::new(0x800AABC8)),
        UserSymbol::new("mes_ep1".into(), Vram::new(0x800AABE0)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_ep2".into(), Vram::new(0x800AAC10)).with_typ(SymbolType::CString),
        UserSymbol::new("mes_ep_data".into(), Vram::new(0x800AAC5C)),
        UserSymbol::new("mes_data".into(), Vram::new(0x800AAC74)).with_size(Size::new(0x78)),
        UserSymbol::new("story_proc_no".into(), Vram::new(0x800AACEC)),
        UserSymbol::new("STR_800AACF0".into(), Vram::new(0x800AACF0)).with_typ(SymbolType::CString),
        UserSymbol::new("EndingLastMessage".into(), Vram::new(0x800AAD04)),
        UserSymbol::new("story_time_cnt".into(), Vram::new(0x800AAD08)),
        UserSymbol::new("story_seq_step".into(), Vram::new(0x800AAD0C)),
        UserSymbol::new("story_zoom".into(), Vram::new(0x800AAD10)),
        UserSymbol::new("story_curtain".into(), Vram::new(0x800AAD14)),
        UserSymbol::new("story_spot_cnt".into(), Vram::new(0x800AAD18)),
        UserSymbol::new("story_kay_wait".into(), Vram::new(0x800AAD1C)),
        UserSymbol::new("story_message_on".into(), Vram::new(0x800AAD20)),
        UserSymbol::new("story_message_start".into(), Vram::new(0x800AAD24)),
        UserSymbol::new("story_doing".into(), Vram::new(0x800AAD28)),
        UserSymbol::new("bgtime".into(), Vram::new(0x800AAD2C)),
        UserSymbol::new("D_800AAD30".into(), Vram::new(0x800AAD30)),
        UserSymbol::new("mes_time".into(), Vram::new(0x800AAD34)),
        UserSymbol::new("story_read_buf".into(), Vram::new(0x800AAD38)),
        UserSymbol::new("story_buffer".into(), Vram::new(0x800AAD3C)),
        UserSymbol::new("story_z_buffer".into(), Vram::new(0x800AAD40)),
        UserSymbol::new("objMtx_FF".into(), Vram::new(0x800AAD44)),
        UserSymbol::new("wakuGraphic_ofs".into(), Vram::new(0x800AAD48)).with_size(Size::new(0xC)),
        UserSymbol::new("D_800AAD58".into(), Vram::new(0x800AAD58)),
        UserSymbol::new("story_setup".into(), Vram::new(0x800AAD68)),
        UserSymbol::new("snd_tbl_838".into(), Vram::new(0x800AAE00)),
        UserSymbol::new("D_800AAE60".into(), Vram::new(0x800AAE60)),
        UserSymbol::new("D_800AAFF4".into(), Vram::new(0x800AAFF4)),
        UserSymbol::new("D_800AB170".into(), Vram::new(0x800AB170)).with_typ(SymbolType::CString),
        UserSymbol::new("D_800AB1B4".into(), Vram::new(0x800AB1B4)),
        UserSymbol::new("D_800AB1D4".into(), Vram::new(0x800AB1D4)).with_typ(SymbolType::CString),
        UserSymbol::new("D_800AB1EC".into(), Vram::new(0x800AB1EC)),
        UserSymbol::new("D_800AB1FC".into(), Vram::new(0x800AB1FC)),
        UserSymbol::new("D_800AB20C".into(), Vram::new(0x800AB20C)),
        UserSymbol::new("D_800AB21C".into(), Vram::new(0x800AB21C)),
        UserSymbol::new("D_800AB29C".into(), Vram::new(0x800AB29C)),
        UserSymbol::new("D_800AB320".into(), Vram::new(0x800AB320)),
        UserSymbol::new("D_800AB32C".into(), Vram::new(0x800AB32C)),
        UserSymbol::new("D_800AB334".into(), Vram::new(0x800AB334)),
        UserSymbol::new("D_800AB3B4".into(), Vram::new(0x800AB3B4)),
        UserSymbol::new("D_800AB440".into(), Vram::new(0x800AB440)).with_size(Size::new(0x810)),
        UserSymbol::new("jumptable".into(), Vram::new(0x800ABC50)),
        UserSymbol::new("SmallRoomParams".into(), Vram::new(0x800ABD10)),
        UserSymbol::new("BigRoomParams".into(), Vram::new(0x800ABD78)),
        UserSymbol::new("EchoParams".into(), Vram::new(0x800ABE00)),
        UserSymbol::new("ChorusParams".into(), Vram::new(0x800ABE28)),
        UserSymbol::new("FlangeParams".into(), Vram::new(0x800ABEB0)),
        UserSymbol::new("NoFxParams".into(), Vram::new(0x800ABED8)),
        UserSymbol::new("EffectCount".into(), Vram::new(0x800ABF00)),
        UserSymbol::new("EffectList".into(), Vram::new(0x800ABF04)),
        UserSymbol::new("default_sched".into(), Vram::new(0x800ABF20)),
        UserSymbol::new("__libmus_current_sched".into(), Vram::new(0x800ABF2C)),
        UserSymbol::new("last_task".into(), Vram::new(0x800ABF30)).with_size(Size::new(0x4)),
        UserSymbol::new("aspMainDataStart".into(), Vram::new(0x800ABF40))
            .with_name_end("aspMainDataEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("gspF3DEX2_fifoDataStart".into(), Vram::new(0x800AC200))
            .with_name_end("gspF3DEX2_fifoDataEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("gspS2DEX_fifoDataStart".into(), Vram::new(0x800AC620))
            .with_name_end("gspS2DEX_fifoDataEnd".into())
            .with_dont_allow_addend(),
        UserSymbol::new("jtbl_800AC9E0".into(), Vram::new(0x800AC9E0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_charSE_tbl".into(), Vram::new(0x800ACA10)),
        UserSymbol::new("RO_800ACA20".into(), Vram::new(0x800ACA20)).with_size(Size::new(0x60)),
        UserSymbol::new("gSndsEntries".into(), Vram::new(0x800ACA80)),
        UserSymbol::new("DBL_800ACCF0".into(), Vram::new(0x800ACCF0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ACCF8".into(), Vram::new(0x800ACCF8)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800ACD00".into(), Vram::new(0x800ACD00))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACD20".into(), Vram::new(0x800ACD20))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACD60".into(), Vram::new(0x800ACD60))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACDA0".into(), Vram::new(0x800ACDA0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACDE0".into(), Vram::new(0x800ACDE0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_seqTbl_224".into(), Vram::new(0x800ACE1C))
            .with_size(Size::new(0xA))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("_demoSeqTbl_225".into(), Vram::new(0x800ACE28))
            .with_size(Size::new(0x2))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("StoryVirLv_226".into(), Vram::new(0x800ACE2C))
            .with_size(Size::new(0x1E))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("_charToAi_227".into(), Vram::new(0x800ACE4C))
            .with_size(Size::new(0xF))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("_stageToChar_tbl_228".into(), Vram::new(0x800ACE5C))
            .with_size(Size::new(0x14))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("_story4PChar_tbl_229".into(), Vram::new(0x800ACE70))
            .with_size(Size::new(0x8))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("_team_flg_230".into(), Vram::new(0x800ACE78))
            .with_size(Size::new(0xC))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("GameSize_231".into(), Vram::new(0x800ACE84))
            .with_size(Size::new(0x7))
            .with_typ(SymbolType::Byte),
        UserSymbol::new("jtbl_800ACE90".into(), Vram::new(0x800ACE90))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800ACEB0".into(), Vram::new(0x800ACEB0)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800ACEB8".into(), Vram::new(0x800ACEB8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACEE8".into(), Vram::new(0x800ACEE8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("RO_800ACF10".into(), Vram::new(0x800ACF10)),
        UserSymbol::new("wave_tbl_2879".into(), Vram::new(0x800ACF20)).with_size(Size::new(0x20)),
        UserSymbol::new("eeprom_header".into(), Vram::new(0x800ACF40)),
        UserSymbol::new("eeprom_header_bits".into(), Vram::new(0x800ACF44)),
        UserSymbol::new("_defName".into(), Vram::new(0x800ACF48)),
        UserSymbol::new("jtbl_800ACF50".into(), Vram::new(0x800ACF50))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACF68".into(), Vram::new(0x800ACF68))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACF80".into(), Vram::new(0x800ACF80))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800ACF98".into(), Vram::new(0x800ACF98))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("RO_800ACFB0".into(), Vram::new(0x800ACFB0)).with_size(Size::new(0x18)),
        UserSymbol::new("RO_800ACFC8".into(), Vram::new(0x800ACFC8)),
        UserSymbol::new("DBL_800AD068".into(), Vram::new(0x800AD068)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD070".into(), Vram::new(0x800AD070)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD078".into(), Vram::new(0x800AD078)).with_typ(SymbolType::Float64),
        UserSymbol::new("RO_800AD080".into(), Vram::new(0x800AD080)).with_size(Size::new(0x18)),
        UserSymbol::new("RO_800AD098".into(), Vram::new(0x800AD098)),
        UserSymbol::new("DBL_800AD138".into(), Vram::new(0x800AD138)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD140".into(), Vram::new(0x800AD140)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD150".into(), Vram::new(0x800AD150)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD158".into(), Vram::new(0x800AD158)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD160".into(), Vram::new(0x800AD160)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD168".into(), Vram::new(0x800AD168)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD170".into(), Vram::new(0x800AD170)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD178".into(), Vram::new(0x800AD178)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD180".into(), Vram::new(0x800AD180)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD188".into(), Vram::new(0x800AD188)).with_typ(SymbolType::Float64),
        UserSymbol::new("RO_800AD190".into(), Vram::new(0x800AD190)),
        UserSymbol::new("RO_800AD194".into(), Vram::new(0x800AD194)),
        UserSymbol::new("DBL_800AD198".into(), Vram::new(0x800AD198)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD1A0".into(), Vram::new(0x800AD1A0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD1A8".into(), Vram::new(0x800AD1A8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD1B0".into(), Vram::new(0x800AD1B0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AD1B8".into(), Vram::new(0x800AD1B8)).with_typ(SymbolType::Float64),
        UserSymbol::new("RO_800AD1C0".into(), Vram::new(0x800AD1C0)),
        UserSymbol::new("jtbl_800AD1C8".into(), Vram::new(0x800AD1C8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800AD210".into(), Vram::new(0x800AD210))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("STR_800AD2E0".into(), Vram::new(0x800AD2E0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD2F0".into(), Vram::new(0x800AD2F0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD2FC".into(), Vram::new(0x800AD2FC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD30C".into(), Vram::new(0x800AD30C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD318".into(), Vram::new(0x800AD318))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD324".into(), Vram::new(0x800AD324))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD330".into(), Vram::new(0x800AD330))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD338".into(), Vram::new(0x800AD338))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD344".into(), Vram::new(0x800AD344))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD350".into(), Vram::new(0x800AD350))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD358".into(), Vram::new(0x800AD358))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD364".into(), Vram::new(0x800AD364))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD36C".into(), Vram::new(0x800AD36C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD374".into(), Vram::new(0x800AD374))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD37C".into(), Vram::new(0x800AD37C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD380".into(), Vram::new(0x800AD380))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD384".into(), Vram::new(0x800AD384))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD38C".into(), Vram::new(0x800AD38C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD394".into(), Vram::new(0x800AD394))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD39C".into(), Vram::new(0x800AD39C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3A4".into(), Vram::new(0x800AD3A4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3AC".into(), Vram::new(0x800AD3AC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3B4".into(), Vram::new(0x800AD3B4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3BC".into(), Vram::new(0x800AD3BC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3CC".into(), Vram::new(0x800AD3CC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3D4".into(), Vram::new(0x800AD3D4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3E0".into(), Vram::new(0x800AD3E0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3E8".into(), Vram::new(0x800AD3E8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD3F4".into(), Vram::new(0x800AD3F4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD400".into(), Vram::new(0x800AD400))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD408".into(), Vram::new(0x800AD408))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD414".into(), Vram::new(0x800AD414))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD41C".into(), Vram::new(0x800AD41C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD428".into(), Vram::new(0x800AD428))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD430".into(), Vram::new(0x800AD430))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD438".into(), Vram::new(0x800AD438))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD440".into(), Vram::new(0x800AD440))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD448".into(), Vram::new(0x800AD448))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD450".into(), Vram::new(0x800AD450))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD458".into(), Vram::new(0x800AD458))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD460".into(), Vram::new(0x800AD460))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD46C".into(), Vram::new(0x800AD46C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD474".into(), Vram::new(0x800AD474))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD47C".into(), Vram::new(0x800AD47C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD484".into(), Vram::new(0x800AD484))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD48C".into(), Vram::new(0x800AD48C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD494".into(), Vram::new(0x800AD494))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4A0".into(), Vram::new(0x800AD4A0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4A8".into(), Vram::new(0x800AD4A8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4B0".into(), Vram::new(0x800AD4B0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4C0".into(), Vram::new(0x800AD4C0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4D0".into(), Vram::new(0x800AD4D0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4D8".into(), Vram::new(0x800AD4D8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4E4".into(), Vram::new(0x800AD4E4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4F0".into(), Vram::new(0x800AD4F0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD4F8".into(), Vram::new(0x800AD4F8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD504".into(), Vram::new(0x800AD504))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD50C".into(), Vram::new(0x800AD50C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD514".into(), Vram::new(0x800AD514))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD520".into(), Vram::new(0x800AD520))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD528".into(), Vram::new(0x800AD528))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD530".into(), Vram::new(0x800AD530))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD53C".into(), Vram::new(0x800AD53C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD544".into(), Vram::new(0x800AD544))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD54C".into(), Vram::new(0x800AD54C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD554".into(), Vram::new(0x800AD554))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD55C".into(), Vram::new(0x800AD55C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD560".into(), Vram::new(0x800AD560))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD56C".into(), Vram::new(0x800AD56C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD574".into(), Vram::new(0x800AD574))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD59C".into(), Vram::new(0x800AD59C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD5A8".into(), Vram::new(0x800AD5A8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD5AC".into(), Vram::new(0x800AD5AC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD5D4".into(), Vram::new(0x800AD5D4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD614".into(), Vram::new(0x800AD614))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD648".into(), Vram::new(0x800AD648))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD658".into(), Vram::new(0x800AD658))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD664".into(), Vram::new(0x800AD664))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD698".into(), Vram::new(0x800AD698))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD6A4".into(), Vram::new(0x800AD6A4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD6B0".into(), Vram::new(0x800AD6B0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD728".into(), Vram::new(0x800AD728))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD74C".into(), Vram::new(0x800AD74C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD790".into(), Vram::new(0x800AD790))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD794".into(), Vram::new(0x800AD794))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("jtbl_800AD7C0".into(), Vram::new(0x800AD7C0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("STR_800AD7F0".into(), Vram::new(0x800AD7F0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD864".into(), Vram::new(0x800AD864))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD898".into(), Vram::new(0x800AD898))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD8CC".into(), Vram::new(0x800AD8CC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD8E8".into(), Vram::new(0x800AD8E8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD8F4".into(), Vram::new(0x800AD8F4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD904".into(), Vram::new(0x800AD904))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD914".into(), Vram::new(0x800AD914))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD91C".into(), Vram::new(0x800AD91C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD928".into(), Vram::new(0x800AD928))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD92C".into(), Vram::new(0x800AD92C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("jtbl_800AD950".into(), Vram::new(0x800AD950))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("STR_800AD980".into(), Vram::new(0x800AD980))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD98C".into(), Vram::new(0x800AD98C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD994".into(), Vram::new(0x800AD994))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD99C".into(), Vram::new(0x800AD99C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9A8".into(), Vram::new(0x800AD9A8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9AC".into(), Vram::new(0x800AD9AC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9B4".into(), Vram::new(0x800AD9B4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9C0".into(), Vram::new(0x800AD9C0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9C8".into(), Vram::new(0x800AD9C8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9D0".into(), Vram::new(0x800AD9D0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9D8".into(), Vram::new(0x800AD9D8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9E0".into(), Vram::new(0x800AD9E0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9E4".into(), Vram::new(0x800AD9E4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9EC".into(), Vram::new(0x800AD9EC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800AD9F8".into(), Vram::new(0x800AD9F8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA00".into(), Vram::new(0x800ADA00))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA08".into(), Vram::new(0x800ADA08))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA10".into(), Vram::new(0x800ADA10))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA18".into(), Vram::new(0x800ADA18))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA24".into(), Vram::new(0x800ADA24))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA28".into(), Vram::new(0x800ADA28))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA2C".into(), Vram::new(0x800ADA2C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA30".into(), Vram::new(0x800ADA30))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA3C".into(), Vram::new(0x800ADA3C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA44".into(), Vram::new(0x800ADA44))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA4C".into(), Vram::new(0x800ADA4C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA50".into(), Vram::new(0x800ADA50))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA58".into(), Vram::new(0x800ADA58))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA64".into(), Vram::new(0x800ADA64))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA6C".into(), Vram::new(0x800ADA6C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA74".into(), Vram::new(0x800ADA74))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA7C".into(), Vram::new(0x800ADA7C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA80".into(), Vram::new(0x800ADA80))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA84".into(), Vram::new(0x800ADA84))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADA8C".into(), Vram::new(0x800ADA8C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("jtbl_800ADA98".into(), Vram::new(0x800ADA98))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("STR_800ADAC8".into(), Vram::new(0x800ADAC8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADAD0".into(), Vram::new(0x800ADAD0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADADC".into(), Vram::new(0x800ADADC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADAE4".into(), Vram::new(0x800ADAE4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADAF0".into(), Vram::new(0x800ADAF0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADB04".into(), Vram::new(0x800ADB04))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADB0C".into(), Vram::new(0x800ADB0C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADB10".into(), Vram::new(0x800ADB10))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADB18".into(), Vram::new(0x800ADB18))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800ADB24".into(), Vram::new(0x800ADB24))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_modes_96".into(), Vram::new(0x800ADB40)),
        UserSymbol::new("DBL_800ADB50".into(), Vram::new(0x800ADB50)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB58".into(), Vram::new(0x800ADB58)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB60".into(), Vram::new(0x800ADB60)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB68".into(), Vram::new(0x800ADB68)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB70".into(), Vram::new(0x800ADB70)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB78".into(), Vram::new(0x800ADB78)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB80".into(), Vram::new(0x800ADB80)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB88".into(), Vram::new(0x800ADB88)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB90".into(), Vram::new(0x800ADB90)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADB98".into(), Vram::new(0x800ADB98)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADBA0".into(), Vram::new(0x800ADBA0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADBA8".into(), Vram::new(0x800ADBA8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADBB0".into(), Vram::new(0x800ADBB0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADBB8".into(), Vram::new(0x800ADBB8)).with_typ(SymbolType::Float64),
        UserSymbol::new("_pnts_871".into(), Vram::new(0x800ADBC0)),
        UserSymbol::new("DBL_800ADC10".into(), Vram::new(0x800ADC10)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC18".into(), Vram::new(0x800ADC18)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC20".into(), Vram::new(0x800ADC20)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC28".into(), Vram::new(0x800ADC28)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC30".into(), Vram::new(0x800ADC30)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC38".into(), Vram::new(0x800ADC38)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC40".into(), Vram::new(0x800ADC40)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800ADC48".into(), Vram::new(0x800ADC48)).with_typ(SymbolType::Float64),
        UserSymbol::new("color_251".into(), Vram::new(0x800ADC50))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("fade_normal_texture_init_dl".into(), Vram::new(0x800ADC60))
            .with_size(Size::new(0x18))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("fade_alpha_texture_init_dl".into(), Vram::new(0x800ADC78))
            .with_size(Size::new(0x18))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new(
            "fade_intensity_texture_init_dl".into(),
            Vram::new(0x800ADC90),
        )
        .with_size(Size::new(0x28))
        .with_typ(SymbolType::UserCustom),
        UserSymbol::new("fade_fillrect_init_dl".into(), Vram::new(0x800ADCB8))
            .with_size(Size::new(0x28))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("fade_shadow_texture_init_dl".into(), Vram::new(0x800ADCE0))
            .with_size(Size::new(0x20))
            .with_typ(SymbolType::UserCustom)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("FLT_800ADD00".into(), Vram::new(0x800ADD00)).with_typ(SymbolType::Float32),
        UserSymbol::new("_timeAttack_levelTable".into(), Vram::new(0x800ADD3C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesP1".into(), Vram::new(0x800ADD40))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mes2P".into(), Vram::new(0x800ADD60))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mes4P".into(), Vram::new(0x800ADD80))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesOpt".into(), Vram::new(0x800ADDBC))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesSt".into(), Vram::new(0x800ADDF0))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesLv".into(), Vram::new(0x800ADF2C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVsCom".into(), Vram::new(0x800AE070))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVcFlash".into(), Vram::new(0x800AE1D8))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesLvTaiQ".into(), Vram::new(0x800AE33C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesLvTimeAt".into(), Vram::new(0x800AE4C8))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNs1".into(), Vram::new(0x800AE5E8))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNs2".into(), Vram::new(0x800AE650))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVsMan".into(), Vram::new(0x800AE66C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVmFlash".into(), Vram::new(0x800AE794))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVmTimeAt".into(), Vram::new(0x800AE898))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVsNum".into(), Vram::new(0x800AE9B8))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVs4P".into(), Vram::new(0x800AE9CC))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mes4PTag".into(), Vram::new(0x800AEB3C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mes4PFlash".into(), Vram::new(0x800AED50))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRec".into(), Vram::new(0x800AEEA4))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesTutol".into(), Vram::new(0x800AEED0))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesName".into(), Vram::new(0x800AEF04))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVsCnt".into(), Vram::new(0x800AEF38))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesSnd".into(), Vram::new(0x800AEF74))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesBak".into(), Vram::new(0x800AEFE0))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesMisc".into(), Vram::new(0x800AF014))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRecMs1".into(), Vram::new(0x800AF048))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRecMs2".into(), Vram::new(0x800AF070))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRule1".into(), Vram::new(0x800AF09C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRule2".into(), Vram::new(0x800AF0CC))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRule3".into(), Vram::new(0x800AF0FC))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesRule4".into(), Vram::new(0x800AF134))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesKotu".into(), Vram::new(0x800AF16C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNameSel".into(), Vram::new(0x800AF18C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNameErase".into(), Vram::new(0x800AF1BC))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNameErase2".into(), Vram::new(0x800AF234))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesVsCnt2".into(), Vram::new(0x800AF268))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesSound".into(), Vram::new(0x800AF29C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesStereo".into(), Vram::new(0x800AF2E0))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesMono".into(), Vram::new(0x800AF304))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesDoErase".into(), Vram::new(0x800AF324))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesDoErase1".into(), Vram::new(0x800AF390))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNoCont2".into(), Vram::new(0x800AF3C4))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNoCont3".into(), Vram::new(0x800AF3F8))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesNoCont4".into(), Vram::new(0x800AF42C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesScore".into(), Vram::new(0x800AF460))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesScoreOnOff".into(), Vram::new(0x800AF498))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesWriting_mainmenu".into(), Vram::new(0x800AF4C4))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesDeleting_mainmenu".into(), Vram::new(0x800AF4EC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_gameLvItemColor".into(), Vram::new(0x800AF514))
            .with_size(Size::new(0x18))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_musicItemColor".into(), Vram::new(0x800AF52C))
            .with_size(Size::new(0x8))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("RO_800AF534".into(), Vram::new(0x800AF534))
            .with_size(Size::new(0x4))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("jtbl_800AF538".into(), Vram::new(0x800AF538))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_menuCursor_cursor_4_pattern".into(), Vram::new(0x800AF660))
            .with_size(Size::new(0x78)),
        UserSymbol::new("_color_1040".into(), Vram::new(0x800AF6D8))
            .with_size(Size::new(0x30))
            .with_typ(SymbolType::Float32),
        UserSymbol::new("_pnts_1106".into(), Vram::new(0x800AF708)).with_size(Size::new(0x120)),
        UserSymbol::new("jtbl_800AF828".into(), Vram::new(0x800AF828))
            .with_size(Size::new(0x24))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_type_1345".into(), Vram::new(0x800AF84C)).with_size(Size::new(0x40)),
        UserSymbol::new("cap_pos_1442".into(), Vram::new(0x800AF88C)).with_size(Size::new(0x23C)),
        UserSymbol::new("_lvGauge_step".into(), Vram::new(0x800AFAC8)).with_size(Size::new(0xC)),
        UserSymbol::new("_size_1542".into(), Vram::new(0x800AFAD4)).with_size(Size::new(0x18)),
        UserSymbol::new("_type_1543".into(), Vram::new(0x800AFAEC)).with_size(Size::new(0xC)),
        UserSymbol::new("_yn_1691".into(), Vram::new(0x800AFAF8)).with_size(Size::new(0x10)),
        UserSymbol::new("STR_800AFB08".into(), Vram::new(0x800AFB08)).with_typ(SymbolType::CString),
        UserSymbol::new("STR_800AFB10".into(), Vram::new(0x800AFB10)).with_typ(SymbolType::CString),
        UserSymbol::new("_tex_1865".into(), Vram::new(0x800AFB18)).with_size(Size::new(0x20)),
        UserSymbol::new("_row_1866".into(), Vram::new(0x800AFB38)),
        UserSymbol::new("_size_1948".into(), Vram::new(0x800AFB48)).with_size(Size::new(0x20)),
        UserSymbol::new("_pos_1959".into(), Vram::new(0x800AFB68)).with_size(Size::new(0x8)),
        UserSymbol::new("DBL_800AFB70".into(), Vram::new(0x800AFB70)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AFB78".into(), Vram::new(0x800AFB78)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AFB80".into(), Vram::new(0x800AFB80)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AFB88".into(), Vram::new(0x800AFB88)).with_typ(SymbolType::Float64),
        UserSymbol::new("_seq_2327".into(), Vram::new(0x800AFB90)),
        UserSymbol::new("DBL_800AFBA0".into(), Vram::new(0x800AFBA0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800AFBA8".into(), Vram::new(0x800AFBA8)).with_typ(SymbolType::Float64),
        UserSymbol::new("_tbl_2516".into(), Vram::new(0x800AFBB0)),
        UserSymbol::new("_step_2517".into(), Vram::new(0x800AFBD8)),
        UserSymbol::new("RO_800AFBE0".into(), Vram::new(0x800AFBE0))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_posDesc_2860".into(), Vram::new(0x800AFBEC))
            .with_size(Size::new(0x28))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_posLine_2861".into(), Vram::new(0x800AFC14)).with_size(Size::new(0x8)),
        UserSymbol::new("_line_2914".into(), Vram::new(0x800AFC1C)).with_size(Size::new(0x4)),
        UserSymbol::new("_desc_2915".into(), Vram::new(0x800AFC20)),
        UserSymbol::new("_panel_3220".into(), Vram::new(0x800AFC24)).with_size(Size::new(0x14)),
        UserSymbol::new("DBL_800AFC38".into(), Vram::new(0x800AFC38)).with_typ(SymbolType::Float64),
        UserSymbol::new("_texEndOrGuest_3608".into(), Vram::new(0x800AFC40))
            .with_size(Size::new(0x8)),
        UserSymbol::new("_name_3803".into(), Vram::new(0x800AFC48)),
        UserSymbol::new("_panel_3859".into(), Vram::new(0x800AFC4C)),
        UserSymbol::new("_pos_4015".into(), Vram::new(0x800AFC5C)),
        UserSymbol::new("_y_4020".into(), Vram::new(0x800AFC64)),
        UserSymbol::new("_tbl_4036".into(), Vram::new(0x800AFC74)),
        UserSymbol::new("_panel_4108".into(), Vram::new(0x800AFCDC)),
        UserSymbol::new("_lvNumScale_4308".into(), Vram::new(0x800AFCEC))
            .with_size(Size::new(0x10))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_player_4309".into(), Vram::new(0x800AFCFC)).with_size(Size::new(0x10)),
        UserSymbol::new("_lvNum_4310".into(), Vram::new(0x800AFD0C)).with_size(Size::new(0x10)),
        UserSymbol::new("_lvGauge_4311".into(), Vram::new(0x800AFD1C)).with_size(Size::new(0x10)),
        UserSymbol::new("_speedAsk_4312".into(), Vram::new(0x800AFD2C)).with_size(Size::new(0x10)),
        UserSymbol::new("_speedItem_4313".into(), Vram::new(0x800AFD3C)).with_size(Size::new(0x10)),
        UserSymbol::new("_glvAsk_4314".into(), Vram::new(0x800AFD4C)).with_size(Size::new(0x10)),
        UserSymbol::new("_glvItem_4315".into(), Vram::new(0x800AFD5C)).with_size(Size::new(0x10)),
        UserSymbol::new("_okY_4316".into(), Vram::new(0x800AFD6C)).with_size(Size::new(0x8)),
        UserSymbol::new("_cursor_4317".into(), Vram::new(0x800AFD74)).with_size(Size::new(0x40)),
        UserSymbol::new("jtbl_800AFDB8".into(), Vram::new(0x800AFDB8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_charTbl_4601".into(), Vram::new(0x800AFE2C)).with_size(Size::new(0x3C)),
        UserSymbol::new("_type_4602".into(), Vram::new(0x800AFE68)).with_size(Size::new(0x40)),
        UserSymbol::new("_texPanelP4_4617".into(), Vram::new(0x800AFEA8)).with_size(Size::new(0x8)),
        UserSymbol::new("_texPanelP2_4618".into(), Vram::new(0x800AFEB0)),
        UserSymbol::new("_bgDataNo_to_stageNo".into(), Vram::new(0x800AFEB8))
            .with_size(Size::new(0x14)),
        UserSymbol::new("_bgCursor_4920".into(), Vram::new(0x800AFECC)).with_size(Size::new(0x10)),
        UserSymbol::new("_cursor_4921".into(), Vram::new(0x800AFEDC)).with_size(Size::new(0x20)),
        UserSymbol::new("jtbl_800AFF00".into(), Vram::new(0x800AFF00))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_root_5339".into(), Vram::new(0x800B0028)).with_size(Size::new(0x10)),
        UserSymbol::new("_play1_5340".into(), Vram::new(0x800B0038)),
        UserSymbol::new("_play2_5341".into(), Vram::new(0x800B0050)),
        UserSymbol::new("_play4_5342".into(), Vram::new(0x800B005C)),
        UserSymbol::new("_play4type_5343".into(), Vram::new(0x800B006C)),
        UserSymbol::new("_option_5344".into(), Vram::new(0x800B0078)),
        UserSymbol::new("_recMs_5345".into(), Vram::new(0x800B008C)),
        UserSymbol::new("_recMs1_5346".into(), Vram::new(0x800B0094)),
        UserSymbol::new("_recMs2_5347".into(), Vram::new(0x800B00AC)),
        UserSymbol::new("_tutorial_5348".into(), Vram::new(0x800B00B8)),
        UserSymbol::new("_count_5349".into(), Vram::new(0x800B00CC)),
        UserSymbol::new("_backup_5350".into(), Vram::new(0x800B00D8)),
        UserSymbol::new("_misc_5351".into(), Vram::new(0x800B00E0)),
        UserSymbol::new("_score_5352".into(), Vram::new(0x800B00EC)),
        UserSymbol::new("jtbl_800B00F8".into(), Vram::new(0x800B00F8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0220".into(), Vram::new(0x800B0220))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("tbl_5517".into(), Vram::new(0x800B0310)),
        UserSymbol::new("_n_5535".into(), Vram::new(0x800B0320)),
        UserSymbol::new("_mode_5538".into(), Vram::new(0x800B0338)),
        UserSymbol::new("mode_5557".into(), Vram::new(0x800B0350)),
        UserSymbol::new("_mode_5570".into(), Vram::new(0x800B035C)),
        UserSymbol::new("_tbl_5598".into(), Vram::new(0x800B0360)).with_size(Size::new(0x18)),
        UserSymbol::new("_tblLS_5599".into(), Vram::new(0x800B0378)).with_size(Size::new(0x24)),
        UserSymbol::new("_tblVM_5600".into(), Vram::new(0x800B039C)).with_size(Size::new(0x24)),
        UserSymbol::new("_tblVC_5601".into(), Vram::new(0x800B03C0)).with_size(Size::new(0x18)),
        UserSymbol::new("tbl_5648".into(), Vram::new(0x800B03D8)),
        UserSymbol::new("tbl_5664".into(), Vram::new(0x800B03EC)),
        UserSymbol::new("_team_5687".into(), Vram::new(0x800B03F8)),
        UserSymbol::new("_mode_5688".into(), Vram::new(0x800B0404)),
        UserSymbol::new("_game_5689".into(), Vram::new(0x800B0410)),
        UserSymbol::new("_mode_5701".into(), Vram::new(0x800B041C)),
        UserSymbol::new("_mode1_5709".into(), Vram::new(0x800B0424)),
        UserSymbol::new("_mode2_5710".into(), Vram::new(0x800B043C)),
        UserSymbol::new("tbl_5735".into(), Vram::new(0x800B0448)),
        UserSymbol::new("jtbl_800B0450".into(), Vram::new(0x800B0450))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0470".into(), Vram::new(0x800B0470))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0588".into(), Vram::new(0x800B0588))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B06B0".into(), Vram::new(0x800B06B0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B06F8".into(), Vram::new(0x800B06F8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B07E8".into(), Vram::new(0x800B07E8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B08D8".into(), Vram::new(0x800B08D8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B0938".into(), Vram::new(0x800B0938)).with_typ(SymbolType::Float64),
        UserSymbol::new("_pat_6137".into(), Vram::new(0x800B0940)).with_size(Size::new(0x18)),
        UserSymbol::new("DBL_800B0958".into(), Vram::new(0x800B0958)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B0960".into(), Vram::new(0x800B0960)).with_typ(SymbolType::Float64),
        UserSymbol::new("_pos_6413".into(), Vram::new(0x800B0968)).with_size(Size::new(0x90)),
        UserSymbol::new("_posChar_6445".into(), Vram::new(0x800B09F8)).with_size(Size::new(0x10)),
        UserSymbol::new("_posBgCursor_6446".into(), Vram::new(0x800B0A08))
            .with_size(Size::new(0x10)),
        UserSymbol::new("_cursor_6447".into(), Vram::new(0x800B0A18)).with_size(Size::new(0x30)),
        UserSymbol::new("DBL_800B0A48".into(), Vram::new(0x800B0A48)).with_typ(SymbolType::Float64),
        UserSymbol::new("RO_800B0A50".into(), Vram::new(0x800B0A50))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_map_6928".into(), Vram::new(0x800B0A5C)),
        UserSymbol::new("_cover_6929".into(), Vram::new(0x800B0A6C)),
        UserSymbol::new("_filter_6930".into(), Vram::new(0x800B0A7C)),
        UserSymbol::new("_wchar_6931".into(), Vram::new(0x800B0B0C)),
        UserSymbol::new("_cursor_7325".into(), Vram::new(0x800B0B14)).with_size(Size::new(0x30)),
        UserSymbol::new("RO_800B0B44".into(), Vram::new(0x800B0B44)),
        UserSymbol::new("_pos_7882".into(), Vram::new(0x800B0B50))
            .with_size(Size::new(0x20))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("jtbl_800B0B70".into(), Vram::new(0x800B0B70))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0BE8".into(), Vram::new(0x800B0BE8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_charTbl_8108".into(), Vram::new(0x800B0C5C)).with_size(Size::new(0x3C)),
        UserSymbol::new("jtbl_800B0C98".into(), Vram::new(0x800B0C98))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0D10".into(), Vram::new(0x800B0D10))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_star_8183".into(), Vram::new(0x800B0D84)),
        UserSymbol::new("_tex_8297".into(), Vram::new(0x800B0D94)).with_size(Size::new(0x8)),
        UserSymbol::new("_pos_8298".into(), Vram::new(0x800B0D9C))
            .with_size(Size::new(0x10))
            .with_typ(SymbolType::Word),
        UserSymbol::new("_moveTbl_8521".into(), Vram::new(0x800B0DAC)).with_typ(SymbolType::Word),
        UserSymbol::new("_onaji_8534".into(), Vram::new(0x800B0DB4)).with_size(Size::new(0x10)),
        UserSymbol::new("_panel2_8535".into(), Vram::new(0x800B0DC4)).with_size(Size::new(0x10)),
        UserSymbol::new("_panel4_8536".into(), Vram::new(0x800B0DD4)).with_size(Size::new(0x20)),
        UserSymbol::new("jtbl_800B0DF8".into(), Vram::new(0x800B0DF8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0E70".into(), Vram::new(0x800B0E70))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B0EE8".into(), Vram::new(0x800B0EE8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("RO_800B0F5C".into(), Vram::new(0x800B0F5C)).with_typ(SymbolType::Byte),
        UserSymbol::new("jtbl_800B1070".into(), Vram::new(0x800B1070))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_color_9658".into(), Vram::new(0x800B1168)),
        UserSymbol::new("DBL_800B1180".into(), Vram::new(0x800B1180)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B1188".into(), Vram::new(0x800B1188))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_posX_0_9712".into(), Vram::new(0x800B11A4)),
        UserSymbol::new("_posX_1_9713".into(), Vram::new(0x800B11CC)),
        UserSymbol::new("_posX_2_9714".into(), Vram::new(0x800B11E0)),
        UserSymbol::new("_posX_4_9715".into(), Vram::new(0x800B11F8)),
        UserSymbol::new("RO_800B1210".into(), Vram::new(0x800B1210)),
        UserSymbol::new("DBL_800B1218".into(), Vram::new(0x800B1218)).with_typ(SymbolType::Float64),
        UserSymbol::new("_rows_9879".into(), Vram::new(0x800B1220)),
        UserSymbol::new("_tex_9880".into(), Vram::new(0x800B1230)),
        UserSymbol::new("_choice_9881".into(), Vram::new(0x800B123C)),
        UserSymbol::new("jtbl_800B1258".into(), Vram::new(0x800B1258))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B1270".into(), Vram::new(0x800B1270))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B1288".into(), Vram::new(0x800B1288)).with_typ(SymbolType::Float64),
        UserSymbol::new("_pos_10346".into(), Vram::new(0x800B1290)).with_typ(SymbolType::Word),
        UserSymbol::new("_hedAllType_10392".into(), Vram::new(0x800B1298)),
        UserSymbol::new("_hedStoryId_10393".into(), Vram::new(0x800B12A8)),
        UserSymbol::new("_hedStoryX_10394".into(), Vram::new(0x800B12B8)),
        UserSymbol::new("_hedStoryY_10395".into(), Vram::new(0x800B12C8)),
        UserSymbol::new("_hedLevelId_10396".into(), Vram::new(0x800B12D8)),
        UserSymbol::new("_hedLevelX_10397".into(), Vram::new(0x800B12E4)),
        UserSymbol::new("_hedLevelY_10398".into(), Vram::new(0x800B12F0)),
        UserSymbol::new("_hed1PTaiQId_10399".into(), Vram::new(0x800B12FC)),
        UserSymbol::new("_hed1PTaiQX_10400".into(), Vram::new(0x800B1308)),
        UserSymbol::new("_hed1PTaiQY_10401".into(), Vram::new(0x800B1314)),
        UserSymbol::new("_hed1PTimeAtId_10402".into(), Vram::new(0x800B1320)),
        UserSymbol::new("_hed1PTimeAtX_10403".into(), Vram::new(0x800B1330)),
        UserSymbol::new("_hed1PTimeAtY_10404".into(), Vram::new(0x800B1340)),
        UserSymbol::new("_hedVsId_10405".into(), Vram::new(0x800B1350)),
        UserSymbol::new("_hedVsX_10406".into(), Vram::new(0x800B1360)),
        UserSymbol::new("_hedVsY_10407".into(), Vram::new(0x800B1370)),
        UserSymbol::new("jtbl_800B1380".into(), Vram::new(0x800B1380))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B13A8".into(), Vram::new(0x800B13A8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B13D0".into(), Vram::new(0x800B13D0))
            .with_typ(SymbolType::Jumptable)
            .with_migration_behavior(RodataMigrationBehavior::ForceMigrate()),
        UserSymbol::new("_lr_10544".into(), Vram::new(0x800B13F8)).with_size(Size::new(0x10)),
        UserSymbol::new("jtbl_800B1408".into(), Vram::new(0x800B1408))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("RO_800B1430".into(), Vram::new(0x800B1430)),
        UserSymbol::new("_dir_10660".into(), Vram::new(0x800B1438)).with_size(Size::new(0x8)),
        UserSymbol::new("jtbl_800B1440".into(), Vram::new(0x800B1440))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B14F8".into(), Vram::new(0x800B14F8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B1600".into(), Vram::new(0x800B1600))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B1708".into(), Vram::new(0x800B1708)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1710".into(), Vram::new(0x800B1710)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1718".into(), Vram::new(0x800B1718)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B1720".into(), Vram::new(0x800B1720))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B1828".into(), Vram::new(0x800B1828)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1830".into(), Vram::new(0x800B1830)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1838".into(), Vram::new(0x800B1838)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B1840".into(), Vram::new(0x800B1840))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("init_dl_135".into(), Vram::new(0x800B1950))
            .with_typ(SymbolType::UserCustom),
        UserSymbol::new("DBL_800B1978".into(), Vram::new(0x800B1978)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1980".into(), Vram::new(0x800B1980)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1988".into(), Vram::new(0x800B1988)).with_typ(SymbolType::Float64),
        UserSymbol::new("sMessageColorTable".into(), Vram::new(0x800B1990))
            .with_size(Size::new(0x18)),
        UserSymbol::new("jtbl_800B19A8".into(), Vram::new(0x800B19A8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B19F8".into(), Vram::new(0x800B19F8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B1A00".into(), Vram::new(0x800B1A00)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B1A08".into(), Vram::new(0x800B1A08))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("STR_800B1A54".into(), Vram::new(0x800B1A54))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("DBL_800B1A58".into(), Vram::new(0x800B1A58)).with_typ(SymbolType::Float64),
        UserSymbol::new("_size_122".into(), Vram::new(0x800B1A60)).with_size(Size::new(0x50)),
        UserSymbol::new("_addrTbl_124".into(), Vram::new(0x800B1AB0)).with_size(Size::new(0x50)),
        UserSymbol::new("_centerTbl_125".into(), Vram::new(0x800B1B00)).with_size(Size::new(0xA0)),
        UserSymbol::new("limit_table".into(), Vram::new(0x800B1BA0)),
        UserSymbol::new("RO_800B1BBC".into(), Vram::new(0x800B1BBC)),
        UserSymbol::new("_n_343".into(), Vram::new(0x800B1BC8)),
        UserSymbol::new("_l_359".into(), Vram::new(0x800B1BCC)),
        UserSymbol::new("_n_564".into(), Vram::new(0x800B1BD0)),
        UserSymbol::new("jtbl_800B1C10".into(), Vram::new(0x800B1C10))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B1C30".into(), Vram::new(0x800B1C30))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("rotate_table_399".into(), Vram::new(0x800B1C4C)),
        UserSymbol::new("rotate_mtx_400".into(), Vram::new(0x800B1C5C)),
        UserSymbol::new("_speed_561".into(), Vram::new(0x800B1C74)),
        UserSymbol::new("_retryMenu_itemCount".into(), Vram::new(0x800B1C78))
            .with_size(Size::new(0x6)),
        UserSymbol::new("_big_virus_def_wait".into(), Vram::new(0x800B1C80))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_big_virus_min_wait".into(), Vram::new(0x800B1C8C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_big_virus_max_wait".into(), Vram::new(0x800B1C98))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_scoreNumsColor".into(), Vram::new(0x800B1CA4))
            .with_size(Size::new(0x12))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesWriting_dmgamemain".into(), Vram::new(0x800B1CB8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("STR_800B1CE0".into(), Vram::new(0x800B1CE0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_posStP4StarX".into(), Vram::new(0x800B1D08)),
        UserSymbol::new("_posStStar".into(), Vram::new(0x800B1D18)).with_size(Size::new(0x10)),
        UserSymbol::new("RO_800B1D28".into(), Vram::new(0x800B1D28)).with_size(Size::new(0x8)),
        UserSymbol::new("_posP2StarX".into(), Vram::new(0x800B1D30)).with_size(Size::new(0x8)),
        UserSymbol::new("_posP2StarY".into(), Vram::new(0x800B1D38)).with_size(Size::new(0x24)),
        UserSymbol::new("_posP4Bottle".into(), Vram::new(0x800B1D5C)).with_size(Size::new(0x20)),
        UserSymbol::new("RO_800B1D7C".into(), Vram::new(0x800B1D7C)).with_size(Size::new(0x8)),
        UserSymbol::new("_posP4CharBase".into(), Vram::new(0x800B1D84)).with_size(Size::new(0x20)),
        UserSymbol::new("_posP4TeamStarX".into(), Vram::new(0x800B1DA4)).with_size(Size::new(0x48)),
        UserSymbol::new("_posP4CharStarX".into(), Vram::new(0x800B1DEC)).with_size(Size::new(0x90)),
        UserSymbol::new("_posP4StockCap".into(), Vram::new(0x800B1E7C))
            .with_size(Size::new(0x10))
            .with_typ(SymbolType::Word),
        UserSymbol::new("_posP2VirusNum".into(), Vram::new(0x800B1E8C)).with_size(Size::new(0x10)),
        UserSymbol::new("_posP2CharFrm".into(), Vram::new(0x800B1E9C))
            .with_size(Size::new(0x10))
            .with_typ(SymbolType::Word)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("RO_800B1EB0".into(), Vram::new(0x800B1EB0)),
        UserSymbol::new("black_color_1384".into(), Vram::new(0x800B1EB8)),
        UserSymbol::new("attack_table_1531".into(), Vram::new(0x800B1EC0)),
        UserSymbol::new("_bonus_1884".into(), Vram::new(0x800B1ECC)),
        UserSymbol::new("DBL_800B1EE8".into(), Vram::new(0x800B1EE8)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B1EF0".into(), Vram::new(0x800B1EF0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B1F68".into(), Vram::new(0x800B1F68)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B1F70".into(), Vram::new(0x800B1F70))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B1FF0".into(), Vram::new(0x800B1FF0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B2008".into(), Vram::new(0x800B2008))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_tbl_2997".into(), Vram::new(0x800B2020)).with_size(Size::new(0xC)),
        UserSymbol::new("_clr_3004".into(), Vram::new(0x800B202C)),
        UserSymbol::new("DBL_800B2030".into(), Vram::new(0x800B2030)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B2038".into(), Vram::new(0x800B2038)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B2040".into(), Vram::new(0x800B2040)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B2048".into(), Vram::new(0x800B2048)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B2050".into(), Vram::new(0x800B2050))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("cap_tex_4162".into(), Vram::new(0x800B2068)),
        UserSymbol::new("cap_pal_4164".into(), Vram::new(0x800B2070)).with_size(Size::new(0x30)),
        UserSymbol::new("DBL_800B20A0".into(), Vram::new(0x800B20A0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B20A8".into(), Vram::new(0x800B20A8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B20B0".into(), Vram::new(0x800B20B0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B20B8".into(), Vram::new(0x800B20B8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B20C0".into(), Vram::new(0x800B20C0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B20C8".into(), Vram::new(0x800B20C8))
            .with_size(Size::new(0x8))
            .with_typ(SymbolType::Float64)
            .with_migration_behavior(RodataMigrationBehavior::ForceMigrate()),
        UserSymbol::new("_tbl_4274".into(), Vram::new(0x800B20D0))
            .with_typ(SymbolType::Byte)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_tbl_4345".into(), Vram::new(0x800B20E0)),
        UserSymbol::new("_tex_4374".into(), Vram::new(0x800B20EC)),
        UserSymbol::new("_row_4375".into(), Vram::new(0x800B20F8)),
        UserSymbol::new("_pos_4415".into(), Vram::new(0x800B2104)),
        UserSymbol::new("_col_4416".into(), Vram::new(0x800B2108)),
        UserSymbol::new("_pos_4426".into(), Vram::new(0x800B210C)),
        UserSymbol::new("_col_4427".into(), Vram::new(0x800B2114)),
        UserSymbol::new("_tex_4459".into(), Vram::new(0x800B211C)).with_size(Size::new(0x18)),
        UserSymbol::new("RO_800B2134".into(), Vram::new(0x800B2134)).with_size(Size::new(0x10)),
        UserSymbol::new("DBL_800B2148".into(), Vram::new(0x800B2148)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B2150".into(), Vram::new(0x800B2150)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B2158".into(), Vram::new(0x800B2158)).with_typ(SymbolType::Float64),
        UserSymbol::new("_x_4663".into(), Vram::new(0x800B2160)),
        UserSymbol::new("_x_4670".into(), Vram::new(0x800B2168)),
        UserSymbol::new("_x_4676".into(), Vram::new(0x800B2170)),
        UserSymbol::new("_x_4693".into(), Vram::new(0x800B2178)),
        UserSymbol::new("_rect_4752".into(), Vram::new(0x800B2180)).with_size(Size::new(0x50)),
        UserSymbol::new("_pat_4838".into(), Vram::new(0x800B21D0)),
        UserSymbol::new("DBL_800B21E8".into(), Vram::new(0x800B21E8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B21F0".into(), Vram::new(0x800B21F0)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B21F8".into(), Vram::new(0x800B21F8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B2250".into(), Vram::new(0x800B2250))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B2278".into(), Vram::new(0x800B2278))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("map_x_table_5383".into(), Vram::new(0x800B2294)),
        UserSymbol::new("map_y_table_5384".into(), Vram::new(0x800B22AC)),
        UserSymbol::new("size_table_5385".into(), Vram::new(0x800B22B0)),
        UserSymbol::new("jtbl_800B22B8".into(), Vram::new(0x800B22B8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B22D8".into(), Vram::new(0x800B22D8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B22F8".into(), Vram::new(0x800B22F8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_bgPos_5792".into(), Vram::new(0x800B2314))
            .with_size(Size::new(0x1C))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_panelPos_5793".into(), Vram::new(0x800B2330))
            .with_size(Size::new(0xC))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_bgTex_5794".into(), Vram::new(0x800B233C)),
        UserSymbol::new("_magTex_5795".into(), Vram::new(0x800B2340)),
        UserSymbol::new("_scrTex_5796".into(), Vram::new(0x800B2344)),
        UserSymbol::new("_panelTex_5797".into(), Vram::new(0x800B2348)),
        UserSymbol::new("tbl_5867".into(), Vram::new(0x800B2354)).with_size(Size::new(0x10)),
        UserSymbol::new("jtbl_800B2368".into(), Vram::new(0x800B2368))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B2388".into(), Vram::new(0x800B2388))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B23A8".into(), Vram::new(0x800B23A8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_x_6416".into(), Vram::new(0x800B23C4)),
        UserSymbol::new("dir_6435".into(), Vram::new(0x800B23CC))
            .with_typ(SymbolType::Word)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("color2index_6470".into(), Vram::new(0x800B23D4)),
        UserSymbol::new("jtbl_800B23E0".into(), Vram::new(0x800B23E0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B2400".into(), Vram::new(0x800B2400)).with_typ(SymbolType::Float64),
        UserSymbol::new("RO_800B2408".into(), Vram::new(0x800B2408)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2410".into(), Vram::new(0x800B2410)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2418".into(), Vram::new(0x800B2418)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2420".into(), Vram::new(0x800B2420)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2428".into(), Vram::new(0x800B2428)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2430".into(), Vram::new(0x800B2430)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2438".into(), Vram::new(0x800B2438)).with_size(Size::new(0x8)),
        UserSymbol::new("RO_800B2440".into(), Vram::new(0x800B2440)).with_size(Size::new(0x8)),
        UserSymbol::new("virus_1_1".into(), Vram::new(0x800B2448)).with_size(Size::new(0x24)),
        UserSymbol::new("RO_800B246C".into(), Vram::new(0x800B246C)).with_size(Size::new(0x5)),
        UserSymbol::new("RO_800B2474".into(), Vram::new(0x800B2474)).with_size(Size::new(0x5)),
        UserSymbol::new("position_1_1".into(), Vram::new(0x800B247C)).with_size(Size::new(0x10)),
        UserSymbol::new("virus_2_1".into(), Vram::new(0x800B248C)).with_size(Size::new(0x3C)),
        UserSymbol::new("capsel_2_1".into(), Vram::new(0x800B24C8)).with_size(Size::new(0xC)),
        UserSymbol::new("position_2_1".into(), Vram::new(0x800B24D4)).with_size(Size::new(0x28)),
        UserSymbol::new("virus_3_1".into(), Vram::new(0x800B24FC)).with_size(Size::new(0x3C)),
        UserSymbol::new("capsel_3_1".into(), Vram::new(0x800B2538)).with_size(Size::new(0xB)),
        UserSymbol::new("position_3_1".into(), Vram::new(0x800B2544)).with_size(Size::new(0x28)),
        UserSymbol::new("virus_4_1".into(), Vram::new(0x800B256C)).with_size(Size::new(0x30)),
        UserSymbol::new("capsel_4_1".into(), Vram::new(0x800B259C)).with_size(Size::new(0x11)),
        UserSymbol::new("position_4_1".into(), Vram::new(0x800B25B0)).with_size(Size::new(0x40)),
        UserSymbol::new("mes_1_1".into(), Vram::new(0x800B25F0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_2".into(), Vram::new(0x800B2618))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_3".into(), Vram::new(0x800B2664))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_4".into(), Vram::new(0x800B26B4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_5".into(), Vram::new(0x800B26EC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_6".into(), Vram::new(0x800B2738))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_7".into(), Vram::new(0x800B2780))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_1_8".into(), Vram::new(0x800B27D4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_1".into(), Vram::new(0x800B2808))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_2".into(), Vram::new(0x800B284C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_3".into(), Vram::new(0x800B2894))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_4".into(), Vram::new(0x800B28DC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_5".into(), Vram::new(0x800B2914))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_6".into(), Vram::new(0x800B2958))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_7".into(), Vram::new(0x800B2994))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_8".into(), Vram::new(0x800B29E0))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_2_9".into(), Vram::new(0x800B2A20))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_1".into(), Vram::new(0x800B2A54))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_2".into(), Vram::new(0x800B2A98))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_3".into(), Vram::new(0x800B2ADC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_4".into(), Vram::new(0x800B2B24))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_5".into(), Vram::new(0x800B2B58))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_6".into(), Vram::new(0x800B2BA8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_7".into(), Vram::new(0x800B2BF4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_8".into(), Vram::new(0x800B2C44))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_9".into(), Vram::new(0x800B2C90))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_10".into(), Vram::new(0x800B2CDC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_11".into(), Vram::new(0x800B2D28))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_12".into(), Vram::new(0x800B2D60))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_13".into(), Vram::new(0x800B2D90))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_14".into(), Vram::new(0x800B2DD8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_3_15".into(), Vram::new(0x800B2E24))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_1".into(), Vram::new(0x800B2E58))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_2".into(), Vram::new(0x800B2E90))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_3".into(), Vram::new(0x800B2ED4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_4".into(), Vram::new(0x800B2F20))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_5".into(), Vram::new(0x800B2F60))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_6".into(), Vram::new(0x800B2FAC))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_7".into(), Vram::new(0x800B2FE4))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_8".into(), Vram::new(0x800B302C))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_9".into(), Vram::new(0x800B3054))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("mes_4_10".into(), Vram::new(0x800B3078))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("jtbl_800B3098".into(), Vram::new(0x800B3098))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B30F0".into(), Vram::new(0x800B30F0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B3118".into(), Vram::new(0x800B3118))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("_tex_884".into(), Vram::new(0x800B3130))
            .with_size(Size::new(0x10))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_pos_885".into(), Vram::new(0x800B3140))
            .with_size(Size::new(0x10))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("RO_800B3150".into(), Vram::new(0x800B3150))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("DBL_800B3168".into(), Vram::new(0x800B3168)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3170".into(), Vram::new(0x800B3170)).with_typ(SymbolType::Float64),
        UserSymbol::new("_posContPanel".into(), Vram::new(0x800B3178))
            .with_size(Size::new(0x8))
            .with_typ(SymbolType::Word)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_posCircle_924".into(), Vram::new(0x800B3180))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_posFinger_925".into(), Vram::new(0x800B31A0))
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("map_x_table_1036".into(), Vram::new(0x800B31C0))
            .with_size(Size::new(0x18)),
        UserSymbol::new("_seqTbl_1037".into(), Vram::new(0x800B31D8)).with_size(Size::new(0x2)),
        UserSymbol::new("map_y_table_1038".into(), Vram::new(0x800B31DC)).with_size(Size::new(0x2)),
        UserSymbol::new("size_table_1039".into(), Vram::new(0x800B31E0)).with_size(Size::new(0x2)),
        UserSymbol::new("jtbl_800B31E8".into(), Vram::new(0x800B31E8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B3200".into(), Vram::new(0x800B3200)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3208".into(), Vram::new(0x800B3208)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B3210".into(), Vram::new(0x800B3210))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B3228".into(), Vram::new(0x800B3228)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3230".into(), Vram::new(0x800B3230)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3238".into(), Vram::new(0x800B3238)).with_typ(SymbolType::Float64),
        UserSymbol::new("_stageTbl".into(), Vram::new(0x800B3240)).with_size(Size::new(0x12)),
        UserSymbol::new("DBL_800B3258".into(), Vram::new(0x800B3258)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3260".into(), Vram::new(0x800B3260)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3268".into(), Vram::new(0x800B3268)).with_typ(SymbolType::Float64),
        UserSymbol::new("RO_800B3270".into(), Vram::new(0x800B3270)),
        UserSymbol::new("_mesBootContErr".into(), Vram::new(0x800B32A8))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("_mesBootCSumErr".into(), Vram::new(0x800B3318))
            .with_typ(SymbolType::CString)
            .with_migration_behavior(RodataMigrationBehavior::ForceNotMigrate()),
        UserSymbol::new("DBL_800B3360".into(), Vram::new(0x800B3360)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3368".into(), Vram::new(0x800B3368)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3370".into(), Vram::new(0x800B3370)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3378".into(), Vram::new(0x800B3378)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3380".into(), Vram::new(0x800B3380)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3388".into(), Vram::new(0x800B3388)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3390".into(), Vram::new(0x800B3390)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3398".into(), Vram::new(0x800B3398)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33A0".into(), Vram::new(0x800B33A0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33A8".into(), Vram::new(0x800B33A8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33B0".into(), Vram::new(0x800B33B0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33B8".into(), Vram::new(0x800B33B8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33C0".into(), Vram::new(0x800B33C0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33C8".into(), Vram::new(0x800B33C8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B33D0".into(), Vram::new(0x800B33D0)).with_typ(SymbolType::Float64),
        UserSymbol::new("jtbl_800B33D8".into(), Vram::new(0x800B33D8))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("jtbl_800B33F0".into(), Vram::new(0x800B33F0))
            .with_typ(SymbolType::Jumptable),
        UserSymbol::new("DBL_800B3450".into(), Vram::new(0x800B3450)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3458".into(), Vram::new(0x800B3458)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3460".into(), Vram::new(0x800B3460)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3468".into(), Vram::new(0x800B3468)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3470".into(), Vram::new(0x800B3470)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3478".into(), Vram::new(0x800B3478)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3480".into(), Vram::new(0x800B3480)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3488".into(), Vram::new(0x800B3488)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3490".into(), Vram::new(0x800B3490)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3498".into(), Vram::new(0x800B3498)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34A0".into(), Vram::new(0x800B34A0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34A8".into(), Vram::new(0x800B34A8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34B0".into(), Vram::new(0x800B34B0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34B8".into(), Vram::new(0x800B34B8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34C0".into(), Vram::new(0x800B34C0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34C8".into(), Vram::new(0x800B34C8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34D0".into(), Vram::new(0x800B34D0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34D8".into(), Vram::new(0x800B34D8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34E0".into(), Vram::new(0x800B34E0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34E8".into(), Vram::new(0x800B34E8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34F0".into(), Vram::new(0x800B34F0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B34F8".into(), Vram::new(0x800B34F8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3500".into(), Vram::new(0x800B3500)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3508".into(), Vram::new(0x800B3508)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3510".into(), Vram::new(0x800B3510)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3518".into(), Vram::new(0x800B3518)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3520".into(), Vram::new(0x800B3520)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3528".into(), Vram::new(0x800B3528)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3530".into(), Vram::new(0x800B3530)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3538".into(), Vram::new(0x800B3538)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3540".into(), Vram::new(0x800B3540)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3548".into(), Vram::new(0x800B3548)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3550".into(), Vram::new(0x800B3550)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3558".into(), Vram::new(0x800B3558)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3560".into(), Vram::new(0x800B3560)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3568".into(), Vram::new(0x800B3568)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3570".into(), Vram::new(0x800B3570)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3578".into(), Vram::new(0x800B3578)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3580".into(), Vram::new(0x800B3580)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3588".into(), Vram::new(0x800B3588)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3590".into(), Vram::new(0x800B3590)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3598".into(), Vram::new(0x800B3598)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35A0".into(), Vram::new(0x800B35A0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35A8".into(), Vram::new(0x800B35A8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35B0".into(), Vram::new(0x800B35B0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35B8".into(), Vram::new(0x800B35B8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35C0".into(), Vram::new(0x800B35C0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35C8".into(), Vram::new(0x800B35C8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35D0".into(), Vram::new(0x800B35D0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35D8".into(), Vram::new(0x800B35D8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35E0".into(), Vram::new(0x800B35E0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35E8".into(), Vram::new(0x800B35E8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35F0".into(), Vram::new(0x800B35F0)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B35F8".into(), Vram::new(0x800B35F8)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3600".into(), Vram::new(0x800B3600)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3608".into(), Vram::new(0x800B3608)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3610".into(), Vram::new(0x800B3610)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3618".into(), Vram::new(0x800B3618)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3620".into(), Vram::new(0x800B3620)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3630".into(), Vram::new(0x800B3630)).with_typ(SymbolType::Float64),
        UserSymbol::new("DBL_800B3638".into(), Vram::new(0x800B3638)).with_typ(SymbolType::Float64),
        UserSymbol::new("audio_memory".into(), Vram::new(0x800B3640)).with_size(Size::new(0x31000)),
        UserSymbol::new("mt".into(), Vram::new(0x800E4640)).with_size(Size::new(0x9C0)),
        UserSymbol::new("pGameState".into(), Vram::new(0x800E5000)),
        UserSymbol::new("delpos_tbl".into(), Vram::new(0x800E5008)),
        UserSymbol::new("delpos_cnt".into(), Vram::new(0x800E5198)),
        UserSymbol::new("OnVirusP_org".into(), Vram::new(0x800E519C)),
        UserSymbol::new("last_flash".into(), Vram::new(0x800E51A0)),
        UserSymbol::new("_cache_1333".into(), Vram::new(0x800E51B0)),
        UserSymbol::new("B_800E53B0".into(), Vram::new(0x800E53B0)),
        UserSymbol::new("binCount".into(), Vram::new(0x800E53C0)),
        UserSymbol::new("cont_bg_flg".into(), Vram::new(0x800E53C4)),
        UserSymbol::new("etcTexAddress".into(), Vram::new(0x800E53C8)),
        UserSymbol::new("etcLwsAddress".into(), Vram::new(0x800E53CC)),
        UserSymbol::new("etcLwsTbl".into(), Vram::new(0x800E53D0)).with_size(Size::new(0x40)),
        UserSymbol::new("logo_ofsY".into(), Vram::new(0x800E5410)),
        UserSymbol::new("etc_mode".into(), Vram::new(0x800E5414)),
        UserSymbol::new("g_etc_work".into(), Vram::new(0x800E5418)).with_size(Size::new(0x3C0)),
        UserSymbol::new("D_800E57D8".into(), Vram::new(0x800E57D8)),
        UserSymbol::new("etc_viewMtx".into(), Vram::new(0x800E5818)).with_size(Size::new(0x40)),
        UserSymbol::new("fbuf".into(), Vram::new(0x800E5860)).with_size(Size::new(0x28)),
        UserSymbol::new("sPrinterState".into(), Vram::new(0x800E5890)).with_size(Size::new(0x12)),
        UserSymbol::new("B_800E58B0".into(), Vram::new(0x800E58B0)),
        UserSymbol::new("B_800E58B4".into(), Vram::new(0x800E58B4)),
        UserSymbol::new("B_800E58B8".into(), Vram::new(0x800E58B8)),
        UserSymbol::new("B_800E58BC".into(), Vram::new(0x800E58BC)).with_size(Size::new(0x4)),
        UserSymbol::new("B_800E58C0".into(), Vram::new(0x800E58C0)).with_size(Size::new(0x50)),
        UserSymbol::new("sDebugMenu_CursorCounter".into(), Vram::new(0x800E5910)),
        UserSymbol::new("B_800E5914".into(), Vram::new(0x800E5914)),
        UserSymbol::new("B_800E5918".into(), Vram::new(0x800E5918)),
        UserSymbol::new("B_800E591C".into(), Vram::new(0x800E591C)),
        UserSymbol::new("B_800E5920".into(), Vram::new(0x800E5920)),
        UserSymbol::new("B_800E5924".into(), Vram::new(0x800E5924)),
        UserSymbol::new("B_800E5928".into(), Vram::new(0x800E5928)),
        UserSymbol::new("B_800E592C".into(), Vram::new(0x800E592C)),
        UserSymbol::new(
            "sDebugMenu_CurrentSettingsPage".into(),
            Vram::new(0x800E5930),
        ),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_SelectedCharacter".into(),
            Vram::new(0x800E5934),
        ),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_CharacterStats".into(),
            Vram::new(0x800E5938),
        ),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_DefaultStats".into(),
            Vram::new(0x800E5968),
        )
        .with_size(Size::new(0x2)),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_SelectedColumn".into(),
            Vram::new(0x800E596C),
        ),
        UserSymbol::new(
            "sDebugMenu_CharacterEdit_SelectedRow".into(),
            Vram::new(0x800E5970),
        )
        .with_size(Size::new(0x10)),
        UserSymbol::new("heapTop".into(), Vram::new(0x800E5980)).with_size(Size::new(0x4)),
        UserSymbol::new("_texAll".into(), Vram::new(0x800E5990)),
        UserSymbol::new("_texKaSa".into(), Vram::new(0x800E5994)),
        UserSymbol::new("title_mode_type".into(), Vram::new(0x800E59A0)),
        UserSymbol::new("title_fade_count".into(), Vram::new(0x800E59A4)),
        UserSymbol::new("title_fade_step".into(), Vram::new(0x800E59A8)),
        UserSymbol::new("fin_frame_440".into(), Vram::new(0x800E59B0)),
        UserSymbol::new("fin_demo_441".into(), Vram::new(0x800E59B4)),
        UserSymbol::new("fin_frame_543".into(), Vram::new(0x800E59B8)),
        UserSymbol::new("fin_frame_568".into(), Vram::new(0x800E59BC)),
        UserSymbol::new("fin_demo_569".into(), Vram::new(0x800E59C0)),
        UserSymbol::new("fin_frame_623".into(), Vram::new(0x800E59C4)),
        UserSymbol::new("fin_demo_624".into(), Vram::new(0x800E59C8)),
        UserSymbol::new("fin_frame_667".into(), Vram::new(0x800E59CC)),
        UserSymbol::new("fin_frame_737".into(), Vram::new(0x800E59D0)),
        UserSymbol::new("fin_frame_768".into(), Vram::new(0x800E59D4)),
        UserSymbol::new("title_time".into(), Vram::new(0x800E59D8)),
        UserSymbol::new("title_wait".into(), Vram::new(0x800E59DC)),
        UserSymbol::new("title_data".into(), Vram::new(0x800E59E0)),
        UserSymbol::new("title_bmp_data".into(), Vram::new(0x800E59E4)),
        UserSymbol::new("lws_data".into(), Vram::new(0x800E59E8)),
        UserSymbol::new("lws_scene".into(), Vram::new(0x800E59EC)),
        UserSymbol::new("mess_st".into(), Vram::new(0x800E59F0)).with_size(Size::new(0x80)),
        UserSymbol::new("mess_roll_st".into(), Vram::new(0x800E5A70)).with_size(Size::new(0x80)),
        UserSymbol::new("mess_heap_area".into(), Vram::new(0x800E5AF0)),
        UserSymbol::new("mess_roll_heap".into(), Vram::new(0x800E5EF0)),
        UserSymbol::new("st_message_count".into(), Vram::new(0x800E5EF4)),
        UserSymbol::new("st_mes_ptr".into(), Vram::new(0x800E5EF8)),
        UserSymbol::new("loop_flg".into(), Vram::new(0x800E5EFC)),
        UserSymbol::new("story_staff_roll".into(), Vram::new(0x800E5F00)),
        UserSymbol::new("story_norm".into(), Vram::new(0x800E5F04)),
        UserSymbol::new("story_viewMtx".into(), Vram::new(0x800E5F08)).with_size(Size::new(0x40)),
        UserSymbol::new("story_objectMtx".into(), Vram::new(0x800E5F50))
            .with_size(Size::new(0x2800)),
        UserSymbol::new("wakuGraphic".into(), Vram::new(0x800E8750)).with_size(Size::new(0x4)),
        UserSymbol::new("bgGraphic".into(), Vram::new(0x800E87A8)),
        UserSymbol::new("storyGraphic".into(), Vram::new(0x800E87AC)),
        UserSymbol::new("messageData".into(), Vram::new(0x800E87B0)),
        UserSymbol::new("first_copy".into(), Vram::new(0x800E87B4)),
        UserSymbol::new("AnimProc".into(), Vram::new(0x800E87C0)),
        UserSymbol::new("randomindex".into(), Vram::new(0x800E97C0)),
        UserSymbol::new("randomtable".into(), Vram::new(0x800E97C8)).with_size(Size::new(0xDC)),
        UserSymbol::new("rec_buff".into(), Vram::new(0x800E98B0)).with_size(Size::new(0x4)),
        UserSymbol::new("pRecBuff".into(), Vram::new(0x800E98B8)).with_size(Size::new(0x10)),
        UserSymbol::new("oldCont".into(), Vram::new(0x800E98C8)).with_size(Size::new(0x8)),
        UserSymbol::new("RecPos".into(), Vram::new(0x800E98D0)).with_size(Size::new(0x10)),
        UserSymbol::new("PlayPos".into(), Vram::new(0x800E98E0)).with_size(Size::new(0x10)),
        UserSymbol::new("WaitTime".into(), Vram::new(0x800E98F0)).with_size(Size::new(0x10)),
        UserSymbol::new("max_rec".into(), Vram::new(0x800E9900)).with_size(Size::new(0x4)),
        UserSymbol::new("replay_player".into(), Vram::new(0x800E9904)).with_size(Size::new(0x4)),
        UserSymbol::new("plr_player".into(), Vram::new(0x800E9910)).with_size(Size::new(0x14)),
        UserSymbol::new("max_channels".into(), Vram::new(0x800E9924)),
        UserSymbol::new("mus_voices".into(), Vram::new(0x800E9928)),
        UserSymbol::new("mus_channels".into(), Vram::new(0x800E992C)),
        UserSymbol::new("mus_channels2".into(), Vram::new(0x800E9930)),
        UserSymbol::new("mus_vsyncs_per_second".into(), Vram::new(0x800E9934)),
        UserSymbol::new("mus_next_frame_time".into(), Vram::new(0x800E9938)),
        UserSymbol::new("mus_master_volume_effects".into(), Vram::new(0x800E993C)),
        UserSymbol::new("mus_master_volume_songs".into(), Vram::new(0x800E993E)),
        UserSymbol::new("mus_current_handle".into(), Vram::new(0x800E9940)),
        UserSymbol::new("mus_random_seed".into(), Vram::new(0x800E9944)),
        UserSymbol::new("mus_init_bank".into(), Vram::new(0x800E9948)),
        UserSymbol::new("mus_default_bank".into(), Vram::new(0x800E994C)),
        UserSymbol::new("mus_songfxchange_flag".into(), Vram::new(0x800E9950)),
        UserSymbol::new("mus_last_fxtype".into(), Vram::new(0x800E9954)),
        UserSymbol::new("libmus_fxheader_current".into(), Vram::new(0x800E9958)),
        UserSymbol::new("libmus_fxheader_single".into(), Vram::new(0x800E995C)),
        UserSymbol::new("marker_callback".into(), Vram::new(0x800E9960)),
        UserSymbol::new("fifo_start".into(), Vram::new(0x800E9964)),
        UserSymbol::new("fifo_current".into(), Vram::new(0x800E9968)),
        UserSymbol::new("fifo_limit".into(), Vram::new(0x800E996C)),
        UserSymbol::new("fifo_addr".into(), Vram::new(0x800E9970)),
        UserSymbol::new("dma_buffer_head".into(), Vram::new(0x800E9980)),
        UserSymbol::new("dma_buffer_free".into(), Vram::new(0x800E9984)),
        UserSymbol::new("dma_buffer_list".into(), Vram::new(0x800E9988)),
        UserSymbol::new("audio_IO_mess_buf".into(), Vram::new(0x800E998C)),
        UserSymbol::new("audio_mess_buf".into(), Vram::new(0x800E9990)),
        UserSymbol::new("audio_dma_size".into(), Vram::new(0x800E9994)),
        UserSymbol::new("audio_dma_count".into(), Vram::new(0x800E9998)).with_size(Size::new(0x4)),
        UserSymbol::new("audDMAMessageQ".into(), Vram::new(0x800E99A0)),
        UserSymbol::new("cartrom_handle".into(), Vram::new(0x800E99B8)).with_size(Size::new(0x4)),
        UserSymbol::new("audio_sched".into(), Vram::new(0x800E99C0)).with_size(Size::new(0x4)),
        UserSymbol::new("sched_mem".into(), Vram::new(0x800E99C4)).with_size(Size::new(0x4)),
        UserSymbol::new(
            "__MusIntAudManInit$in_function_static$thread".into(),
            Vram::new(0x800E99D0),
        ),
        UserSymbol::new(
            "__MusIntAudManInit$in_function_static$stack_addr".into(),
            Vram::new(0x800E9B80),
        ),
        UserSymbol::new("audio_tasks".into(), Vram::new(0x800E9B84)),
        UserSymbol::new("audio_command_list".into(), Vram::new(0x800E9B88)),
        UserSymbol::new("audio_heap".into(), Vram::new(0x800E9B90)).with_size(Size::new(0x10)),
        UserSymbol::new("frame_samples".into(), Vram::new(0x800E9BA0)),
        UserSymbol::new("frame_samples_min".into(), Vram::new(0x800E9BA4)),
        UserSymbol::new("frame_samples_max".into(), Vram::new(0x800E9BA8)),
        UserSymbol::new("extra_samples".into(), Vram::new(0x800E9BAC)),
        UserSymbol::new("aiRootP".into(), Vram::new(0x800E9BB0)),
        UserSymbol::new("aiWall".into(), Vram::new(0x800E9BB4)).with_size(Size::new(0x1)),
        UserSymbol::new("gfx_msg_no".into(), Vram::new(0x800E9BB6)).with_size(Size::new(0x2)),
        UserSymbol::new("ai_char_data".into(), Vram::new(0x800E9BC0)).with_size(Size::new(0x6C0)),
        UserSymbol::new("success".into(), Vram::new(0x800EA280)).with_size(Size::new(0x1)),
        UserSymbol::new("attack_sprite_idx".into(), Vram::new(0x800EA284))
            .with_size(Size::new(0x4)),
        UserSymbol::new("sBgTasksManager_Manager".into(), Vram::new(0x800EA290))
            .with_size(Size::new(0x1248)),
        UserSymbol::new("B_800EB4D8".into(), Vram::new(0x800EB4D8)),
        UserSymbol::new("main_old".into(), Vram::new(0x800EB4F0)),
        UserSymbol::new("_menuMain_lastDepth".into(), Vram::new(0x800EB4F4))
            .with_size(Size::new(0x4)),
        UserSymbol::new("aiRecurData".into(), Vram::new(0x800EB4F8)).with_size(Size::new(0x168)),
        UserSymbol::new("aiTEdgeCnt".into(), Vram::new(0x800EB660)).with_size(Size::new(0x1)),
        UserSymbol::new("B_800EB670".into(), Vram::new(0x800EB670)).with_size(Size::new(0x680)),
        UserSymbol::new("main_no".into(), Vram::new(0x800EBCF0)),
        UserSymbol::new("gGfxHead".into(), Vram::new(0x800EBCF4)).with_size(Size::new(0x4)),
        UserSymbol::new("gControllerHoldButtons".into(), Vram::new(0x800EBCF8))
            .with_size(Size::new(0xA)),
        UserSymbol::new("hei_data".into(), Vram::new(0x800EBD02)).with_size(Size::new(0xA)),
        UserSymbol::new("PlayTime".into(), Vram::new(0x800EBD10)),
        UserSymbol::new("aiNext".into(), Vram::new(0x800EBD14)).with_size(Size::new(0x2)),
        UserSymbol::new("main_joy".into(), Vram::new(0x800EBD16)).with_size(Size::new(0x4)),
        UserSymbol::new("gfxThread".into(), Vram::new(0x800EBD20)).with_size(Size::new(0x1B0)),
        UserSymbol::new("gfx_msgbuf".into(), Vram::new(0x800EBED0)),
        UserSymbol::new("game_map_data".into(), Vram::new(0x800EBEF0)).with_size(Size::new(0x1540)),
        UserSymbol::new("gfx_msg".into(), Vram::new(0x800ED430)).with_size(Size::new(0x2)),
        UserSymbol::new("nnScGraphicsStack".into(), Vram::new(0x800ED440))
            .with_size(Size::new(0x2000)),
        UserSymbol::new("gameBackup".into(), Vram::new(0x800EF440)).with_size(Size::new(0x8)),
        UserSymbol::new("CapsMagazine".into(), Vram::new(0x800EF448)).with_size(Size::new(0x100)),
        UserSymbol::new("attack_effect_idx".into(), Vram::new(0x800EF548)),
        UserSymbol::new("diskrom_handle".into(), Vram::new(0x800EF54C)),
        UserSymbol::new("fool_mode".into(), Vram::new(0x800EF550)),
        UserSymbol::new("joycur1".into(), Vram::new(0x800EF554)),
        UserSymbol::new("decide".into(), Vram::new(0x800EF556)).with_size(Size::new(0x1)),
        UserSymbol::new("evs_mem_data".into(), Vram::new(0x800EF560)).with_size(Size::new(0x750)),
        UserSymbol::new("_menuMain_lastSelect".into(), Vram::new(0x800EFCB0))
            .with_size(Size::new(0x20)),
        UserSymbol::new("evs_gamesel".into(), Vram::new(0x800EFCD0)).with_size(Size::new(0x4)),
        UserSymbol::new("nnScStack".into(), Vram::new(0x800EFCE0)),
        UserSymbol::new("evs_playmax".into(), Vram::new(0x800F1CE0)).with_size(Size::new(0x1)),
        UserSymbol::new("aiFieldData".into(), Vram::new(0x800F1CE8)).with_size(Size::new(0x110)),
        UserSymbol::new("pObjectMtx".into(), Vram::new(0x800F1DF8)).with_size(Size::new(0x4)),
        UserSymbol::new("evs_default_name".into(), Vram::new(0x800F1E00)),
        UserSymbol::new("joycur2".into(), Vram::new(0x800F1E20)).with_size(Size::new(0x2)),
        UserSymbol::new("sGraphicStack".into(), Vram::new(0x800F1E30)).with_size(Size::new(0x2000)),
        UserSymbol::new("aiHiErB".into(), Vram::new(0x800F3E30)).with_size(Size::new(0x2)),
        UserSymbol::new("aiPriOfs".into(), Vram::new(0x800F3E32)),
        UserSymbol::new("aiRollFinal".into(), Vram::new(0x800F3E34)).with_size(Size::new(0x1)),
        UserSymbol::new("B_800F3E38".into(), Vram::new(0x800F3E38)),
        UserSymbol::new("watchGame".into(), Vram::new(0x800F3E50)),
        UserSymbol::new("aiFlagCnt".into(), Vram::new(0x800F3E54)),
        UserSymbol::new("aiGoalX".into(), Vram::new(0x800F3E56)).with_size(Size::new(0x2)),
        UserSymbol::new("watchMenu".into(), Vram::new(0x800F3E5C)),
        UserSymbol::new("B_800F3E60".into(), Vram::new(0x800F3E60)),
        UserSymbol::new("gControllerPrevHoldButtons".into(), Vram::new(0x800F3E64))
            .with_size(Size::new(0xA)),
        UserSymbol::new("wid_data".into(), Vram::new(0x800F3E6E)).with_size(Size::new(0xA)),
        UserSymbol::new("link_joy".into(), Vram::new(0x800F3E78)),
        UserSymbol::new("aiHiErY".into(), Vram::new(0x800F3E7C)).with_size(Size::new(0x2)),
        UserSymbol::new("D_800F3E80".into(), Vram::new(0x800F3E80)),
        UserSymbol::new("attack_sprite".into(), Vram::new(0x800F3E90)).with_size(Size::new(0xA00)),
        UserSymbol::new("watchManual".into(), Vram::new(0x800F4890)),
        UserSymbol::new("aiRootCnt".into(), Vram::new(0x800F4894)).with_size(Size::new(0x1)),
        UserSymbol::new("gfx_msgQ".into(), Vram::new(0x800F4898)),
        UserSymbol::new("gfx_client".into(), Vram::new(0x800F48B0)).with_size(Size::new(0x8)),
        UserSymbol::new("joygam".into(), Vram::new(0x800F48B8)),
        UserSymbol::new("gameGeom".into(), Vram::new(0x800F48C0)),
        UserSymbol::new("joycur".into(), Vram::new(0x800F48C4)).with_size(Size::new(0xA)),
        UserSymbol::new("ai_param".into(), Vram::new(0x800F48D0)).with_size(Size::new(0xA80)),
        UserSymbol::new("MissRate".into(), Vram::new(0x800F5350)).with_size(Size::new(0x4)),
        UserSymbol::new("B_800F5358".into(), Vram::new(0x800F5358)).with_size(Size::new(0x10)),
        UserSymbol::new("aiFlag".into(), Vram::new(0x800F5370)).with_size(Size::new(0x1900)),
        UserSymbol::new("aiRoot".into(), Vram::new(0x800F6C70)).with_size(Size::new(0x64)),
        UserSymbol::new("__muscontrol_flag".into(), Vram::new(0x800F6CD4)),
        UserSymbol::new("joyflg".into(), Vram::new(0x800F6CD8)).with_size(Size::new(0x8)),
        UserSymbol::new("aiYEdgeCnt".into(), Vram::new(0x800F6CE0)).with_size(Size::new(0x1)),
        UserSymbol::new("virus_map_data".into(), Vram::new(0x800F6CF0)).with_size(Size::new(0x600)),
        UserSymbol::new("virus_map_disp_order".into(), Vram::new(0x800F72F0))
            .with_size(Size::new(0x180)),
        UserSymbol::new("evs_cfg_4p".into(), Vram::new(0x800F7470)).with_size(Size::new(0x16)),
        UserSymbol::new("s_hard_mode".into(), Vram::new(0x800F7488)),
        UserSymbol::new("attack_sprite_address".into(), Vram::new(0x800F748C)),
        UserSymbol::new("gfxYieldBuf".into(), Vram::new(0x800F7490)).with_size(Size::new(0xC00)),
        UserSymbol::new("__libmus_alglobals".into(), Vram::new(0x800F8C90))
            .with_size(Size::new(0x4C)),
        UserSymbol::new("nnScAudioStack".into(), Vram::new(0x800F8CE0))
            .with_size(Size::new(0x2000)),
        UserSymbol::new("sound_song_id".into(), Vram::new(0x800FACE0)),
        UserSymbol::new("aiHiErR".into(), Vram::new(0x800FAD28)).with_size(Size::new(0x2)),
        UserSymbol::new("gfx_gtask_no".into(), Vram::new(0x800FAD2C)),
        UserSymbol::new("aiSelCom".into(), Vram::new(0x800FAD30)),
        UserSymbol::new("joygmf".into(), Vram::new(0x800FAD31)).with_size(Size::new(0x4)),
        UserSymbol::new("aiGoalY".into(), Vram::new(0x800FAD36)).with_size(Size::new(0x2)),
        UserSymbol::new("aipn".into(), Vram::new(0x800FAD38)).with_size(Size::new(0x1)),
        UserSymbol::new("aif_field".into(), Vram::new(0x800FAD40)).with_size(Size::new(0x110)),
        UserSymbol::new("gReverbFx".into(), Vram::new(0x800FAE50)).with_size(Size::new(0x28)),
        UserSymbol::new("evs_select_name_no".into(), Vram::new(0x800FAE78))
            .with_size(Size::new(0x2)),
        UserSymbol::new("B_800FAE80".into(), Vram::new(0x800FAE80)),
        UserSymbol::new("gControllerPressedButtons".into(), Vram::new(0x800FAF88))
            .with_size(Size::new(0xA)),
        UserSymbol::new("aiSelSpeed".into(), Vram::new(0x800FAF92)).with_size(Size::new(0x1)),
        UserSymbol::new("sched_gfxMQ".into(), Vram::new(0x800FAF94)),
        UserSymbol::new("gAudio_800FAF98".into(), Vram::new(0x800FAF98)),
        UserSymbol::new("aiHiEraseCtr".into(), Vram::new(0x800FAF9C)).with_size(Size::new(0x2)),
        UserSymbol::new("dram_stack".into(), Vram::new(0x800FAFA0)).with_size(Size::new(0x400)),
        UserSymbol::new("aiMoveSF".into(), Vram::new(0x800FB3A0)).with_size(Size::new(0x1)),
        UserSymbol::new("evs_game_time".into(), Vram::new(0x800FB3A4)),
        UserSymbol::new("attack_effect".into(), Vram::new(0x800FB3A8)).with_size(Size::new(0x2C0)),
        UserSymbol::new("gGfxGlist".into(), Vram::new(0x800FB670)).with_size(Size::new(0x18000)),
        UserSymbol::new("joycnt".into(), Vram::new(0x80113670)),
        UserSymbol::new("rdp_output".into(), Vram::new(0x801136F0)).with_size(Size::new(0x10000)),
        UserSymbol::new("evs_gamemode".into(), Vram::new(0x801236F0)).with_size(Size::new(0x4)),
        UserSymbol::new("game_state_data".into(), Vram::new(0x80123700))
            .with_size(Size::new(0xF10)),
        UserSymbol::new_ignored(Vram::new(0x00000000)),
        UserSymbol::new_ignored(Vram::new(0x80000000)),
        UserSymbol::new_ignored(Vram::new(0x80000004)),
        UserSymbol::new_ignored(Vram::new(0x80000008)),
        UserSymbol::new_ignored(Vram::new(0x8000000C)),
        UserSymbol::new_ignored(Vram::new(0xA0000000)),
        UserSymbol::new_ignored(Vram::new(0x8008F800)),
        UserSymbol::new_ignored(Vram::new(0x8008FB00)),
        UserSymbol::new_ignored(Vram::new(0x800F0535)),
    ]
}
