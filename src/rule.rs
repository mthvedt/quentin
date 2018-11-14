//! Rules in a context-free grammar.

use std::collections::HashMap;
use util::arena::{ForwardArena, ForwardRef};

pub type ItemRef<'a> = ForwardRef<'a, Item<'a>>;

pub trait Rule {
    fn build_item<'a>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'a>) -> Item<'a>;
}

/// It's actually unsound in Rust to use trait references as keys!
/// The reason references are partial keys is that
/// a single struct might have two different trait implementations in scope in two different places,
/// so the Trait pointer is not guaranteed to point to the same vtable.
/// Even more generally, Rust may (and does) at its option
/// generate two different vtables for the same implementation.
#[derive(Clone, Copy)]
pub enum RuleKey<'a> {
    Ref(&'a Rule),
    // This is 'static for now to avoid more lifetimes everywhere.
    // Will generalize this key wholesale later.
    Lookup(&'static str),
}

pub trait AsRuleKey {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a>;
}

impl<R: Rule> AsRuleKey for R {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a> {
        RuleKey::Ref(self)
    }
}

impl AsRuleKey for &'static str {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a> {
        RuleKey::Lookup(self)
    }
}

// TODO: Put rules in a context-free-grammar module, hide this in a generic Rule module.
// TODO: generify this on some kind of key.
pub enum Combinator<'a> {
    Seq(ItemRef<'a>, ItemRef<'a>),
    Choice(ItemRef<'a>, ItemRef<'a>),
    Term(u8),
    Done,
    Empty,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Class {
    Normal,
    Passthrough,
    Elide,
}

pub struct Item<'a> {
    combinator: Combinator<'a>,
    class: Class,
}

struct WithClass<R: Rule> {
    wrapped: R,
    class: Class,
}

impl<R: Rule> WithClass<R> {
    fn new(wrapped: R, class: Class) -> Self {
        Self {
            wrapped: wrapped,
            class: class,
        }
    }
}

impl<R: Rule> Rule for WithClass<R> {
    fn build_item<'a>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'a>) -> Item<'a> {
        let item = self.wrapped.build_item(f);
        Item {
            combinator: item.combinator,
            class: self.class,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Seq<'a> {
    rules: &'a [&'a AsRuleKey],
}

impl<'a> Seq<'a> {
    pub fn new(rules: &'a [&'a AsRuleKey]) -> Self {
        Seq { rules: rules }
    }
}

impl<'a> Rule for Seq<'a> {
    fn build_item<'b>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'b>) -> Item<'b> {
        Item {
            combinator: if self.rules.len() == 0 {
                Combinator::Empty
            } else {
                Combinator::Seq(
                    f(&self.rules[0].as_rule_key()),
                    f(&RuleKey::Ref(&WithClass::new(
                        Self::new(&self.rules[1..]),
                        Class::Elide,
                    ))),
                )
            },
            class: Class::Normal,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Choice<'a> {
    rules: &'a [&'a AsRuleKey],
    passthrough: bool,
}

impl<'a> Choice<'a> {
    pub fn new(rules: &'a [&'a AsRuleKey]) -> Self {
        Choice {
            rules: rules,
            passthrough: false,
        }
    }

    pub fn passthrough(&self) -> Self {
        let mut r = *self;
        r.passthrough = true;
        r
    }
}

impl<'a> Rule for Choice<'a> {
    fn build_item<'b>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'b>) -> Item<'b> {
        Item {
            combinator: if self.rules.len() == 0 {
                Combinator::Empty
            } else {
                Combinator::Seq(
                    f(&self.rules[0].as_rule_key()),
                    f(&RuleKey::Ref(&WithClass::new(
                        Self::new(&self.rules[1..]),
                        Class::Elide,
                    ))),
                )
            },
            class: match self.passthrough {
                true => Class::Passthrough,
                false => Class::Normal,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Term {
    val: u8,
}

impl Term {
    pub fn new(val: u8) -> Self {
        Term { val: val }
    }
}

impl Rule for Term {
    fn build_item<'b>(&self, _f: &mut FnMut(&RuleKey) -> ItemRef<'b>) -> Item<'b> {
        Item {
            combinator: Combinator::Term(self.val),
            class: Class::Normal,
        }
    }
}

// FIXME: name
pub struct Grammar<'a> {
    map: HashMap<&'a str, &'a Rule>,
}

impl<'a> Grammar<'a> {
    pub fn new() -> Self {
        Grammar {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, k: &str) -> Option<&'a Rule> {
        self.map.get(k).map(|r| *r)
    }

    pub fn put(&mut self, name: &'a str, v: &'a Rule) -> Option<&'a Rule> {
        self.map.insert(name, v)
    }
}

pub struct ItemSetAllocator<'a> {
    arena: ForwardArena<'a, Item<'a>>,
}

impl<'a> ItemSetAllocator<'a> {
    pub fn new() -> Self {
        Self {
            arena: ForwardArena::new(),
        }
    }
}

pub struct ItemSet<'a> {
    root: &'a Item<'a>,
}

pub struct GrammarBuilder<'b, 'a: 'b, 'g: 'b> {
    arena: &'a ForwardArena<'a, Item<'a>>,
    lookup: HashMap<&'static str, ItemRef<'a>>,
    grammar: &'b Grammar<'g>,
}

fn do_build_grammar<'b, 'a: 'b, 'g: 'b>(
    r: &RuleKey,
    b: &mut GrammarBuilder<'b, 'a, 'g>,
) -> ItemRef<'a> {
    match r {
        RuleKey::Ref(rule) => {
            let cell = b.arena.forward();
            cell.set(rule.build_item(&mut |k| do_build_grammar(k, b)))
        }
        // TODO: return errors if absent
        // map deref to get around borrow checker
        RuleKey::Lookup(k) => match b.lookup.get(k).map(|v| *v) {
            Some(pending) => pending,
            None => {
                let cell = b.arena.forward();
                b.lookup.insert(k, cell.borrow());
                cell.set(
                    b.grammar
                        .get(k)
                        .unwrap()
                        .build_item(&mut |k| do_build_grammar(k, b)),
                )
            }
        },
    }
}

pub fn build_items<'a>(r: &Rule, g: &Grammar, alloc: &'a ItemSetAllocator<'a>) -> ItemSet<'a> {
    ItemSet {
        root: do_build_grammar(
            &RuleKey::Ref(r),
            &mut GrammarBuilder {
                arena: &alloc.arena,
                lookup: HashMap::new(),
                grammar: g,
            },
        ).get()
            .unwrap(),
    }
}
