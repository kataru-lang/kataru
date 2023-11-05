# Parser

Kataru parser needs to:

1. Produce (Span, SemanticTokenType) pairs for syntax highlighting (quickly).
1. Produce a list of all Passages, Variables, Characters, etc. in the current namespace.
1. Create a zero-copy AST that can be transformed into the parsed Story used by the Runner.
