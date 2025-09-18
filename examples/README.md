Examples from Cassia:
- `imp_example.json`

All other examples are from Justin.

Useful jq scripts:

```
jq '.graph.nodes |= with_entries(.value.metadata.data = .value.data | .value |= del(.data))'
```

```
jq '.graph.edges |= map({source: .source, target: .target})'
```
