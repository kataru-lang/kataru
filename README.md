# Kataru 「カタル」 The YAML based dialogue engine.

Kataru 「カタル」is a dialogue engine like [Yarn Spinner](yarnspinner.dev) but based completed on YAML.

```yml
---
# Configure the scene. List your characters, commands, etc.
characters:
  Alice:

---
# Define each of your passages.
Start:
  - Alice walks into the room...
  - Alice: Welcome to my story!
  - Now make a decision...
  - choices:
      continue: Continue
      end: End

Continue:
  - Alice: I see you want to keep reading...
  - Alice: To bad, this is just a demo story!
  - goto: End

End:
  - Thanks for reading!
```

## Features

Each line of a Kataru dialogue script can perform one of many operations, each of which are described in detail in this section.

### Text

Any dialogue line that contains only a YAML string is interpretated as narration text.
YAML supports different ways of multi-lining strings, including the `|` operator for retaining white space and `>-` operator for ignoring white space.

```yaml
---
# No config required for this scene.
---
Passage:
  # A single text block can have many lines using `|`.
  - |
    Welcome to Kataru!
    「カタル」へようこそ!
    Kataru is a dialogue engine built completely on top of YAML with a focus on ease of implementation and simplicity of writing.

  # If you want to ignore whitespace, use `>-`.
  - >-
    Alice walks into the room,
    her footsteps echoing through the halls.

  # Single line text requires no operator.
  - A simple, single line of text.
```

### Dialogue

Dialogue lines are maps of configured character names to their speech text.

```yaml
---
# Define your character(s)
characters:
  Alice:

---
Start:
  - Alice: Welcome to my story!
```

### Set state

To set state in a story, use the `set: {}` command.

### Commands

Commands are delineated by `[]`, and are recognized by the YAML parser as arrays.
Any dialogue lines that are arrays are viewed as list of commands to be executed by the client.

```yaml
# Define commands and their default parameters.
cmds:
  clearScreen: { duration: 0 }
---
# Call them in a passage
Passage:
  - [clearScreen: {}] # Call with default parameters.
  - [clearScreen: { duration: 1 }] # Call with overriden parameters.
  - [clearScreen: {}, clearScreen: {}] # Call multiple commands in sequence.
```

## Understanding the `Bookmark`

Kataru keeps track of your position in a story via a `Bookmark`.
For the simplest stories, this is as simple as keeping track of your current line number.
But nonlinear stories with true agency need to evolve based on the decisions the user made in the past.
Kataru keeps track of the state of the story inside of the `Bookmark` as well.

State can be accessed via if statements:

```yml
---
state:
  Alice.numTalked: 3
---
Passage:
  - if Alice.numTalked > 2:
      - Alice: I'm tired of talking to you!
    else:
      - Alice: Hello there!
```

Or via text variable substitution.

```yml
---
state:
  Alice.numTalked: 3
---
Passage:
  - Hi there, I've already talked to you ${Alice.numTalked} times today.
```

## Namespaces

Documentation pending.

## Examples

See [./examples/simple](./examples/simple) for a minimal example running the engine in the terminal.

## Implementation notes

Notes on how the language parsing, validation and dialogue runner are implemented.

### Dialogue Runner

The `Runner` class only needs a `passage` name and a `line` number to keep its cursor in the story.

Tree-like structures such as `if` and `else` statements are flattened.

In example,

```yaml
Passage:
  - Line 1
  - if x > 2:
      - Line 3
    else:
      - Line 5
  - Line 6
```

Will be flattened to

```yaml
Passage:
  - Line 1
  - if x > 2 jump to Line 3, else Jump to Line 5
  - Line 3
  - Jump to Line 6
  - Line 5
  - Line 6
```
