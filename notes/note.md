# REGEXER

- Due at:

## Todo

### Larger tasks

- Program Algorithm
- Make CLI
- Make Slides
- Make video

### Backlog

- Implement plotters crate for graphing times

## Requirements

- Need 2 to 3.5 min video

## Resources

- [Initial nerd snip video (what got me to consider this)](https://www.0de5.net/stimuli/a-reintroduction-to-programming/instructions-to-languages/how-regexes-got-catastrophic)
- Alternative implementations
  - eager vs lazy DFA
    - There are pros and cons for each
    - No best algorithm exists
      - Bad algorithms do exist
      - Bad algorithm choises do exist
  - list
    - [Regex_automata crate](https://docs.rs/regex-automata/latest/regex_automata/)
      - Thompson NFA
      - allows for codecapture
      - "specifically guarantee worst case O(m \* n) time complexity for all searches"
    - [Regex_dfa crate](https://jneem.github.io/regex-dfa/regex_dfa/index.html)
      - lazy DFA
    - [Regex-automata](https://github.com/BurntSushi/regex-automata)
      - lazy DFA
      - archived
