#[cfg(feature = "linter")]
use std::hash::BuildHasher;

#[cfg(feature = "linter")]
use oxc_allocator::IdentBuildHasher;
#[cfg(feature = "linter")]
use oxc_str::Ident;

#[cfg(feature = "linter")]
const BLOOM_WORDS: usize = 32;
#[cfg(feature = "linter")]
const WORD_BITS: u64 = 64;
#[cfg(feature = "linter")]
const BLOOM_BITS: u64 = (BLOOM_WORDS as u64) * WORD_BITS;
#[cfg(feature = "linter")]
const BLOOM_MASK: u64 = BLOOM_BITS - 1;
#[cfg(feature = "linter")]
const BLOOM_HASHES: u64 = 3;

#[cfg(feature = "linter")]
const _: () = assert!(BLOOM_BITS.is_power_of_two());

#[derive(Debug, Clone, Copy)]
#[cfg(feature = "linter")]
pub enum NameFilterKind {
    GlobalReference,
    MemberExpressionProperty,
}

#[derive(Debug, Clone, Copy)]
#[cfg(feature = "linter")]
pub struct NameFilter {
    kind: NameFilterKind,
    names: &'static [&'static str],
}

#[cfg(feature = "linter")]
impl NameFilter {
    #[must_use]
    pub const fn global_reference(names: &'static [&'static str]) -> Self {
        Self { kind: NameFilterKind::GlobalReference, names }
    }

    #[must_use]
    pub const fn member_expression_property(names: &'static [&'static str]) -> Self {
        Self { kind: NameFilterKind::MemberExpressionProperty, names }
    }

    #[inline]
    fn matches(self, filters: &SemanticNameFilters) -> bool {
        self.names.iter().any(|name| match self.kind {
            NameFilterKind::GlobalReference => filters.might_contain_global_reference(name),
            NameFilterKind::MemberExpressionProperty => {
                filters.might_contain_member_expression_property(name)
            }
        })
    }
}

#[derive(Debug, Default, Clone)]
#[cfg(feature = "linter")]
pub struct SemanticNameFilters {
    global_references: NameBloomFilter,
    member_expression_properties: NameBloomFilter,
}

#[cfg(feature = "linter")]
impl SemanticNameFilters {
    #[inline]
    pub fn add_global_reference(&mut self, name: &str) {
        self.global_references.insert_str(name);
    }

    #[inline]
    pub fn add_global_reference_ident(&mut self, name: Ident<'_>) {
        self.global_references.insert_ident(name);
    }

    #[inline]
    pub fn add_member_expression_property(&mut self, name: &str) {
        self.member_expression_properties.insert_str(name);
    }

    #[inline]
    pub fn add_member_expression_property_ident(&mut self, name: Ident<'_>) {
        self.member_expression_properties.insert_ident(name);
    }

    #[inline]
    #[must_use]
    pub fn might_contain_global_reference(&self, name: &str) -> bool {
        self.global_references.might_contain_str(name)
    }

    #[inline]
    #[must_use]
    pub fn might_contain_member_expression_property(&self, name: &str) -> bool {
        self.member_expression_properties.might_contain_str(name)
    }

    #[inline]
    #[must_use]
    pub fn might_match_rule_filters(&self, filters: &[NameFilter]) -> bool {
        filters.iter().copied().all(|filter| filter.matches(self))
    }
}

#[derive(Debug, Default, Clone)]
#[cfg(feature = "linter")]
struct NameBloomFilter {
    bits: [u64; BLOOM_WORDS],
}

#[cfg(feature = "linter")]
impl NameBloomFilter {
    #[inline]
    fn insert_ident(&mut self, name: Ident<'_>) {
        self.insert_hash(IdentBuildHasher.hash_one(name));
    }

    #[inline]
    fn insert_str(&mut self, name: &str) {
        self.insert_hash(IdentBuildHasher.hash_one(name));
    }

    #[inline]
    fn might_contain_str(&self, name: &str) -> bool {
        self.might_contain_hash(IdentBuildHasher.hash_one(name))
    }

    #[inline]
    fn insert_hash(&mut self, hash: u64) {
        let (mut hash, step) = split_hash(hash);
        for _ in 0..BLOOM_HASHES {
            self.insert_bit(hash);
            hash = hash.wrapping_add(step);
        }
    }

    #[inline]
    fn might_contain_hash(&self, hash: u64) -> bool {
        let (mut hash, step) = split_hash(hash);
        for _ in 0..BLOOM_HASHES {
            if !self.has_bit(hash) {
                return false;
            }
            hash = hash.wrapping_add(step);
        }
        true
    }

    #[inline]
    fn insert_bit(&mut self, hash: u64) {
        let bit = hash & BLOOM_MASK;
        self.bits[(bit / WORD_BITS) as usize] |= 1u64 << (bit % WORD_BITS);
    }

    #[inline]
    fn has_bit(&self, hash: u64) -> bool {
        let bit = hash & BLOOM_MASK;
        (self.bits[(bit / WORD_BITS) as usize] & (1u64 << (bit % WORD_BITS))) != 0
    }
}

#[inline]
#[cfg(feature = "linter")]
fn split_hash(hash: u64) -> (u64, u64) {
    let step = hash.rotate_left(17) | 1;
    (hash, step)
}
