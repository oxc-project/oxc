use super::chain_member::ChainMember;
use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*},
    generated::ast_nodes::AstNode,
    parentheses::NeedsParentheses,
    write,
};
use oxc_span::{GetSpan, Span};
use std::cell::RefCell;

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

    /// adds the passed element to the current group.
    ///
    /// # Panics
    ///
    /// If there's no started group.
    pub fn continue_group(&mut self, member: ChainMember<'a, 'b>) {
        match &mut self.current_group {
            None => {
                panic!("It is necessary to start a group first using `start_group`.");
            }
            Some(group) => {
                group.members.push(member);
            }
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

    /// Checks if the groups contain comments.
    pub fn has_comments(&self, f: &Formatter) -> bool {
        let mut members = self.groups.iter().flat_map(|item| item.members.iter());

        let has_comments = members.any(|item| {
            false
            // f.comments().has_trailing_comments(item.syntax().span())
            //     || f.comments().has_leading_comments(item.syntax().span())
        });

        let cutoff_has_leading_comments = if !self.groups.is_empty() {
            let group = self.groups.get(1);
            if let Some(group) = group {
                let first_item = group.members.first();
                first_item.is_some_and(|first_item| {
                    false
                    // f.comments().has_leading_comments(first_item.syntax().span())
                })
            } else {
                false
            }
        } else {
            false
        };

        has_comments || cutoff_has_leading_comments
    }

    /// Here we check if the length of the groups exceeds the cutoff or there are comments
    /// This function is the inverse of the prettier function
    /// [Prettier applies]: https://github.com/prettier/prettier/blob/a043ac0d733c4d53f980aa73807a63fc914f23bd/src/language-js/print/member-chain.js#L342
    pub(crate) fn is_member_call_chain(&self, f: &Formatter) -> bool {
        self.groups.len() > 1 || self.has_comments(f)
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

impl<'a, 'b> Format<'a> for TailChainGroups<'a, 'b> {
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
        self.members.extend(members)
    }

    /// Tests if the formatted result of this group results in a [break](FormatElements::will_break).
    pub(super) fn will_break(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<bool> {
        let mut cell = self.formatted.borrow_mut();
        let result = match cell.as_ref() {
            Some(formatted) => formatted.will_break(),
            None => {
                let interned = f.intern(&FormatMemberChainGroup { group: self })?;

                if let Some(interned) = interned {
                    let breaks = interned.will_break();
                    *cell = Some(interned);
                    breaks
                } else {
                    false
                }
            }
        };

        Ok(result)
    }

    pub(super) fn has_comments(&self, f: &Formatter) -> bool {
        self.members.iter().enumerate().any(|(index, member)| {
            false
            // if index == 0 {
            //     f.comments().has_trailing_comments(member.syntax().span())
            // } else if index < self.members.len() {
            //     f.comments().has_leading_comments(member.syntax().span())
            //         || f.comments().has_trailing_comments(member.syntax().span())
            // } else {
            //     false
            // }
        })
    }

    pub(super) fn needs_empty_line_before(&self, f: &Formatter) -> bool {
        // TODO: Needs to consider there is a comment around the `?`, `.` or `[` character.
        let first = self.members.first();
        let result = first.is_some_and(|first| match first {
            ChainMember::StaticMember(expression) => {
                let object_end = expression.object().span().end;
                let operator_byte = if expression.optional { b'?' } else { b'.' };
                let operator_position = f.source_text().as_bytes()[(object_end as usize)..]
                    .iter()
                    .position(|&b| b == operator_byte)
                    .unwrap();

                let start = object_end + operator_position as u32;
                let end = start + if expression.optional { 2 } else { 1 };
                let operator_span = Span::new(start, end);

                get_lines_before(operator_span, f) > 1
            }
            ChainMember::ComputedMember(expression) => {
                let object_end = expression.object().span().end;
                let operator_byte = b'[';
                let operator_position = f.source_text().as_bytes()[(object_end as usize)..]
                    .iter()
                    .position(|&b| b == operator_byte)
                    .unwrap();

                let start = object_end + operator_position as u32;
                let end = start + 1;
                let operator_span = Span::new(start, end);

                get_lines_before(operator_span, f) > 1
            }
            _ => false,
        });

        result
    }
}

impl<'a, 'b> From<Vec<ChainMember<'a, 'b>>> for MemberChainGroup<'a, 'b> {
    fn from(entries: Vec<ChainMember<'a, 'b>>) -> Self {
        Self { members: entries, formatted: RefCell::new(None) }
    }
}

impl<'a, 'b> std::fmt::Debug for MemberChainGroup<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MemberChainGroup").field(&self.members).finish()
    }
}

impl<'a, 'b> Format<'a> for MemberChainGroup<'a, 'b> {
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

impl<'a, 'b> Format<'a> for FormatMemberChainGroup<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let group = self.group;

        let last = group.members.last();

        let needs_parens = last.is_some_and(|last| match last {
            ChainMember::StaticMember(expression) => expression.needs_parentheses(f),
            ChainMember::ComputedMember(expression) => expression.needs_parentheses(f),
            _ => false,
        });

        // TODO: the assert here is for checking whether `needs_parens` check is unnecessary.
        debug_assert!(!needs_parens);

        let format_entries = format_with(|f| f.join().entries(group.members.iter()).finish());

        if needs_parens {
            write!(f, [text("("), format_entries, text(")")])
        } else {
            write!(f, [format_entries])
        }
    }
}
