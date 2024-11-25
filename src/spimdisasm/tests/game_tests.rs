/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::InstructionFlags;
use spimdisasm::{
    context::{ContextBuilder, GlobalConfig, InputEndian},
    parent_segment_info::ParentSegmentInfo,
    sections::{SectionData, SectionDataSettings, SectionText, SectionTextSettings},
};

mod game_tests_info;
use game_tests_info::{
    create_drmario64_us_segments, get_ranges_from_segments, SegmentData, TestSection, TestSegment,
    TestSegmentInfo,
};

#[cfg_attr(feature = "game_tests", test)]
#[allow(dead_code)]
fn drmario64_us_without_symbols() {
    let drmario64_us_segments = create_drmario64_us_segments();

    let rom_bytes = std::fs::read("../../baserom_uncompressed.us.z64").unwrap();

    let (global_rom_range, global_vram_range) = get_ranges_from_segments(&drmario64_us_segments);

    let mut context = ContextBuilder::new(
        GlobalConfig::new(InputEndian::Big),
        global_rom_range,
        global_vram_range,
    )
    .process()
    .process()
    .process()
    .build();

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
                let mut data_sections = Vec::new();

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
                        TestSection::Data(rom, name) => {
                            let data_settings = SectionDataSettings::new();
                            data_sections.push(
                                SectionData::new(
                                    &mut context,
                                    data_settings,
                                    (*name).into(),
                                    &rom_bytes[rom.inner() as usize..rom_end.inner() as usize],
                                    *rom,
                                    info.vram_from_rom(*rom),
                                    parent_segment_info.clone(),
                                )
                                .unwrap(),
                            );
                        }
                        TestSection::Rodata(..) => {}
                        TestSection::Bss(..) => {}
                        TestSection::Bin(..) => {}
                    }
                }
                segments.push(SegmentData {
                    name: info.name.into(),
                    text_sections,
                    data_sections,
                });
            }
        }
    }

    assert_eq!(context.global_segment().symbols().len(), 1459);

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

    let data_syms_count: usize = segments
        .iter()
        .map(|x| {
            x.data_sections
                .iter()
                .map(|y| y.data_symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(data_syms_count, 57);
}
