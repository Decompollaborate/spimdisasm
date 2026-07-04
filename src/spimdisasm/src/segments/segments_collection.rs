/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;

use ::polonius_the_crab::prelude::*;
use address_space::Vram;

use super::{
    AbsoluteSegment, OverlaysCollection, OwnedSegmentNotFoundError, ParentSegmentInfo, Segment,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentsCollection<SEG, ASEG> {
    absolute_segment: ASEG,

    global_segments: Vec<SEG>,

    overlay_segments: OverlaysCollection<SEG>,

    unknown_segment: SEG,
}

impl<SEG, ASEG> SegmentsCollection<SEG, ASEG> {
    pub(crate) fn new(
        absolute_segment: ASEG,
        global_segments: Vec<SEG>,
        overlay_segments: OverlaysCollection<SEG>,
        unknown_segment: SEG,
    ) -> Self {
        Self {
            absolute_segment,
            global_segments,
            overlay_segments,
            unknown_segment,
        }
    }

    #[must_use]
    pub const fn absolute_segment(&self) -> &ASEG {
        &self.absolute_segment
    }
    #[must_use]
    pub const fn absolute_segment_mut(&mut self) -> &mut ASEG {
        &mut self.absolute_segment
    }

    #[must_use]
    pub fn global_segments(&self) -> &[SEG] {
        &self.global_segments
    }
    #[must_use]
    pub fn global_segments_mut(&mut self) -> &mut [SEG] {
        &mut self.global_segments
    }

    #[must_use]
    pub const fn overlay_segments(&self) -> &OverlaysCollection<SEG> {
        &self.overlay_segments
    }
    #[must_use]
    pub const fn overlay_segments_mut(&mut self) -> &mut OverlaysCollection<SEG> {
        &mut self.overlay_segments
    }
}

impl<SEG, ASEG, SYM, LAB, FINDSETTINGS> SegmentsCollection<SEG, ASEG>
where
    SEG: Segment<FindSettings = FINDSETTINGS, Symbol = SYM, Label = LAB>,
    ASEG: AbsoluteSegment<FindSettings = FINDSETTINGS, Symbol = SYM>,
{
    pub fn find_owned_segment(
        &self,
        info: &ParentSegmentInfo,
    ) -> Result<&SEG, OwnedSegmentNotFoundError> {
        if let Some(overlay_name) = info.overlay_category_name() {
            if let Some(segment) = self
                .overlay_segments
                .get_segment(overlay_name, info.segment_rom())
            {
                // TODO
                /*
                debug_assert!(segment.category_name() == Some(overlay_name));
                debug_assert!(segment.rom_range().start() == info.segment_rom());
                */
                return Ok(segment);
            }
        } else {
            for seg in self.global_segments.iter() {
                if seg.rom_vram_range().in_rom_range(info.segment_rom()) {
                    // Global segment may contain more than one actual segment, so checking for ranges is okay.
                    return Ok(seg);
                } else if seg.rom_vram_range().in_vram_range(info.segment_vram()) {
                    // Global segment doesn't have overlapping issues, so it should be fine to check for vram address.
                    // This can be required by segments that only have bss sections.
                    return Ok(seg);
                }
            }
        }
        Err(OwnedSegmentNotFoundError::new(info.clone()))
    }
    pub fn find_owned_segment_mut(
        &mut self,
        info: &ParentSegmentInfo,
    ) -> Result<&mut SEG, OwnedSegmentNotFoundError> {
        if let Some(overlay_category_name) = info.overlay_category_name() {
            if let Some(segment) = self
                .overlay_segments
                .get_segment_mut(overlay_category_name, info.segment_rom())
            {
                /*
                debug_assert!(segment.category_name() == Some(overlay_category_name));
                debug_assert!(segment.rom_range().start() == info.segment_rom());
                */
                return Ok(segment);
            }
        } else {
            for seg in &mut self.global_segments {
                if seg.rom_vram_range().in_rom_range(info.segment_rom()) {
                    // Global segment may contain more than one actual segment, so checking for ranges is okay.
                    return Ok(seg);
                } else if seg.rom_vram_range().in_vram_range(info.segment_vram()) {
                    // Global segment doesn't have overlapping issues, so it should be fine to check for vram address.
                    // This can be required by segments that only have bss sections.
                    return Ok(seg);
                }
            }
        }
        Err(OwnedSegmentNotFoundError::new(info.clone()))
    }

    #[must_use]
    pub fn find_symbol_from_any_segment<V>(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
        settings: &FINDSETTINGS,
        sym_validation: V,
    ) -> Option<&SYM>
    where
        V: Fn(&&SYM) -> bool,
    {
        self.find_from_any_segment(
            vram,
            info,
            |segment| segment.find_symbol(vram, settings),
            |absolute_segment| absolute_segment.find_symbol(vram, settings),
            sym_validation,
        )
    }

    #[must_use]
    pub fn find_label_from_any_segment<V>(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
        settings: &FINDSETTINGS,
        label_validation: V,
    ) -> Option<&LAB>
    where
        V: Fn(&&LAB) -> bool,
    {
        self.find_from_any_segment(
            vram,
            info,
            |segment| segment.find_label(vram, settings),
            |_| None,
            label_validation,
        )
    }

    #[must_use]
    pub fn find_from_any_segment<'a, T: 'a, FS, FU, FV>(
        &'a self,
        vram: Vram,
        info: &ParentSegmentInfo,
        find_within_segment: FS,
        find_within_absolute_segment: FU,
        validate: FV,
    ) -> Option<T>
    where
        FS: Fn(&'a SEG) -> Option<T>,
        FU: Fn(&'a ASEG) -> Option<T>,
        FV: Fn(&T) -> bool,
    {
        if let Some(t) = find_within_absolute_segment(&self.absolute_segment) {
            return Some(t);
        }

        for seg in &self.global_segments {
            // If we find this vram is within a global segment then we can stop
            // searching, because we know this should be the only segment that
            // should overlap this segment.
            if seg.rom_vram_range().in_vram_range(vram) {
                return find_within_segment(seg).filter(|t| validate(t));
            }
        }

        if !self.overlay_segments.is_empty() {
            if let Some(t) =
                self.find_from_overlay_segments(vram, info, &find_within_segment, &validate)
            {
                return Some(t);
            }
        }

        // If we still can't find it, fall back to the unknown segment
        find_within_segment(&self.unknown_segment).filter(|t| validate(t))
    }

    #[must_use]
    fn find_from_overlay_segments<'a, T: 'a, FS, FV>(
        &'a self,
        vram: Vram,
        info: &ParentSegmentInfo,
        find_within_segment: FS,
        validate: FV,
    ) -> Option<T>
    where
        FS: Fn(&'a SEG) -> Option<T>,
        FV: Fn(&T) -> bool,
    {
        let overlay_category_name = info.overlay_category_name();

        if let Some(overlay_category_name) = overlay_category_name {
            // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.
            if let Some(owned_segment) = self
                .overlay_segments
                .get_segment(overlay_category_name, info.segment_rom())
            {
                if owned_segment.rom_vram_range().in_vram_range(vram) {
                    return find_within_segment(owned_segment);
                }

                // Check for any prioiritised overlay, if any.
                for prioritised_overlay in owned_segment.prioritised_overlays() {
                    for (_ovl_cat, segments_per_rom) in self.overlay_segments.inner() {
                        if !segments_per_rom.ranges().in_vram_range(vram) {
                            continue;
                        }
                        for (_segment_rom, segment) in segments_per_rom.segments() {
                            if &segment.name() == prioritised_overlay
                                && segment.rom_vram_range().in_vram_range(vram)
                            {
                                if let Some(t) =
                                    find_within_segment(segment).filter(|t| validate(t))
                                {
                                    return Some(t);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If not found, then we should check every category except the one that associated to the parent segment.

        // First we look for segments categories that only contain a single segment, since it is less likely to grab the wrong symbol.
        for (ovl_cat, segments_per_rom) in self.overlay_segments.inner() {
            if overlay_category_name == Some(ovl_cat)
                || !segments_per_rom.ranges().in_vram_range(vram)
            {
                continue;
            }

            let segments = segments_per_rom.segments();
            if segments.len() == 1 {
                let (_, segment) = segments
                    .into_iter()
                    .next()
                    .expect("Should exist since we already checked the length");
                if segment.rom_vram_range().in_vram_range(vram) {
                    if let Some(t) = find_within_segment(segment).filter(|t| validate(t)) {
                        return Some(t);
                    }
                }
            }
        }

        // If we haven't found the symbol yet then we just look everywhere
        for (ovl_cat, segments_per_rom) in self.overlay_segments.inner() {
            if overlay_category_name == Some(ovl_cat)
                || !segments_per_rom.ranges().in_vram_range(vram)
            {
                continue;
            }

            let segments = segments_per_rom.segments();
            if segments.len() != 1 {
                for (_, segment) in segments {
                    if segment.rom_vram_range().in_vram_range(vram) {
                        if let Some(t) = find_within_segment(segment).filter(|t| validate(t)) {
                            return Some(t);
                        }
                    }
                }
            }
        }

        None
    }
}

impl<SEG, ASEG, SYM, LAB, FINDSETTINGS> SegmentsCollection<SEG, ASEG>
where
    SEG: Segment<FindSettings = FINDSETTINGS, Symbol = SYM, Label = LAB>,
    ASEG: AbsoluteSegment<FindSettings = FINDSETTINGS, Symbol = SYM>,
{
    #[must_use]
    pub fn find_referenced_segment_mut(
        &mut self,
        vram: Vram,
        info: &ParentSegmentInfo,
    ) -> &mut SEG {
        find_referenced_segment_mut_impl(self, vram, info)
    }
}

fn find_referenced_segment_mut_impl<'ctx, SEG, ASEG, SYM, LAB, FINDSETTINGS>(
    mut slf: &'ctx mut SegmentsCollection<SEG, ASEG>,
    vram: Vram,
    info: &ParentSegmentInfo,
) -> &'ctx mut SEG
where
    SEG: Segment<FindSettings = FINDSETTINGS, Symbol = SYM, Label = LAB>,
    ASEG: AbsoluteSegment<FindSettings = FINDSETTINGS, Symbol = SYM>,
{
    polonius!(|slf| -> &'polonius mut SEG {
        for seg in &mut slf.global_segments {
            if seg.rom_vram_range().in_vram_range(vram) {
                polonius_return!(seg);
            }
        }
    });

    if !slf.overlay_segments.is_empty() {
        polonius!(|slf| -> &'polonius mut SEG {
            if let Some(owned_segment) = find_referenced_overlay_segment_mut(slf, vram, info) {
                polonius_return!(owned_segment);
            }
        });
    }

    // Fallback to the unknown segment
    &mut slf.unknown_segment
}

fn find_referenced_overlay_segment_mut<'ctx, SEG, ASEG, SYM, LAB, FINDSETTINGS>(
    mut slf: &'ctx mut SegmentsCollection<SEG, ASEG>,
    vram: Vram,
    info: &ParentSegmentInfo,
) -> Option<&'ctx mut SEG>
where
    SEG: Segment<FindSettings = FINDSETTINGS, Symbol = SYM, Label = LAB>,
    ASEG: AbsoluteSegment<FindSettings = FINDSETTINGS, Symbol = SYM>,
{
    if let Some(overlay_category_name) = info.overlay_category_name() {
        // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.

        let mut has_prioritised_overlays = false;

        polonius!(|slf| -> Option<&'polonius mut SEG> {
            if let Some(owned_segment) = slf
                .overlay_segments
                .get_segment_mut(overlay_category_name, info.segment_rom())
            {
                if owned_segment.rom_vram_range().in_vram_range(vram) {
                    polonius_return!(Some(owned_segment));
                }

                has_prioritised_overlays = !owned_segment.prioritised_overlays().is_empty();
            }
        });

        if has_prioritised_overlays {
            let mut prioritised_overlay_info = None;

            if let Some(owned_segment) = slf
                .overlay_segments
                .get_segment(overlay_category_name, info.segment_rom())
            {
                // Check for any prioiritised overlay, if any.
                for prioritised_overlay in owned_segment.prioritised_overlays() {
                    for (ovl_cat, segments_per_rom) in slf.overlay_segments.inner() {
                        if !segments_per_rom.ranges().in_vram_range(vram) {
                            continue;
                        }
                        for (segment_rom, segment) in segments_per_rom.segments() {
                            if &segment.name() == prioritised_overlay
                                && segment.rom_vram_range().in_vram_range(vram)
                            {
                                // We need to clone here to avoid lifetime issues
                                prioritised_overlay_info = Some((ovl_cat.clone(), *segment_rom));
                                break;
                            }
                        }
                        if prioritised_overlay_info.is_some() {
                            break;
                        }
                    }
                    if prioritised_overlay_info.is_some() {
                        break;
                    }
                }
            }

            if let Some((ovl_cat, segment_rom)) = prioritised_overlay_info {
                polonius!(|slf| -> Option<&'polonius mut SEG> {
                    if let Some(segment) =
                        slf.overlay_segments.get_segment_mut(&ovl_cat, segment_rom)
                    {
                        polonius_return!(Some(segment));
                    }
                });
            }
        }
    }

    None
}
