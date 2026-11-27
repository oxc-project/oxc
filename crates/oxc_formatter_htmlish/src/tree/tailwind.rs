//! The attributes whose values hold Tailwind classes, from `sortTailwindcss`'s list,
//! as prettier-plugin-tailwindcss reads its `tailwindAttributes` per parser (`createMatcher`).

use oxc_allocator::{Allocator, ArenaVec};

use crate::HtmlishVariant;

pub struct TailwindAttributes<'a> {
    /// The names whose values are classes: `class` and each listed one;
    /// in Vue a listed binding (`:a`, `v-bind:a`) names its attribute (`a`), in Angular one in brackets (`[a]`) too.
    names: ArenaVec<'a, &'a str>,
    /// Vue: the directives listed as written (`v-a`), whose values are JS.
    directives: ArenaVec<'a, &'a str>,
    /// Angular: `ngClass` too, bound (`[ngClass]`).
    ng_class: bool,
}

impl<'a> TailwindAttributes<'a> {
    pub fn new(allocator: &'a Allocator, listed: &[String], variant: HtmlishVariant) -> Self {
        let mut names = ArenaVec::from_iter_in(["class"], &allocator);
        let mut directives = ArenaVec::new_in(&allocator);
        for listed in listed {
            let name = match variant {
                HtmlishVariant::Vue => {
                    listed.strip_prefix(':').or_else(|| listed.strip_prefix("v-bind:"))
                }
                HtmlishVariant::Angular => {
                    listed.strip_prefix('[').and_then(|l| l.strip_suffix(']'))
                }
                HtmlishVariant::Html | HtmlishVariant::Xhtml => None,
            };
            if variant == HtmlishVariant::Vue && name.is_none() && listed.starts_with("v-") {
                directives.push(allocator.alloc_str(listed));
            } else {
                names.push(allocator.alloc_str(name.unwrap_or(listed)));
            }
        }
        Self { names, directives, ng_class: variant == HtmlishVariant::Angular }
    }

    /// An attribute named `name` holds classes, as a static one's value or a binding's (Vue's `:a`'s argument).
    pub fn holds(&self, name: &str) -> bool {
        self.names.contains(&name)
    }

    /// An Angular property binding to `target` holds classes (`[class]`, `[ngClass]`, `bind-class`;
    /// apps/oxfmt/DIVERGENCES.md#js-in-angular-tailwind-bind-class): every string in it, its pipes' arguments included.
    pub fn binds(&self, target: &str) -> bool {
        self.holds(target) || (self.ng_class && target == "ngClass")
    }

    /// A Vue directive named `name` (as written) holds classes.
    pub fn directive(&self, name: &str) -> bool {
        self.directives.contains(&name)
    }
}
