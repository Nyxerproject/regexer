# Regexer

- Due at: December 12 at 9:50 AM (due before this actually)

## Todo

- [ ] unable to swap between patern and text
- [ ] add more todo stuff
- [ ] list things needed for the delivery
- [ ] should fail early if stuff isn't a directory
- [ ] figure out how to fail a regex pattern
- [ ] interactive mode should auto select pattern

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
- [livestream of someone writting a regex lib in rust](https://www.youtube.com/watch?v=MH56D5M9xSQ)
- [random regex obfuscator I found](https://github.com/nexxeln/yugen)
- [ratatui examples](https://ratatui.rs/examples/apps/)
- Alternative implementations
  - eager vs lazy DFA
    - There are pros and cons for each
    - No best algorithm exists
      - Bad algorithms do exist
      - Bad algorithm choises do exist
    - how do these compare the to backtracknig and other methods talked about
  - list
    - [Regex standard lib](https://github.com/rust-lang/regex)
      - lazy DFA
      - no codecapture
    - [Regex_automata crate](https://docs.rs/regex-automata/latest/regex_automata/)
      - Thompson NFA
      - allows for codecapture
      - "specifically guarantee worst case O(m \* n) time complexity for all searches"
    - [Regex_dfa crate](https://jneem.github.io/regex-dfa/regex_dfa/index.html)
      - lazy DFA
      - no codecapture
    - [Regex-automata](https://github.com/BurntSushi/regex-automata)
      - lazy DFA
      - no codecapture
      - archived
