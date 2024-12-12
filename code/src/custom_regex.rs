#[derive(Debug, Clone, PartialEq, Eq)]
// disclosure; I used these sites as reference while making this: https://jneem.github.io/regex-dfa/src/regex_dfa/src/regex.rs.html?search=
enum RegexAST {
    Empty,                                // ε
    Literal(char),                        // single character
    Concat(Box<RegexAST>, Box<RegexAST>), // AB
    Union(Box<RegexAST>, Box<RegexAST>),  // A|B
    Kleene(Box<RegexAST>),                // A*
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).cloned()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn parse(&mut self) -> RegexAST {
        self.parse_union()
    }

    fn parse_union(&mut self) -> RegexAST {
        let mut node = self.parse_concat();
        while let Some('|') = self.peek() {
            self.next();
            let right = self.parse_concat();
            node = RegexAST::Union(Box::new(node), Box::new(right));
        }
        node
    }

    fn parse_concat(&mut self) -> RegexAST {
        let mut node = self.parse_kleene();
        while let Some(c) = self.peek() {
            if c == ')' || c == '|' {
                break;
            }
            let right = self.parse_kleene();
            node = RegexAST::Concat(Box::new(node), Box::new(right));
        }
        node
    }

    fn parse_kleene(&mut self) -> RegexAST {
        let mut node = self.parse_base();
        while let Some('*') = self.peek() {
            self.next();
            node = RegexAST::Kleene(Box::new(node));
        }
        node
    }

    fn parse_base(&mut self) -> RegexAST {
        match self.peek() {
            Some('(') => {
                self.next();
                let node = self.parse_union();
                if self.next() != Some(')') {
                    panic!("Unmatched parenthesis");
                }
                node
            }
            Some(c) if c.is_ascii_lowercase() => {
                self.next();
                RegexAST::Literal(c)
            }
            None => RegexAST::Empty,
            _ => panic!("Unexpected character"),
        }
    }
}

// ----- NFA Construction using Thompson's Construction -----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
enum NFASymbol {
    Char(char),
    Epsilon,
}

#[derive(Debug, Clone)]
struct NFA {
    start: State,
    accept: State,
    transitions: Vec<(State, NFASymbol, State)>,
    state_count: usize,
}

impl NFA {
    fn new_empty() -> Self {
        Self {
            start: State(0),
            accept: State(1),
            transitions: Vec::new(),
            state_count: 2,
        }
    }

    fn new_literal(c: char) -> Self {
        let mut nfa = NFA::new_empty();
        // start --c--> accept
        nfa.transitions
            .push((nfa.start, NFASymbol::Char(c), nfa.accept));
        nfa
    }

    fn new_concat(a: NFA, b: NFA) -> Self {
        // Merge by connecting a.accept -> b.start (ε-transition)
        let mut nfa = NFA {
            start: a.start,
            accept: b.accept,
            transitions: a.transitions,
            state_count: a.state_count.max(b.state_count),
        };

        let offset = nfa.state_count;
        let remapped = b
            .transitions
            .into_iter()
            .map(|(s, sym, t)| (State(s.0 + offset), sym, State(t.0 + offset)))
            .collect::<Vec<_>>();

        nfa.transitions.extend(remapped);
        nfa.transitions
            .push((a.accept, NFASymbol::Epsilon, State(b.start.0 + offset)));
        nfa.state_count = offset + b.state_count;
        nfa.accept = State(b.accept.0 + offset);
        nfa
    }

    fn new_union(a: NFA, b: NFA) -> Self {
        let mut nfa = NFA {
            start: State(0),
            accept: State(1),
            transitions: Vec::new(),
            state_count: 2,
        };

        let a_offset = nfa.state_count;
        nfa.state_count += a.state_count;
        for (s, sym, t) in a.transitions {
            nfa.transitions
                .push((State(s.0 + a_offset), sym, State(t.0 + a_offset)));
        }

        let b_offset = nfa.state_count;
        nfa.state_count += b.state_count;
        for (s, sym, t) in b.transitions {
            nfa.transitions
                .push((State(s.0 + b_offset), sym, State(t.0 + b_offset)));
        }

        // ε from new start to a.start+a_offset and b.start+b_offset
        nfa.transitions
            .push((nfa.start, NFASymbol::Epsilon, State(a.start.0 + a_offset)));
        nfa.transitions
            .push((nfa.start, NFASymbol::Epsilon, State(b.start.0 + b_offset)));

        // ε from a.accept+a_offset and b.accept+b_offset to new accept
        nfa.transitions
            .push((State(a.accept.0 + a_offset), NFASymbol::Epsilon, nfa.accept));
        nfa.transitions
            .push((State(b.accept.0 + b_offset), NFASymbol::Epsilon, nfa.accept));

        nfa
    }

    fn new_kleene(a: NFA) -> Self {
        let mut nfa = NFA {
            start: State(0),
            accept: State(1),
            transitions: Vec::new(),
            state_count: 2,
        };
        let offset = nfa.state_count;
        nfa.state_count += a.state_count;

        for (s, sym, t) in a.transitions {
            nfa.transitions
                .push((State(s.0 + offset), sym, State(t.0 + offset)));
        }

        // ε from new start to a.start+offset and to new accept
        nfa.transitions
            .push((nfa.start, NFASymbol::Epsilon, State(a.start.0 + offset)));
        nfa.transitions
            .push((nfa.start, NFASymbol::Epsilon, nfa.accept));
        // ε from a.accept+offset to new accept and back to a.start+offset
        nfa.transitions
            .push((State(a.accept.0 + offset), NFASymbol::Epsilon, nfa.accept));
        nfa.transitions.push((
            State(a.accept.0 + offset),
            NFASymbol::Epsilon,
            State(a.start.0 + offset),
        ));

        nfa
    }

    fn from_ast(ast: &RegexAST) -> Self {
        match ast {
            RegexAST::Empty => NFA::new_empty(),
            RegexAST::Literal(c) => NFA::new_literal(*c),
            RegexAST::Concat(a, b) => NFA::new_concat(NFA::from_ast(a), NFA::from_ast(b)),
            RegexAST::Union(a, b) => NFA::new_union(NFA::from_ast(a), NFA::from_ast(b)),
            RegexAST::Kleene(a) => NFA::new_kleene(NFA::from_ast(a)),
        }
    }
}

// ----- Subset construction (NFA -> DFA) -----

use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DFAState {
    nfa_states: BTreeSet<usize>,
}

#[derive(Debug, Clone)]
struct DFA {
    start: DFAState,
    accept_states: Vec<DFAState>,
    transitions: HashMap<(DFAState, char), DFAState>,
}

impl DFA {
    fn from_nfa(nfa: &NFA) -> Self {
        let start_closure = epsilon_closure(&nfa.transitions, nfa.start.0);
        let start_state = DFAState {
            nfa_states: start_closure,
        };

        let mut dfa = DFA {
            start: start_state.clone(),
            accept_states: Vec::new(),
            transitions: HashMap::new(),
        };

        let mut worklist = vec![start_state.clone()];
        let mut visited = BTreeSet::new();
        visited.insert(start_state.clone());

        while let Some(current) = worklist.pop() {
            if current.nfa_states.contains(&nfa.accept.0) {
                dfa.accept_states.push(current.clone());
            }

            let mut char_map: HashMap<char, BTreeSet<usize>> = HashMap::new();
            for &s in &current.nfa_states {
                for (src, sym, dst) in &nfa.transitions {
                    if src.0 == s {
                        if let NFASymbol::Char(c) = sym {
                            char_map.entry(*c).or_default().insert(dst.0);
                        }
                    }
                }
            }

            for (c, set) in char_map {
                let mut new_set = BTreeSet::new();
                for st in set {
                    let cl = epsilon_closure(&nfa.transitions, st);
                    new_set.extend(cl);
                }
                let new_state = DFAState {
                    nfa_states: new_set,
                };
                if !visited.contains(&new_state) {
                    visited.insert(new_state.clone());
                    worklist.push(new_state.clone());
                }
                dfa.transitions.insert((current.clone(), c), new_state);
            }
        }

        dfa
    }

    fn matches(&self, input: &str) -> bool {
        let mut current = self.start.clone();
        for c in input.chars() {
            if let Some(next) = self.transitions.get(&(current.clone(), c)) {
                current = next.clone();
            } else {
                return false;
            }
        }
        self.accept_states.contains(&current)
    }
}

fn epsilon_closure(transitions: &[(State, NFASymbol, State)], start: usize) -> BTreeSet<usize> {
    let mut stack = vec![start];
    let mut closure = BTreeSet::new();
    closure.insert(start);
    while let Some(s) = stack.pop() {
        for (src, sym, dst) in transitions {
            if src.0 == s && *sym == NFASymbol::Epsilon && !closure.contains(&dst.0) {
                closure.insert(dst.0);
                stack.push(dst.0);
            }
        }
    }
    closure
}

fn regex_to_dfa(pattern: &str) -> DFA {
    let mut parser = Parser::new(pattern);
    let ast = parser.parse();
    let nfa = NFA::from_ast(&ast);
    DFA::from_nfa(&nfa)
}

// ----- CustomRegex definition -----

pub struct CustomRegex {
    dfa: DFA,
}

#[derive(Debug)]
pub struct RegexError(String);

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegexError: {}", self.0)
    }
}

impl std::error::Error for RegexError {}

impl CustomRegex {
    pub fn new(pattern: &str) -> Result<CustomRegex, RegexError> {
        if pattern.is_empty() {
            return Err(RegexError("Empty pattern".to_string()));
        }
        let dfa = regex_to_dfa(pattern);
        Ok(CustomRegex { dfa })
    }

    pub fn find_iter<'a>(&'a self, text: &'a str) -> Vec<&'a str> {
        // Naive substring search:
        let mut results = Vec::new();
        for start in 0..text.len() {
            for end in start..=text.len() {
                let substring = &text[start..end];
                if self.dfa.matches(substring) {
                    results.push(substring);
                }
            }
        }
        results
    }
}
