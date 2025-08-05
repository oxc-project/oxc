use std::cell::RefCell;

use oxc_span::{GetSpan, Span};

use super::chain_member::ChainMember;
use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*},
    generated::ast_nodes::AstNode,
    parentheses::NeedsParentheses,
    write,
};

#[derive(Default)]
pub(super) struct MemberChainGroupsBuilder<'a, 'b> {
    /// keeps track of the groups created
    groups: Vec<MemberChainGroup<'a, 'b>>,
    /// keeps track of the current group that is being created/updated
    current_group: Option<MemberChainGroup<'a, 'b>>,
}

impl<'a, 'b> MemberChainGroupsBuilder<'a, 'b> {
    /// starts a new group
    pub fn start_group(&mut self, member: ChainMember<'a, 'b>) {
        debug_assert!(self.current_group.is_none());
        let mut group = MemberChainGroup::default();
        group.members.push(member);
        self.current_group = Some(group);
    }

    /// continues of starts a new group
    pub fn start_or_continue_group(&mut self, member: ChainMember<'a, 'b>) {
        match &mut self.current_group {
            None => self.start_group(member),
            Some(group) => group.members.push(member),
        }
    }

    /// clears the current group, and adds it to the groups collection
    pub fn close_group(&mut self) {
        if let Some(group) = self.current_group.take() {
            self.groups.push(group);
        }
    }

    pub(super) fn finish(self) -> TailChainGroups<'a, 'b> {
        let mut groups = self.groups;

        if let Some(group) = self.current_group {
            groups.push(group);
        }

        TailChainGroups { groups }
    }
}

/// Groups following on the head group.
///
/// May be empty if all members are part of the head group
#[derive(Debug)]
pub(super) struct TailChainGroups<'a, 'b> {
    groups: Vec<MemberChainGroup<'a, 'b>>,
}

impl<'a, 'b> TailChainGroups<'a, 'b> {
    /// Returns `true` if there are no tail groups.
    pub(crate) fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Returns the number of tail groups.
    pub(crate) fn len(&self) -> usize {
        self.groups.len()
    }

    /// Returns the first group
    pub(crate) fn first(&self) -> Option<&MemberChainGroup<'a, 'b>> {
        self.groups.first()
    }

    /// Returns the last group
    pub(crate) fn last(&self) -> Option<&MemberChainGroup<'a, 'b>> {
        self.groups.last()
    }

    /// Removes the first group and returns it
    pub(super) fn pop_first(&mut self) -> Option<MemberChainGroup<'a, 'b>> {
        if self.groups.is_empty() { None } else { Some(self.groups.remove(0)) }
    }

    /// Here we check if the length of the groups exceeds the cutoff or there are comments
    /// This function is the inverse of the prettier function
    /// [Prettier applies]: <https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/member-chain.js#L342>
    pub(crate) fn is_member_call_chain(&self, f: &Formatter) -> bool {
        self.groups.len() > 1
    }

    /// Returns an iterator over the groups.
    pub(super) fn iter(&self) -> impl Iterator<Item = &MemberChainGroup<'a, 'b>> {
        self.groups.iter()
    }

    /// Test if any group except the last group [break](FormatElements::will_break).
    pub(super) fn any_except_last_will_break(
        &self,
        f: &mut Formatter<'_, 'a>,
    ) -> FormatResult<bool> {
        for group in &self.groups[..self.groups.len().saturating_sub(1)] {
            if group.will_break(f)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns an iterator over all members
    pub(super) fn members(&self) -> impl Iterator<Item = &ChainMember<'a, 'b>> {
        self.groups.iter().flat_map(|group| group.members().iter())
    }
}

impl<'a> Format<'a> for TailChainGroups<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.join().entries(self.groups.iter()).finish()
    }
}

#[derive(Default)]
pub(super) struct MemberChainGroup<'a, 'b> {
    members: Vec<ChainMember<'a, 'b>>,

    /// Stores the formatted result of this group.
    ///
    /// Manual implementation of `Memoized` to only memorizing the formatted result
    /// if [MemberChainGroup::will_break] is called but not otherwise.
    formatted: RefCell<Option<FormatElement<'a>>>,
}

impl<'a, 'b> MemberChainGroup<'a, 'b> {
    pub(super) fn into_members(self) -> Vec<ChainMember<'a, 'b>> {
        self.members
    }

    /// Returns the chain members of the group.
    pub(super) fn members(&self) -> &[ChainMember<'a, 'b>] {
        &self.members
    }

    /// Extends the members of this group with the passed in members
    pub(super) fn extend_members(
        &mut self,
        members: impl IntoIterator<Item = ChainMember<'a, 'b>>,
    ) {
        self.members.extend(members);
    }

    /// Tests if the formatted result of this group results in a [break](FormatElements::will_break).
    pub(super) fn will_break(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<bool> {
        let mut cell = self.formatted.borrow_mut();
        Ok(if let Some(formatted) = cell.as_ref() {
            formatted.will_break()
        } else {
            let interned = f.intern(&FormatMemberChainGroup { group: self })?;

            if let Some(interned) = interned {
                let breaks = interned.will_break();
                *cell = Some(interned);
                breaks
            } else {
                false
            }
        })
    }

    pub(super) fn needs_empty_line_before(&self, f: &Formatter) -> bool {
        // TODO: Needs to consider there is a comment around the `?`, `.` or `[` character.
        let Some(ChainMember::StaticMember(expression)) = self.members.first() else {
            return false;
        };

        let source_text = f.source_text().as_bytes();

        // `A \n\n/* comment . */ . / comment . */ B`
        //    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        // Check whether has more than 1 continuous new lines before the operator (`.`)
        let start = expression.object().span().end;
        let mut end = expression.property().span().start;
        let mut comments = f.comments().comments_between(start, end).iter();
        let mut last_comment_span = comments.next_back();

        while start < end {
            // Skip comments that are after the operator (`.`)
            if let Some(last_comment) = last_comment_span
                && last_comment.span.end == end
            {
                end = last_comment.span.start - 1;
                last_comment_span = comments.next_back();
                continue;
            } else if matches!(source_text[end as usize], b'.') {
                // Found the operator, stop the loop
                break;
            }

            end -= 1;
        }

        // Skip comments that are before the operator (`.`)
        if let Some(comment) = comments.next() {
            end = comment.span.start;
        }

        // Count the number of continuous new lines
        let mut new_lines_count = 0;
        for &b in &source_text[start as usize..end as usize] {
            if matches!(b, b'\n' | b'\r') {
                new_lines_count += 1;
                // If there are more than 1 continuous new lines, return true
                if new_lines_count > 1 {
                    return true;
                }
            } else if !b.is_ascii_whitespace() {
                // Reset the counter if there is a non-new-line character,
                // because the new lines are not continuous
                new_lines_count = 0;
            }
        }

        false
    }
}

impl<'a, 'b> From<Vec<ChainMember<'a, 'b>>> for MemberChainGroup<'a, 'b> {
    fn from(entries: Vec<ChainMember<'a, 'b>>) -> Self {
        Self { members: entries, formatted: RefCell::new(None) }
    }
}

impl std::fmt::Debug for MemberChainGroup<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MemberChainGroup").field(&self.members).finish()
    }
}

impl<'a> Format<'a> for MemberChainGroup<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let Some(formatted) = self.formatted.borrow().as_ref() {
            return f.write_element(formatted.clone());
        }

        FormatMemberChainGroup { group: self }.fmt(f)
    }
}

pub struct FormatMemberChainGroup<'a, 'b> {
    group: &'b MemberChainGroup<'a, 'b>,
}

impl<'a> Format<'a> for FormatMemberChainGroup<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        f.join().entries(self.group.members.iter()).finish()
    }
}
