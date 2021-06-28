# Namespaces

Kataru namespaces are a way of organizing your passages into sections such that two sections can use the same passage or character names.

In many stories, different sections or chapters will contain similar passasges in their structure, such as a `Start`, an `End`, etc.
Instead of prefixing your passage names `Chapter1Start`, you can use a namespace `Chapter1` and define a passage`Start`.
This passage can be referenced within the `Chapter` namespace as `Start`, and outside of `Chapter1` as `Chapter1:Start`.
The `:` is a special character that delimits namespaces from identifiers.

Configs such as `characters`, `commands`, `onEnter`, and `onExit` are all shared within the namespace.

## Nested namespaces

All namespaces implicitly belong to the `global` namespace, so any configs defined in `global` namespace are accessible from all other namespaces.
If you want to define shared behavior over a group of namespaces but not every namespace in your story, you can define nested namespaces.

In example, imagine your story has an `Act1` that has some characters and commands you want to refer to only during this Act.
But `Act1` is much too large to fit into a singular namespace, as there may be many chapters inside of the act.
To handle this case, you can define your chapter namespaces as `Act1:Chapter1`. `Act1:Chapter2`, etc.
These namespaces will inherit configs from `Act1` as well as `global` namespace.

What if you have the same identifier in `Act1` and `global` namespace, i.e. a passage called `SharedPassage`?
If we write `- call: SharedPassage` in `Act1:Chapter1`, which `SharedPassage` will be called (assuming `SharedPassage` is not defined in `Act1:Chapter1`)?
The order of resolution is as follows:

1. Search in the current namespace.
   `SharedPassage` is not defined in `Act1:Chapter1`, so we check the next namespace up.
2. Search the parent namespace(s) in order.
   `SharedPassage` _is_ defined in `Act1`, so this name resolves to `Act1:SharedPassage`.
3. Search the `global` namespace.
   We already found `SharedPassage`, so no need to check `global`.
