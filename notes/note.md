# Regexer

- Due at: December 12 at 9:50 AM (due before this actually)

## Todo

- [x] unable to swap between patern and text
- [ ] list things needed for the delivery
- [ ] should fail early if -f isn't a directory
- [ ] should fail early if pattern is a directory
- [ ] by default, custom_meta_parser should be the default
  - it should use my custom method first and veryify it is valid.
    - if custoRegex has an error output the error to stderr 
    - when using meta, customRegex should always be compared against another regex engine
    - when not using meta, customRegex shouldn't be compared against another regex engine
    - [ ] figure out how to fail a regex pattern
- [ ] interactive mode should auto select pattern
- [ ] parse all of the file and go line by line
  - [ ] this is related to a bug where, when started with a file and in interactive mode and with a pattern, it outputs weirdly. needs more testing
- [ ] add testing to be done by the compiler/toolchain. should be easy >:3
- [ ] when doing regexer pattern -f file -i, the file doesn't get added. to the expressions table. instead, it only gets parsed once and only once when the pattern is first edited
- [ ] use silicon to make pretty code
- [ ] -f files should be parsed as a list of entries seperated by newlines
- [ ] implement very ez and baby basic regex handeler for "homebrew" for engins
- [ ] add modified std_lib to engins
- [ ] can i make different DFA algos work togeather?
- [ ] can I use DFA from regex-automita
  - [ ] it should implemnet a paper that makes regex go fasterintroduction
- [ ] create custom meta regex
  - it should try to use my own regex stuff first
  - if it fails, try the next one.
  - It will eventually get to meta::regex and that will for sure work
- [ ] add restrictions
  - only ascii/UTF-8

### Larger tasks

- Program Algorithm
- Make CLI/TUI
- Make Slides
- Make video

### Backlog

- Implement plotters crate for graphing times

## Requirements

- Need 2 to 3.5 min video

## Resources

- add another DFA https://arxiv.org/pdf/2403.16533
-  https://arxiv.org/pdf/2407.20479 
  - 5.1.1 and 5.1.2
- https://jneem.github.io/regex-dfa/src/regex_dfa/src/regex.rs.html#18-20
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
