String foo(String, String qux?, *):
    say $1$2$qux

# Test simple invocation
foo(bar baz) # $1 = bar, $2 is unset, $qux = baz
foo(bar 6 qux) # $1 = bar, $2 = 6, $3 = qux, $qux is unset
foo(qux=baz bar) # $1 = bar, $qux = baz

# Test simple invocation, parenthesis-less style
foo bar baz # $1 = bar, $2 is unset, $qux = baz
foo bar 6 qux # $1 = bar, $2 = 6, $3 = qux, $qux is unset
foo qux=baz bar # $1 = bar, $qux = baz

# Test invocation with "invoke"
routineName="foo"
invoke $routineName bar baz
