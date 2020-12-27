# Kataru 「カタル」 The YAML based dialogue engine.

Kataru 「カタル」is a dialogue engine like [Yarn Spinner](yarnspinner.dev) but based completed on YAML.

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
