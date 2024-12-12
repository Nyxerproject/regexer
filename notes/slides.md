---
title: Deterministic Finite Automata, Rust, and RegEx
author: Isaac M.
---

## Greedy DFA RegEx Implementation in Rust

### Algorithm Choice

For this project, I decided to begin work on implementing a RegEx engine and parser for Rust.

<!-- end_slide -->

## Greedy DFA RegEx Implementation in Rust

### Application Choice

The application is a CLI and TUI for listing, adding, and evaluating expressions.

<!-- end_slide -->

## Greedy DFA RegEx Implementation in Rust

### Language Choice

This was implemented in Rust.

<!-- end_slide -->

## Program Usage or README

```
regexer is a command-line/text-user interface
tool for parsing and testing regular expressions.

...
Use --engine to select the regex engine: (some are wip and are not implemented)
  - builtin
  - custom
  - dfa
  - hybrid
  - onepass
  - boundedbacktracker
  - pikevm
  - meta
  - custommeta (tries CustomRegex first, verify with builtin,
  fallback to builtin on error)

Usage: regexer [OPTIONS] [pattern] [text]

Arguments:
  [pattern]
          The regular expression pattern to match

  [text]
          The text to search within (use -f to read from a file)

Options:
  -i, --interactive
          Launch the interactive TUI mode

  -f, --file <FILE>
          Read text from a file instead of standard input

  -o, --output <FILE>
          Write the output to a file instead of standard output

      --engine <engine>
          Select the regex engine to use: builtin, custom, dfa,
          hybrid, onepass, boundedbacktracker, pikevm, meta, custommeta

          [default: builtin]
          [possible values: builtin, custom, dfa, hybrid, onepass,
          boundedbacktracker, pikevm, meta, custommeta]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

<!-- end_slide -->

## Where It Is Used

This is intended to be used in an educational setting and for seeing how regex works.

<!-- end_slide -->

## Other Applications

This could be used as a CLI utility for a server. I wouldn't recommend it.
This could be used as a TUI for finding general regex evaluations.

## Alternative Algorithms

Other methods include Eager DFA, PFA, ReMatch, and Re#. All of these are usable alternatives.

<!-- end_slide -->

## Reason for Choice

I decided to pick DFA due to it being the most common modern implementation and I thought it would be the easiest. Many implementations have trade-offs. This one takes a longer time to execute and compile. Other implementations offer trade-offs, such as PFA, or ReMatch, which is more powerful than the common regular expressions.

<!-- end_slide -->

## How Your Algorithm/Program Works

State machines

<!-- end_slide -->

## Run Time

My program has the possibility of O(m + n) run time when using the standard library. When not, my algorithm may fail ahead of time or fail to implement RegEx fully, exiting early.

[A fully labeled runtime graph for varying n with a minimum of 10 points. If solo, one variable may be varied. If team, two graphs with at least 5 points each are required.]

(Team only: Formally prove the run time using instruction counting, probability, or recursion analysis. Show pseudocode.)

<!-- end_slide -->

## Thanks for watching

<!-- end_slide -->
