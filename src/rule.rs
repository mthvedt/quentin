//! Rules in a context-free grammar.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use util::arena::{ForwardArena, ForwardRef};

pub type ItemRef<'a> = ForwardRef<'a, Item<'a>>;

impl<'a> ItemRef<'a> {
    fn do_write(&self, f: &mut fmt::Formatter, joiner: &str) -> fmt::Result {
        match self.get() {
            Some(item) => write!(f, "{}{}", joiner, item),
            None => write!(f, "<unbuilt>"),
        }
    }
}

pub trait Rule {
    fn build_item<'a>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'a>) -> Item<'a>;
}

/// It's actually unsound in Rust to use trait references as keys!
/// The reason references are partial keys is that
/// a single struct might have two different trait implementations in scope in two different places,
/// so the Trait pointer is not guaranteed to point to the same vtable.
/// Even more generally, Rust may (and does) at its option
/// generate two different vtables for the same implementation.
#[derive(Clone)]
pub enum RuleKey<'a> {
    Ref(&'a Rule),
    // This is String for now to avoid more lifetimes everywhere.
    // Will generalize this key wholesale later.
    Lookup(Rc<String>),
}

pub trait AsRuleKey {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a>;
}

impl<R: Rule> AsRuleKey for R {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a> {
        RuleKey::Ref(self)
    }
}

impl AsRuleKey for String {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a> {
        RuleKey::Lookup(Rc::new(self.clone()))
    }
}

impl<'x> AsRuleKey for &'x str {
    fn as_rule_key<'a>(&'a self) -> RuleKey<'a> {
        RuleKey::Lookup(Rc::new(String::from(*self)))
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

impl<'a> fmt::Display for Combinator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Combinator::*;

        match self {
            Seq(a, b) => {
                a.do_write(f, &"")?;
                b.do_write(f, &" ")
            }
            Choice(a, b) => {
                a.do_write(f, &"")?;
                b.do_write(f, &"|")
            }
            Term(v) => write!(f, "({})", v),
            Done => write!(f, "i"),
            Empty => write!(f, "e"),
        }
    }
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
    name: String,
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Class::*;

        match self.class {
            Normal => write!(f, "[{}]", self.name),
            Passthrough => self.combinator.fmt(f),
            Elide => self.combinator.fmt(f),
        }
    }
}

struct Elide<R: Rule>(R);

impl<R: Rule> Rule for Elide<R> {
    fn build_item<'a>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'a>) -> Item<'a> {
        let item = self.0.build_item(f);
        Item {
            combinator: item.combinator,
            class: Class::Elide,
            name: String::from("(elided)"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Seq<'a, Name: 'a> {
    name: Name,
    rules: &'a [&'a AsRuleKey],
}

impl<'a, Name: 'a + Borrow<str> + Clone> Seq<'a, Name> {
    pub fn new(name: Name, rules: &'a [&'a AsRuleKey]) -> Self {
        Seq {
            name: name,
            rules: rules,
        }
    }
}

impl<'a, Name: 'a + Borrow<str> + Clone> Rule for Seq<'a, Name> {
    fn build_item<'b>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'b>) -> Item<'b> {
        Item {
            combinator: if self.rules.len() == 0 {
                Combinator::Empty
            } else {
                Combinator::Seq(
                    f(&self.rules[0].as_rule_key()),
                    f(&RuleKey::Ref(&Elide(Seq::new(
                        "(elided)",
                        &self.rules[1..],
                    )))),
                )
            },
            class: Class::Normal,
            name: String::from(self.name.borrow()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Choice<'a, Name: 'a> {
    name: Name,
    rules: &'a [&'a AsRuleKey],
    passthrough: bool,
}

impl<'a, Name: 'a + Borrow<str> + Clone> Choice<'a, Name> {
    pub fn new(name: Name, rules: &'a [&'a AsRuleKey]) -> Self {
        Choice {
            name: name,
            rules: rules,
            passthrough: false,
        }
    }

    pub fn passthrough(&self) -> Self {
        let mut r = self.clone();
        r.passthrough = true;
        r
    }
}

impl<'a, Name: 'a + Borrow<str> + Clone> Rule for Choice<'a, Name> {
    fn build_item<'b>(&self, f: &mut FnMut(&RuleKey) -> ItemRef<'b>) -> Item<'b> {
        Item {
            combinator: if self.rules.len() == 0 {
                Combinator::Empty
            } else {
                Combinator::Seq(
                    f(&self.rules[0].as_rule_key()),
                    f(&RuleKey::Ref(&Elide(Choice::new(
                        "(elided)",
                        &self.rules[1..],
                    )))),
                )
            },
            class: match self.passthrough {
                true => Class::Passthrough,
                false => Class::Normal,
            },
            name: String::from(self.name.borrow()),
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
            name: String::from_utf8([self.val].to_vec()).unwrap(),
        }
    }
}

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

impl<'a> fmt::Display for ItemSet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.root.fmt(f)
    }
}

pub struct GrammarBuilder<'b, 'a: 'b, 'g: 'b> {
    arena: &'a ForwardArena<'a, Item<'a>>,
    lookup: HashMap<Rc<String>, ItemRef<'a>>,
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
                b.lookup.insert(k.clone(), cell.borrow());
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
