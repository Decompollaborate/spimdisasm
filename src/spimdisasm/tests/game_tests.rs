/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pretty_assertions::assert_eq;
use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion};
use spimdisasm::{
    addresses::{AddressRange, RomVramRange},
    config::{Compiler, Endian, GlobalConfigBuilder},
    context::{builder::UserSegmentBuilder, Context, ContextBuilder, GlobalSegmentBuilder},
    parent_segment_info::ParentSegmentInfo,
    sections::before_proc::{
        DataSectionSettings, ExecutableSectionSettings, NobitsSectionSettings,
    },
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings, SymNobitsDisplaySettings},
};

mod game_tests_info;
use game_tests_info::{
    create_drmario64_us_segments, create_drmario64_us_symbols, SegmentData, SegmentDataProcessed,
    TestSection, TestSegment, TestSegmentInfo, UserSymbol,
};

const COMPILER: Option<Compiler> = Some(Compiler::KMC);

pub fn get_ranges_from_segments(segments: &[TestSegment]) -> RomVramRange {
    let mut rom_start = None;
    let mut rom_end = None;
    let mut vram_start = None;
    let mut vram_end = None;

    for w in segments.windows(2) {
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

    let global_rom_range = AddressRange::new(rom_start.unwrap(), rom_end.unwrap());
    let global_vram_range = AddressRange::new(vram_start.unwrap(), vram_end.unwrap());

    RomVramRange::new(global_rom_range, global_vram_range)
}

fn init_context(
    global_ranges: RomVramRange,
    symbols: Vec<UserSymbol>,
    rom_bytes: &[u8],
    user_segments: &[TestSegment],
) -> Context {
    assert!(user_segments.len() >= 2);

    let global_config = GlobalConfigBuilder::new(Endian::Big).build();
    let mut global_segment = GlobalSegmentBuilder::new(global_ranges);

    for sym in symbols {
        match sym {
            game_tests_info::UserSymbol::Info(user_symbol_info) => {
                let mut sym = global_segment
                    .add_user_symbol(
                        user_symbol_info.name,
                        user_symbol_info.vram,
                        user_symbol_info.rom,
                        user_symbol_info.size,
                        user_symbol_info.typ,
                    )
                    .unwrap();

                // TODO:
                // if let Some(name_end) = user_symbol_info.name_end {
                //     sym.set_user_declared_name_end(name_end);
                // }
                *sym.rodata_migration_behavior_mut() = user_symbol_info.migration_behavior;
                if user_symbol_info.dont_allow_addend {
                    sym.set_allow_ref_with_addend(false);
                }
            }
            UserSymbol::Label(name, vram, label_type) => {
                global_segment
                    .add_user_label(name, vram, None, label_type)
                    .unwrap();
            }
            game_tests_info::UserSymbol::Ignored(_vram, _size) => {}
        }
    }

    let mut global_segment_heater = global_segment.finish_symbols();

    for w in user_segments.windows(2) {
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
                        TestSection::Text(rom, name) => global_segment_heater
                            .preheat_text(
                                &global_config,
                                &ExecutableSectionSettings::new(
                                    COMPILER,
                                    InstructionFlags::new(IsaVersion::MIPS_III),
                                ),
                                *name,
                                &rom_bytes[AddressRange::new(*rom, rom_end)],
                                *rom,
                                info.vram_from_rom(*rom),
                            )
                            .unwrap(),
                        TestSection::Data(rom, name) => global_segment_heater
                            .preheat_data(
                                &global_config,
                                &DataSectionSettings::new(COMPILER),
                                *name,
                                &rom_bytes[AddressRange::new(*rom, rom_end)],
                                *rom,
                                info.vram_from_rom(*rom),
                            )
                            .unwrap(),
                        TestSection::Rodata(rom, name) => global_segment_heater
                            .preheat_rodata(
                                &global_config,
                                &DataSectionSettings::new(COMPILER),
                                *name,
                                &rom_bytes[AddressRange::new(*rom, rom_end)],
                                *rom,
                                info.vram_from_rom(*rom),
                            )
                            .unwrap(),
                        TestSection::Bss(..) | TestSection::Bin(..) => {}
                    }
                }
            }
        }
    }

    let mut platform_segment = UserSegmentBuilder::new();
    platform_segment.n64_libultra_symbols().unwrap();
    platform_segment.n64_hardware_registers(true, true).unwrap();

    let builder = ContextBuilder::new(global_segment_heater, platform_segment);
    builder.build(global_config).unwrap()
}

fn init_segments(
    context: &mut Context,
    rom_bytes: &[u8],
    user_segments: Vec<TestSegment>,
) -> Vec<SegmentDataProcessed> {
    assert!(user_segments.len() >= 2);

    let mut segments = Vec::new();

    for w in user_segments.windows(2) {
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
                let mut rodata_sections = Vec::new();
                let mut bss_sections = Vec::new();

                let parent_segment_info = ParentSegmentInfo::new(info.rom, info.vram, None);

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
                            let text_settings = ExecutableSectionSettings::new(
                                COMPILER,
                                InstructionFlags::new(IsaVersion::MIPS_III),
                            );
                            text_sections.push(
                                context
                                    .create_section_text(
                                        &text_settings,
                                        *name,
                                        rom_bytes[AddressRange::new(*rom, rom_end)].to_vec(),
                                        *rom,
                                        info.vram_from_rom(*rom),
                                        parent_segment_info.clone(),
                                    )
                                    .unwrap(),
                            );
                        }
                        TestSection::Data(rom, name) => {
                            let data_settings = DataSectionSettings::new(COMPILER);
                            data_sections.push(
                                context
                                    .create_section_data(
                                        &data_settings,
                                        *name,
                                        rom_bytes[AddressRange::new(*rom, rom_end)].to_vec(),
                                        *rom,
                                        info.vram_from_rom(*rom),
                                        parent_segment_info.clone(),
                                    )
                                    .unwrap(),
                            );
                        }
                        TestSection::Rodata(rom, name) => {
                            let rodata_settings = DataSectionSettings::new(COMPILER);
                            rodata_sections.push(
                                context
                                    .create_section_rodata(
                                        &rodata_settings,
                                        *name,
                                        rom_bytes[AddressRange::new(*rom, rom_end)].to_vec(),
                                        *rom,
                                        info.vram_from_rom(*rom),
                                        parent_segment_info.clone(),
                                    )
                                    .unwrap(),
                            );
                        }
                        TestSection::Bss(vram, name) => {
                            let bss_settings = NobitsSectionSettings::new(COMPILER);

                            let bss_section_vram_end = if i + 1 < info.sections.len() {
                                match info.sections[i + 1] {
                                    TestSection::Text(..)
                                    | TestSection::Data(..)
                                    | TestSection::Rodata(..)
                                    | TestSection::Bin(..) => panic!("load follows noload????"),
                                    TestSection::Bss(next_vram, ..) => next_vram,
                                }
                            } else {
                                info.vram
                                    + (*segment_rom_end - info.rom)
                                    + info.noload_size.unwrap()
                            };
                            let vram_range = AddressRange::new(*vram, bss_section_vram_end);
                            bss_sections.push(
                                context
                                    .create_section_bss(
                                        &bss_settings,
                                        *name,
                                        vram_range,
                                        parent_segment_info.clone(),
                                    )
                                    .unwrap(),
                            );
                        }
                        TestSection::Bin(..) => {}
                    }
                }
                segments.push(SegmentData {
                    name: info.name.into(),
                    text_sections,
                    data_sections,
                    rodata_sections,
                    bss_sections,
                });
            }
        }
    }

    segments
        .into_iter()
        .map(|x| x.post_process(context))
        .collect()
}

#[cfg_attr(feature = "game_tests", test)]
#[allow(dead_code)]
fn drmario64_us_without_symbols() {
    let drmario64_us_segments = create_drmario64_us_segments();

    let rom_bytes = std::fs::read("../../baserom_uncompressed.us.z64").unwrap();

    let global_ranges = get_ranges_from_segments(&drmario64_us_segments);
    println!("Global ranges: {:?}", global_ranges);

    let mut context = init_context(
        global_ranges,
        Vec::new(),
        &rom_bytes,
        &drmario64_us_segments,
    );

    let segments = init_segments(&mut context, &rom_bytes, drmario64_us_segments);

    let instr_display_flags = InstructionDisplayFlags::default();
    let function_display_settings = FunctionDisplaySettings::new(instr_display_flags);
    let sym_data_display_settings = SymDataDisplaySettings::new();
    let sym_noload_display_settings = SymNobitsDisplaySettings::new();
    for seg in &segments {
        for sect in &seg.text_sections {
            for sym in sect.symbols() {
                // sym.display(&context, &function_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(
                        &context,
                        &function_display_settings,
                        &sym_data_display_settings,
                    )
                    .unwrap()
                    .to_string();
            }
        }
        for sect in &seg.data_sections {
            for sym in sect.data_symbols() {
                // sym.display(&context, &data_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(&context, &sym_data_display_settings)
                    .unwrap()
                    .to_string();
            }
        }
        for sect in &seg.rodata_sections {
            for sym in sect.data_symbols() {
                // sym.display(&context, &data_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(&context, &sym_data_display_settings)
                    .unwrap()
                    .to_string();
            }
        }
        for sect in &seg.bss_sections {
            for sym in sect.nobits_symbols() {
                // sym.display(&context, &data_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(&context, &sym_noload_display_settings)
                    .unwrap()
                    .to_string();
            }
        }
    }

    assert_eq!(context.global_segment().symbols().len(), 3400);
    assert_eq!(context.global_segment().labels().len(), 8594);

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
                .map(|y| y.symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(function_count, 1417);

    let data_syms_count: usize = segments
        .iter()
        .map(|x| {
            x.data_sections
                .iter()
                .map(|y| y.data_symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(data_syms_count, 545);
}

#[cfg_attr(feature = "game_tests", test)]
#[allow(dead_code)]
fn drmario64_us_with_symbols() {
    let drmario64_us_segments = create_drmario64_us_segments();

    let rom_bytes = std::fs::read("../../baserom_uncompressed.us.z64").unwrap();

    let global_ranges = get_ranges_from_segments(&drmario64_us_segments);
    println!("Global ranges: {:?}", global_ranges);

    let mut context = init_context(
        global_ranges,
        create_drmario64_us_symbols(),
        &rom_bytes,
        &drmario64_us_segments,
    );

    let segments = init_segments(&mut context, &rom_bytes, drmario64_us_segments);

    let instr_display_flags = InstructionDisplayFlags::default();
    let function_display_settings = FunctionDisplaySettings::new(instr_display_flags);
    let sym_data_display_settings = SymDataDisplaySettings::new();
    let sym_noload_display_settings = SymNobitsDisplaySettings::new();
    for seg in &segments {
        for sect in &seg.text_sections {
            for sym in sect.symbols() {
                // sym.display(&context, &function_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(
                        &context,
                        &function_display_settings,
                        &sym_data_display_settings,
                    )
                    .unwrap()
                    .to_string();
            }
        }
        for sect in &seg.data_sections {
            for sym in sect.data_symbols() {
                // sym.display(&context, &data_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(&context, &sym_data_display_settings)
                    .unwrap()
                    .to_string();
            }
        }
        for sect in &seg.rodata_sections {
            for sym in sect.data_symbols() {
                // sym.display(&context, &data_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(&context, &sym_data_display_settings)
                    .unwrap()
                    .to_string();
            }
        }
        for sect in &seg.bss_sections {
            for sym in sect.nobits_symbols() {
                // sym.display(&context, &data_display_settings).hash(&mut hasher);
                let _a = sym
                    .display(&context, &sym_noload_display_settings)
                    .unwrap()
                    .to_string();
            }
        }
    }

    assert_eq!(context.global_segment().symbols().len(), 3097);
    assert_eq!(context.global_segment().labels().len(), 8637);

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
                .map(|y| y.symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(function_count, 1413);

    let data_syms_count: usize = segments
        .iter()
        .map(|x| {
            x.data_sections
                .iter()
                .map(|y| y.data_symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(data_syms_count, 460);
}
