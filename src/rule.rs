//! Rules in a context-free grammar.

use std::cell::UnsafeCell;
use std::collections::HashMap;

use util::refs::{ShallowRef, DeepRef};

use typed_arena::Arena;

pub trait Rule {
    fn build_item(&self, g: &'static mut Graph) -> Item;
}

type RuleRef = ShallowRef<'static, Rule>;

// TODO: can we hide this? trouble is Rule needs to know. We might not want to hide it
// (instead hide through cfg module).
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Combinator {
    Seq(ItemThunk, ItemThunk),
    Choice(Vec<ItemThunk>),
    Term(i8),
    Done,
    Empty,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Item {
    combinator: Combinator,
    // line: String,
    passthrough: bool,
}

type ItemRef = DeepRef<'static, Item>;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ItemThunk {
    Real(ItemRef),
    Thunk(RuleRef),
}

/*
Graph exhibits interior mutability.
The reason for this is Graph must outlive the lifetimes of the graph's interior references,
which makes it difficult to write functions that may take a &mut Graph
and cause new such references to exist.
*/
pub struct Graph {
    inner: UnsafeCell<GraphInner>,
}

pub struct GraphInner {
    rule_to_thunk: HashMap<RuleRef, ItemThunk>,
    item_alloc: Arena<Item>,
    // TODO: we store items twice here.
    // We can store them once if we're more creative with lifetimes,
    // which we eventually will need to be.
    // The problem is we need ItemRef to be contravariant in the lifetimes of its referents,
    // so we can cast a short-lived ItemRef to an ItemRef<'static>.
    // I think that's the problem anyway...
    // Anyawy, these clones are expensive if a Choice is involved.
    item_dedup: HashMap<Item, ItemRef>,
}

impl Graph {
    // fn new() -> Self {
    //     Graph {
    //         inner: UnsafeCell::new(GraphInner {
    //             rule_to_thunk: HashMap::new(),
    //             item_alloc: Arena::new(),
    //             item_dedup: HashMap::new(),
    //         })
    //     }
    // }

    fn doto<'a, T, F: FnOnce(&'a mut GraphInner) -> T>(&'a self, f: F) -> T {
        let ptr = self.inner.get();
        let intref = unsafe { &mut *ptr };
        f(intref)
    }

    fn item_thunk(&self, rule: &'static Rule) -> ItemThunk {
        self.doto(|inner| {
            let rref = RuleRef::new(rule);

            match inner.rule_to_thunk.get(&rref).map(|v| *v) {
                Some(r) => r,
                None => {
                    let t = ItemThunk::Thunk(rref);
                    inner.rule_to_thunk.insert(rref, t);
                    t
                },
            }
        })
    }

    fn add_item_ref(&'static self, item: Item) -> ItemRef {
        self.doto(|inner| {
            match inner.item_dedup.get(&item).map(|v| *v) {
                Some(i) => i,
                None => {
                    let raw_iref = inner.item_alloc.alloc(item.clone());
                    let iref = ItemRef::new(raw_iref);
                    inner.item_dedup.insert(item, iref);
                    iref
                }
            }
        })
    }

    fn add_item(&'static self, item: Item) -> ItemThunk {
        ItemThunk::Real(self.add_item_ref(item))
    }
}

pub struct Seq {
    // Seqs are never passthrough
    rules: &'static [&'static Rule],
}

impl Seq {
    fn build_item_from_rules(rules: &'static [&'static Rule], g: &'static Graph) -> Item {
        Item {
            combinator: if rules.len() == 0 {
                Combinator::Empty
            } else {
                let i0ref = g.item_thunk(rules[0]);
                let i1 = Seq::build_item_from_rules(&rules[1..], g);
                let i1ref = g.add_item(i1);

                Combinator::Seq(i0ref, i1ref)
            },
            passthrough: false,
        }
    }
}

impl Rule for Seq {
    fn build_item(&self, g: &'static mut Graph) -> Item {
        Seq::build_item_from_rules(self.rules, g)
    }
}

pub struct Choice {
    rules: &'static [&'static Rule],
    passthrough: bool,
}

impl Rule for Choice {
    fn build_item(&self, g: &'static mut Graph) -> Item {
        Item {
            combinator: Combinator::Choice(
                self.rules.iter().map(|rref| g.item_thunk(*rref)).collect()),
            passthrough: self.passthrough,
        }
    }
}

pub struct Term {
    val: i8,
    // Terms are never passthrough
}

impl Rule for Term {
    fn build_item(&self, _g: &'static mut Graph) -> Item {
        Item {
            combinator: Combinator::Term(self.val),
            passthrough: false,
        }
    }
}
