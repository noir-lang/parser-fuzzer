use cfg::symbol::SymbolBitSet;
use pest_meta::ast::RuleType;
use proc_macro2::{TokenStream, TokenTree};
use proc_macro2::{Ident, Span};
use pest_meta::optimizer::*;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::{Range, RangeInclusive};

use cfg::prelude::*;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum SymbolKind {
    Nonterminal,
    Null,
    Single(char),
    Range(char, char),
}

#[derive(Clone, Copy)]
struct SymbolWithKind {
    symbol: Symbol,
    kind: SymbolKind,
}
struct NegativeRuleMeta {
    neg: Symbol,
    chars: String,
}

struct Generator {
    grammar: Cfg,
    syms: BTreeMap<String, SymbolWithKind>,
    syms_by_range: HashMap<RangeInclusive<char>, String>,
    negative_rules: Vec<NegativeRuleMeta>,
    // chars: BTreeMap<Terminal, NamedSymbol>,
    // // chars_by_sym: BTreeMap<Symbol, Terminal>,
    // rules: BTreeMap<String, Symbol>,
}

impl Generator {
    fn new() -> Self {
        Self {
            grammar: Cfg::new(),
            syms: BTreeMap::new(),
            syms_by_range: HashMap::new(),
            negative_rules: vec![],
        }
    }

    fn process_rule(&mut self, rule: &OptimizedRule) {
        let lhs = self.intern_ident(rule.name.clone());
        let rhs = self.process_expr(&rule.expr, rule.ty);
        self.grammar.rule(lhs).rhs(rhs);
        // self.grammar.push(Rule { lhs: rule.name.clone(), rhs: self.process_expr(&rule.expr) })
    }

    fn intern_chars(&mut self, range: RangeInclusive<char>) -> Symbol {
        match self.syms_by_range.entry(range.clone()) {
            Entry::Occupied(occupied) => {
                self.syms.get(occupied.get()).unwrap().symbol
            }
            Entry::Vacant(vacant) => {
                let symbol: Symbol = self.grammar.sym();
                let name = format!("s_{}", symbol.usize());
                self.syms.insert(name.clone(), SymbolWithKind { symbol, kind: if range.clone().count() == 1 { SymbolKind::Single(*range.start()) } else { SymbolKind::Range(*range.start(), *range.end()) } });
                vacant.insert(name);
                symbol
            }
        }
    }

    fn intern_ident(&mut self, name: String) -> Symbol {
        self.syms.entry(name).or_insert_with(|| {
            SymbolWithKind { symbol: self.grammar.sym(), kind: SymbolKind::Nonterminal }
        }).symbol
    }

    fn add_rule(&mut self, lhs: Symbol) {
        self.syms.insert(format!("r_{}", lhs.usize()), SymbolWithKind { symbol: lhs, kind: SymbolKind::Nonterminal });
    }

    fn add_negative_sym(&mut self, neg: Symbol) {
        self.syms.insert(format!("neg_{}", neg.usize()), SymbolWithKind { symbol: neg, kind: SymbolKind::Nonterminal });
    }

    fn add_negative_rule(&mut self, neg: Symbol, chars: String) {
        self.negative_rules.push(NegativeRuleMeta { neg, chars });
    }

    fn decl_negative_rules(&self) -> Vec<TokenStream> {
        self.negative_rules.iter().map(|neg_rule| {
            let chars = &neg_rule.chars;
            let name = format!("neg_{}", neg_rule.neg.usize());
            let ident = Ident::new_raw(&name[..], Span::call_site());
            quote! { NegativeRule { sym: #ident, chars: #chars } }
        }).collect()
    }

    fn process_expr(&mut self, expr: &OptimizedExpr, rule_type: RuleType) -> Vec<Symbol> {
        use OptimizedExpr::*;
        match expr {
            // /// Matches an exact string, e.g. `"a"`
            Str(chars) | Insens(chars) => {
                // TODO: Insens
                chars.chars().map(|ch| self.intern_chars(ch..=ch)).collect()
            }

            // /// Matches an exact string, case insensitively (ASCII only), e.g. `^"a"`
            // Insens(String),
            // /// Matches one character in the range, e.g. `'a'..'z'`
            Range(ch_a, ch_b) => {
                assert_eq!(ch_a.chars().count(), 1);
                assert_eq!(ch_b.chars().count(), 1);
                vec![self.intern_chars(ch_a.chars().next().unwrap() ..= ch_b.chars().next().unwrap())]
            }
            // /// Matches the rule with the given name, e.g. `a`
            Ident(name) => {
                vec![self.intern_ident(name.clone())]
            }
            // /// Matches a sequence of two expressions, e.g. `e1 ~ e2`
            Seq(left, right) => {
                let mut result = self.process_expr(left, rule_type);
                match rule_type {
                    RuleType::Atomic | RuleType::CompoundAtomic => {}
                    RuleType::NonAtomic | RuleType::Silent | RuleType::Normal => {
                        result.push(self.intern_ident("WHITESPACE".to_string()));
                    }
                }
                result.extend(self.process_expr(right, rule_type));
                result
            }
            // /// Matches either of two expressions, e.g. `e1 | e2`
            Choice(left, right, _weights) => {
                let lhs = self.grammar.sym();
                let left_rhs = self.process_expr(left, rule_type);
                let right_rhs = self.process_expr(right, rule_type);
                self.grammar.rule(lhs).rhs(left_rhs).rhs(right_rhs);
                self.add_rule(lhs);
                vec![lhs]
            }
            // /// Optionally matches an expression, e.g. `e?`
            Opt(expr) => {
                let lhs = self.grammar.sym();
                let rhs = self.process_expr(expr, rule_type);
                self.grammar.rule(lhs).rhs(rhs).rhs([]);
                self.add_rule(lhs);
                vec![lhs]
            }
            // /// Matches an expression zero or more times, e.g. `e*`
            Rep(expr) => {
                let lhs = self.grammar.sym();
                let rhs = self.process_expr(expr, rule_type);
                let rhs_sym = if rhs.len() > 1 {
                    let lhs = self.grammar.sym();
                    self.grammar.rule(lhs).rhs(rhs);
                    self.add_rule(lhs);
                    lhs
                } else {
                    rhs[0]
                };
                self.grammar.sequence(lhs).inclusive(0, None).rhs(rhs_sym);
                self.add_rule(lhs);
                vec![lhs]
            }
            // /// Matches a custom part of the stack, e.g. `PEEK[..]`
            PeekSlice(..) => panic!(),
            // /// Positive lookahead; matches expression without making progress, e.g. `&e`
            PosPred(expr) => {
                // TODO
                vec![]
            },
            // /// Negative lookahead; matches if expression doesn't match, without making progress, e.g. `!e`
            NegPred(expr) => {
                let neg = self.grammar.sym();
                self.add_negative_sym(neg);
                eprintln!("{:?}", expr);
                match &**expr {
                    Str(chars) => {
                        self.add_negative_rule(neg, chars.clone());
                    }
                    Ident(name) if name == "ASCII_ALPHA" => {
                        for ch in ('0'..='9').chain('a'..='z').chain('A'..='Z') {
                            self.add_negative_rule(neg, ch.to_string());
                        }
                    }
                    _ => panic!("invalid negative lookahead - only strings or ASCII_ALPHA are allowed")
                }
                self.grammar.rule(neg).rhs([]);
                vec![neg]
            },
            // /// Continues to match expressions until one of the strings in the `Vec` is found
            Skip(v) => panic!(),
            // /// Matches an expression and pushes it to the stack, e.g. `push(e)`
            Push(v) => panic!(),
            // /// Matches an expression and assigns a label to it, e.g. #label = exp
            NodeTag(expr, s) => panic!(),
            // /// Restores an expression's checkpoint
            RestoreOnErr(expr) => panic!(),
            // /// Weight.
            Weight => {
                vec![]
            }
        }
    }

    fn rewrite_sequences(&mut self) {
        self.grammar.rewrite_sequences();
    }

    fn update_chars(&mut self) {
        let chars: BTreeSet<Symbol> = self.syms.iter().filter_map(|(_, &sym_with_kind)| if sym_with_kind.kind != SymbolKind::Nonterminal { Some(sym_with_kind.symbol) } else { None }).collect();
        let syms: BTreeMap<Symbol, (String, SymbolWithKind)> = self.syms.iter().map(|(name, sym_with_kind)| (sym_with_kind.symbol, (name.clone(), *sym_with_kind))).collect();
        let bitset = SymbolBitSet::terminal_set(&self.grammar);
        for terminal in bitset.iter() {
            if !chars.contains(&terminal) {
                // let name = rules.get(&terminal).unwrap();
                let name = syms.get(&terminal).map(|r| &r.0[..]).unwrap();
                let gen = match name {
                    "SOI" | "EOI" => SymbolKind::Null,
                    "ASCII_ALPHA" => {
                        SymbolKind::Range('a', 'z')
                    }
                    "ASCII_ALPHANUMERIC" => {
                        SymbolKind::Range('a', 'z')
                    }
                    "ASCII_DIGIT" => {
                        SymbolKind::Range('0', '9')
                    }
                    "ANY" => {
                        SymbolKind::Range('\0', '\x7f')
                    }
                    other => {
                        eprintln!("unknown terminal: {:?}", other);
                        SymbolKind::Null
                    }
                };
                self.syms.insert(name.to_string(), SymbolWithKind { symbol: terminal, kind: gen });
            }
        }
    }

    fn decl_rules(&mut self) -> Vec<TokenStream> {
        let sym_names: BTreeMap<Symbol, String> = self.syms.iter().map(|(name, sym_with_kind)| (sym_with_kind.symbol, name.clone())).collect();
        let mut new_syms = BTreeMap::new();
        let result = self.grammar.rules().map(|rule| {
            let mut get_or_intern = |maybe_name: Option<&String>, sym: Symbol| {
                if let Some(name) = maybe_name {
                    name.clone()
                } else {
                    let name = format!("g_{}", sym.usize());
                    new_syms.insert(name.clone(), SymbolWithKind { symbol: sym, kind: SymbolKind::Nonterminal });
                    name
                }
            };
            let lhs = get_or_intern(sym_names.get(&rule.lhs()), rule.lhs());
            let lhs = Ident::new_raw(&lhs[..], Span::call_site());
            let rhs: Vec<Ident> = rule.rhs().iter().map(|&sym| {
                get_or_intern(sym_names.get(&sym), sym)
            }).map(|name| {
                Ident::new_raw(&name[..], Span::call_site())
            }).collect();
            quote! {
                grammar.rule(#lhs).rhs([#(#rhs),*]);
            }
        }).collect();
        self.syms.append(&mut new_syms);
        result
    }

    fn decl_symbols(&self) -> Vec<TokenStream> {
        self.syms.keys()
            .map(|literal| {
                let name = Ident::new_raw(&literal[..], Span::call_site());
                quote! {
                    let #name: Symbol = grammar.sym();
                    debug!("SYM: {:?} NAME: {:?}", #name, #literal);
                }
            })
            .collect()
    }

    fn match_start(&self) -> Vec<TokenStream> {
        self.syms.keys()
            .map(|name| {
                let sym = Ident::new_raw(&name[..], Span::call_site());
                quote! {
                    #name => #sym,
                }
            })
            .collect()
    }

    fn stmt_char_from_sym(&self) -> Vec<TokenStream> {
        self.syms.iter()
            .filter_map(|(name, sym_with_kind)| {
                let ch = match sym_with_kind.kind {
                    SymbolKind::Single(ch) => quote! { Some(#ch) },
                    SymbolKind::Range(start, end) => quote! { {
                        let start = #start as u32;
                        let end = #end as u32;
                        let offset = byte_source.gen(end as f64 - start as f64);
                        let result = char::from_u32(start + offset as u32).expect("incorrect char");
                        Some(result)
                    } },
                    SymbolKind::Null => quote! { None },
                    SymbolKind::Nonterminal => return None,
                };
                let name = Ident::new_raw(&name[..], Span::call_site());
                Some(quote! { if sym == #name { return #ch } })
            })
            .collect()
    }
}

pub fn generate_cfg_generator(rules: &[OptimizedRule]) -> TokenStream {
    let mut generator = Generator::new();
    for rule in rules {
        generator.process_rule(rule);
    }
    generator.rewrite_sequences();
    generator.update_chars();
    let stmt_char_from_sym = generator.stmt_char_from_sym();
    let decl_rules = generator.decl_rules();
    let decl_symbols = generator.decl_symbols();
    let match_start = generator.match_start();
    let decl_negative_rules = generator.decl_negative_rules();
    let result = quote! {
        fn generate(start_sym: &str, driver: &[u8], limit: Option<u64>) -> Result<String, ()> {
            use pest::cfg::prelude::*;
            use pest::cfg::generation::weighted::{Random, NegativeRule};
            use pest::cfg::generation::weighted::random::ByteSource;
            use pest::cfg::generation::weighted::random::GenRange;
            use pest::env_logger::try_init;
            use pest::log::debug;
            let _ = try_init();
            let mut grammar = Cfg::new();
            #(#decl_symbols)*
            #(#decl_rules)*
            let mut binarized = grammar.binarize();
            let start_sym = match start_sym {
                #(#match_start)*
                _ => panic!("incorrect start_sym provided"),
            };
            let negative_rules = vec![#(#decl_negative_rules),*];
            let mut byte_source = ByteSource::new(driver.iter().cloned());
            let to_stmt_char_with_byte_source = |sym, byte_source: &mut ByteSource<_>| {
                #(#stmt_char_from_sym)*
                return Some('X');
            };
            let (result, string) = binarized.random(start_sym, limit, &mut byte_source, &negative_rules[..], to_stmt_char_with_byte_source).map_err(|_| ())?;
            Ok(string.into_iter().collect())
        }
    };
    eprintln!("GENERATE: {}", result);
    result
}