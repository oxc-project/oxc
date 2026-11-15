//! Live ranges of the candidates of one scope, from its [`FlowBuffers`].
//!
//! A single backward pass over the blocks suffices: forward edges are resolved from successors
//! already visited, and a variable live on entry to a loop header is kept live across the whole
//! loop, which covers everything a backward edge could carry.

use oxc_allocator::{Allocator, ArenaVec};

use super::flow::{EDGE_EXCEPTIONAL, EDGE_TO_LABEL, FlowBuffers, LABEL_PENDING};

/// Bound on `blocks × ⌈locals / 64⌉`, the size of the live-in sets.
const MAX_LIVE_IN_WORDS: usize = 1 << 20;

/// Marks an exceptional successor in [`Liveness::successors`].
const EXCEPTIONAL: u32 = 1 << 31;

/// Positions `start..=end`.
#[derive(Clone, Copy)]
pub(super) struct Range {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy)]
struct LocalRange {
    local: u32,
    start: u32,
    end: u32,
}

pub(super) struct Liveness<'t> {
    /// Successors of each block, as offsets into `successors`. Length `blocks + 1`.
    successor_start: ArenaVec<'t, u32>,
    successors: ArenaVec<'t, u32>,
    cursor: ArenaVec<'t, u32>,
    /// One bit set of live variables per block, on entry to the block.
    live_in: ArenaVec<'t, u64>,
    live: ArenaVec<'t, u64>,
    exit: ArenaVec<'t, u64>,
    /// Variables live on entry to an exceptional successor, which stay live across the block.
    exceptional: ArenaVec<'t, u64>,
    /// Ranges in the order they are found.
    found: ArenaVec<'t, LocalRange>,
    /// The end of the open range of each live variable.
    open: ArenaVec<'t, u32>,
    /// The sorted, coalesced ranges of all variables.
    ranges: ArenaVec<'t, Range>,
    /// Ranges of each variable, as offsets into `ranges`. Length `locals + 1`.
    range_start: ArenaVec<'t, u32>,
}

impl<'t> Liveness<'t> {
    pub(super) fn new_in(allocator: &'t Allocator) -> Self {
        Self {
            successor_start: ArenaVec::new_in(&allocator),
            successors: ArenaVec::new_in(&allocator),
            cursor: ArenaVec::new_in(&allocator),
            live_in: ArenaVec::new_in(&allocator),
            live: ArenaVec::new_in(&allocator),
            exit: ArenaVec::new_in(&allocator),
            exceptional: ArenaVec::new_in(&allocator),
            found: ArenaVec::new_in(&allocator),
            open: ArenaVec::new_in(&allocator),
            ranges: ArenaVec::new_in(&allocator),
            range_start: ArenaVec::new_in(&allocator),
        }
    }

    /// The sorted, disjoint, non-adjacent ranges in which `local` is live.
    pub(super) fn ranges(&self, local: u32) -> &[Range] {
        let local = local as usize;
        &self.ranges[self.range_start[local] as usize..self.range_start[local + 1] as usize]
    }

    /// Compute the live ranges of `locals` variables. Returns `false` if the graph is too large,
    /// or is not structured as expected.
    pub(super) fn compute(&mut self, flow: &FlowBuffers<'_, '_>, locals: usize) -> bool {
        let words = locals.div_ceil(64);
        if flow.blocks.len().saturating_mul(words) > MAX_LIVE_IN_WORDS || !self.link(flow) {
            return false;
        }
        let Self {
            successor_start,
            successors,
            live_in,
            live,
            exit,
            exceptional,
            found,
            open,
            ranges,
            range_start,
            ..
        } = self;
        live_in.clear();
        live_in.resize(flow.blocks.len() * words, 0);
        live.clear();
        live.resize(words, 0);
        exit.clear();
        exit.resize(words, 0);
        exceptional.clear();
        exceptional.resize(words, 0);
        found.clear();
        open.clear();
        open.resize(locals, 0);

        // `live` holds the variables live on entry to the block after `index`, each with a range
        // open from that block's start to `open[local]`.
        let mut next_start = 0;
        for (index, block) in flow.blocks.iter().enumerate().rev() {
            exit.fill(0);
            exceptional.fill(0);
            let successors =
                &successors[successor_start[index] as usize..successor_start[index + 1] as usize];
            for &successor in successors {
                let target = (successor & !EXCEPTIONAL) as usize;
                // Backward edges go to a loop header; the loop is handled at the header.
                if target <= index {
                    continue;
                }
                let target_live_in = &live_in[target * words..(target + 1) * words];
                for (word, &bits) in exit.iter_mut().zip(target_live_in) {
                    *word |= bits;
                }
                if successor & EXCEPTIONAL != 0 {
                    for (word, &bits) in exceptional.iter_mut().zip(target_live_in) {
                        *word |= bits;
                    }
                }
            }
            // Blocks are contiguous, so a variable live on both sides of a boundary keeps its
            // range open.
            for (word_index, (word, &exit_bits)) in live.iter_mut().zip(exit.iter()).enumerate() {
                for_each_bit(word_index, *word & !exit_bits, |local| {
                    found.push(LocalRange {
                        local: to_u32(local),
                        start: next_start,
                        end: open[local],
                    });
                });
                for_each_bit(word_index, exit_bits & !*word, |local| open[local] = block.end);
                *word = exit_bits;
            }

            let events_end = flow
                .blocks
                .get(index + 1)
                .map_or(flow.events.len(), |next| next.first_event as usize);
            for event in flow.events[block.first_event as usize..events_end].iter().rev() {
                let local = event.local() as usize;
                let (word, bit) = (local / 64, 1u64 << (local % 64));
                if event.is_write() {
                    if exceptional[word] & bit != 0 {
                        continue;
                    }
                    let end = if live[word] & bit == 0 { event.position } else { open[local] };
                    found.push(LocalRange { local: to_u32(local), start: event.position, end });
                    live[word] &= !bit;
                } else if live[word] & bit == 0 {
                    open[local] = event.position;
                    live[word] |= bit;
                }
            }

            if block.loop_end != 0 {
                for (word_index, &bits) in live.iter().enumerate() {
                    for_each_bit(word_index, bits, |local| {
                        found.push(LocalRange {
                            local: to_u32(local),
                            start: block.start,
                            end: block.loop_end,
                        });
                    });
                }
            }
            live_in[index * words..(index + 1) * words].copy_from_slice(live);
            next_start = block.start;
        }
        for (word_index, &bits) in live.iter().enumerate() {
            for_each_bit(word_index, bits, |local| {
                found.push(LocalRange {
                    local: to_u32(local),
                    start: next_start,
                    end: open[local],
                });
            });
        }

        found.sort_unstable_by_key(|range| (range.local, range.start));
        ranges.clear();
        range_start.clear();
        let mut found_ranges = found.iter().peekable();
        for local in 0..locals {
            let first = ranges.len();
            range_start.push(to_u32(first));
            while let Some(range) = found_ranges.next_if(|range| range.local as usize == local) {
                if ranges.len() > first
                    && let Some(last) = ranges.last_mut()
                    && range.start <= last.end.saturating_add(1)
                {
                    last.end = last.end.max(range.end);
                } else {
                    ranges.push(Range { start: range.start, end: range.end });
                }
            }
        }
        range_start.push(to_u32(ranges.len()));
        true
    }

    /// Build the successor lists. Returns `false` if a label was never placed, or a backward edge
    /// leaves its loop.
    fn link(&mut self, flow: &FlowBuffers<'_, '_>) -> bool {
        let Self { successor_start, successors, cursor, .. } = self;
        successor_start.clear();
        successor_start.resize(flow.blocks.len() + 1, 0);
        for edge in &flow.edges {
            successor_start[edge.from as usize + 1] += 1;
        }
        for index in 1..successor_start.len() {
            successor_start[index] += successor_start[index - 1];
        }
        cursor.clear();
        cursor.extend_from_slice(&successor_start[..flow.blocks.len()]);
        successors.clear();
        successors.resize(flow.edges.len(), 0);

        for edge in &flow.edges {
            let target = if edge.flags & EDGE_TO_LABEL == 0 {
                edge.to
            } else {
                let target = flow.labels[edge.to as usize];
                if target >= LABEL_PENDING {
                    return false;
                }
                target
            };
            if target <= edge.from
                && flow.blocks[target as usize].loop_end < flow.blocks[edge.from as usize].end
            {
                return false;
            }
            let slot = &mut cursor[edge.from as usize];
            successors[*slot as usize] =
                target | if edge.flags & EDGE_EXCEPTIONAL == 0 { 0 } else { EXCEPTIONAL };
            *slot += 1;
        }
        true
    }
}

fn for_each_bit(word_index: usize, mut bits: u64, mut f: impl FnMut(usize)) {
    while bits != 0 {
        f(word_index * 64 + bits.trailing_zeros() as usize);
        bits &= bits - 1;
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
