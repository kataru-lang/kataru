# Kataru 「カタル」 The YAML based dialogue engine.

Kataru 「カタル」is a dialogue engine like [Yarn Spinner](yarnspinner.dev) but based completed on YAML.

## Examples

See [./examples/simple](./examples/simple) for a minimal example running the engine in the terminal.

## Implementation notes

Notes on how the language parsing, validation and dialogue runner are implemented.

### Dialogue Runner

The `Runner` class only needs a `passage` name and a `line` number to keep its cursor in the story.

Tree-like structures such as `if` and `else` statements are flattened.
