/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{collections::HashSet, sync::Arc};

use pretty_assertions::assert_eq;

use address_space::{AddressRange, RomVramRange};
use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion};
use spimdisasm::{
    config::{Compiler, Endian, GlobalConfigBuilder},
    context::{builder::AbsoluteSegmentBuilder, Context, ContextBuilder, GlobalSegmentBuilder},
    sections::before_proc::{
        DataSectionSettings, ExecutableSectionSettings, NobitsSectionSettings,
    },
    segments::ParentSegmentInfo,
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings, SymNobitsDisplaySettings},
};

mod game_tests_info;
use game_tests_info::{
    create_drmario64_us_segments, create_drmario64_us_symbols, SegmentData, SegmentDataProcessed,
    TestSection, TestSegment, TestSegmentInfo, UserSymbol,
};

const COMPILER: Option<Compiler> = Some(Compiler::KMC);

fn init_context(
    mut symbols: Vec<UserSymbol>,
    rom_bytes: &[u8],
    user_defined_segment: &[TestSegment],
) -> Context {
    assert!(user_defined_segment.len() >= 2);

    let mut context_builder = ContextBuilder::new();

    let global_config = GlobalConfigBuilder::new(Endian::Big).build();

    for w in user_defined_segment.windows(2) {
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
                let ranges = RomVramRange::new_option(
                    AddressRange::new(info.rom, *segment_rom_end),
                    AddressRange::new(info.vram, info.vram_end(segment_rom_end.sub_rom(&info.rom))),
                    4,
                )
                .unwrap();
                let mut global_segment = GlobalSegmentBuilder::new(info.name, ranges);

                let mut remove = HashSet::new();
                for sym in &symbols {
                    if !ranges.vram().in_range(sym.vram()) {
                        continue;
                    }
                    match sym {
                        game_tests_info::UserSymbol::Info(user_symbol_info) => {
                            let mut sym = global_segment
                                .add_user_symbol(
                                    Arc::clone(&user_symbol_info.name),
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
                            *sym.rodata_migration_behavior_mut() =
                                user_symbol_info.migration_behavior.clone();
                            if user_symbol_info.dont_allow_addend {
                                sym.set_allow_ref_with_addend(false);
                            }
                        }
                        UserSymbol::Label(name, vram, label_type) => {
                            global_segment
                                .add_user_label(name.clone(), *vram, None, *label_type)
                                .unwrap();
                        }
                        game_tests_info::UserSymbol::Ignored(_vram, _size) => {
                            // TODO
                        }
                    }
                    remove.insert(sym.vram());
                }

                if !remove.is_empty() {
                    symbols.retain(|x| !remove.contains(&x.vram()));
                }

                let mut global_segment_heater = global_segment.finish_symbols();

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
                                &rom_bytes[AddressRange::new(*rom, rom_end).unwrap()],
                                *rom,
                                info.vram_from_rom(*rom),
                            )
                            .unwrap(),
                        TestSection::Data(rom, name) => global_segment_heater
                            .preheat_data(
                                &global_config,
                                &DataSectionSettings::new(COMPILER),
                                *name,
                                &rom_bytes[AddressRange::new(*rom, rom_end).unwrap()],
                                *rom,
                                info.vram_from_rom(*rom),
                            )
                            .unwrap(),
                        TestSection::Rodata(rom, name) => global_segment_heater
                            .preheat_rodata(
                                &global_config,
                                &DataSectionSettings::new(COMPILER),
                                *name,
                                &rom_bytes[AddressRange::new(*rom, rom_end).unwrap()],
                                *rom,
                                info.vram_from_rom(*rom),
                            )
                            .unwrap(),
                        TestSection::Bss(..) | TestSection::Bin(..) => {}
                    }
                }

                context_builder
                    .add_global_segment(global_segment_heater)
                    .unwrap();
            }
        }
    }

    let mut absolute_segment = AbsoluteSegmentBuilder::new();
    absolute_segment.n64_libultra_symbols().unwrap();
    absolute_segment.n64_hardware_registers(true, true).unwrap();

    context_builder
        .build(global_config, absolute_segment)
        .unwrap()
}

fn init_segments(
    context: &mut Context,
    rom_bytes: &[u8],
    user_defined_segment: Vec<TestSegment>,
) -> Vec<SegmentDataProcessed> {
    assert!(user_defined_segment.len() >= 2);

    let mut segments = Vec::new();

    for w in user_defined_segment.windows(2) {
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
                                        rom_bytes[AddressRange::new(*rom, rom_end).unwrap()]
                                            .to_vec(),
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
                                        rom_bytes[AddressRange::new(*rom, rom_end).unwrap()]
                                            .to_vec(),
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
                                        rom_bytes[AddressRange::new(*rom, rom_end).unwrap()]
                                            .to_vec(),
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
                                    + segment_rom_end.sub_rom(&info.rom)
                                    + info.noload_size.unwrap()
                            };
                            let vram_range =
                                AddressRange::new(*vram, bss_section_vram_end).unwrap();
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

    let mut context = init_context(Vec::new(), &rom_bytes, &drmario64_us_segments);

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

    static PER_SEGMENT_SYM_COUNT: [(&str, usize, usize); 9] = [
        ("header", 6, 6),
        ("ipl3", 0, 0),
        ("entry", 1, 1),
        ("boot", 515, 1248),
        ("dma_table", 4, 4),
        ("main_segment", 2814, 7358),
        ("buffer1", 2, 2),
        ("buffer2", 1, 1),
        ("framebuffer", 1, 1),
    ];
    assert_eq!(context.segments().global_segments().len(), 9);
    for (seg, expected) in context
        .segments()
        .global_segments()
        .iter()
        .zip(PER_SEGMENT_SYM_COUNT)
    {
        let name = seg.name();
        let has = (name.as_ref(), seg.symbols().len(), seg.labels().len());
        assert_eq!(has, expected);
    }

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
    assert_eq!(function_count, 1411);

    let data_syms_count: usize = segments
        .iter()
        .map(|x| {
            x.data_sections
                .iter()
                .map(|y| y.data_symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(data_syms_count, 517);
}

#[cfg_attr(feature = "game_tests", test)]
#[allow(dead_code)]
fn drmario64_us_with_symbols() {
    let drmario64_us_segments = create_drmario64_us_segments();

    let rom_bytes = std::fs::read("../../baserom_uncompressed.us.z64").unwrap();

    let mut context = init_context(
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

    /*
    for seg in context.global_segments() {
        println!("{}:", seg.name());
        for (vram, sym) in seg.symbols() {
            println!("    {} {}", vram, sym.display_name());
        }
        println!();
    }
    */

    static PER_SEGMENT_SYM_COUNT: [(&str, usize, usize); 9] = [
        ("header", 6, 6),
        ("ipl3", 0, 0),
        ("entry", 1, 1),
        ("boot", 486, 1247),
        ("dma_table", 1, 4),
        ("main_segment", 2596, 7371),
        ("buffer1", 1, 2),
        ("buffer2", 1, 1),
        ("framebuffer", 1, 1),
    ];
    assert_eq!(context.segments().global_segments().len(), 9);
    for (seg, expected) in context
        .segments()
        .global_segments()
        .iter()
        .zip(PER_SEGMENT_SYM_COUNT)
    {
        let name = seg.name();
        let has = (name.as_ref(), seg.symbols().len(), seg.labels().len());
        assert_eq!(has, expected);
    }

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
    assert_eq!(function_count, 1412);

    let data_syms_count: usize = segments
        .iter()
        .map(|x| {
            x.data_sections
                .iter()
                .map(|y| y.data_symbols().len())
                .sum::<usize>()
        })
        .sum();
    assert_eq!(data_syms_count, 461);
}
