# oxc_query

## What is this?

`oxc_query` is a [trustfall](https://github.com/obi1kenobi/trustfall) [adapter](https://docs.rs/trustfall_core/0.6.0/trustfall_core/interpreter/trait.Adapter.html) which can be used for querying data about code.

Example: Query the name of every variable declared in a js/ts/tsx file

```graphql
query {
    File {
        variable_declaration {
            left {
                assignment_to_variable_name @filter(op: "is_not_null") @output # we filter not null because destructured variables will be null when outputted on this edge.
            }
        }
    }
}
```

Hint: See example/simple.rs for how to run this query.
