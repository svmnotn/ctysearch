# C-Type-Search (ctysearch)

Search for a C function definition in a given header file by return/parameter
types, using fuzzy matching. Useful when you remember a signature shape
(e.g. "returns a `Test_t*` and takes a `Test_t*`") but not the exact name.

## Installation

```sh
cargo install --path .
# or: cargo build --release
```

## Usage

```
Usage: ctysearch [-h|--help] [-l|--limit=NUM] [-f|--header=C_HEADER_FILE] FUNCTION

--help: Prints this message

--limit: Limit the number of results to return - Default: 10
--header: The C Header file to search through
```

- `FUNCTION` is a type signature to search for, written as
  `RETURN_TYPE (PARAM1, PARAM2, ...)` — names don't matter, only types.
  Examples: `"void ()"`, `"uint8_t* (uint8_t)"`,
  `"struct Test_t* (struct Test_t, uint8_t*)"`.
- Results are ranked by Levenshtein edit distance between your query and each
  function's canonical signature, closest first.
- Each result line looks like
  `FILE:ROW:COL: NAME :: SIGNATURE`, e.g.
  `example.h:5:1: test1 :: void (uint8_t, uint8_t*)`
  (rows/columns are 1-based).

## How matching works

1. The header is parsed with [tree-sitter-c](https://github.com/tree-sitter/tree-sitter-c).
2. Plain declarations, function-pointer `typedef`s, and struct-member function
   pointers are all collected.
3. Each function is canonicalized to `RETURN (ARGS)` — e.g. `test1` becomes
   `void (uint8_t, uint8_t*)`.
4. Candidates are sorted by `edit_distance(canonical, query)` and the first
   `--limit` (default 10) are printed.

Because matching is fuzzy, an approximate query still finds the right
function (see Example 5).

## Examples

All examples below use the repo's [`example.h`](example.h), which contains
plain functions (`test`…`test3`, `test10`, `test13`…`test15`), function-pointer
typedefs (`test4_t`…`test7_t`, `test11_t`), and struct members
(`test8`, `test9`, `test12`).

### 1. Find nullary `void` functions

```sh
ctysearch -f example.h "void ()"
```

```text
example.h:10:1: test14 :: void ()
example.h:11:1: test15 :: void (void)
example.h:20:5: test8 :: void (Test_t*, void*)
example.h:21:5: test9 :: Test_t* (Test_t*)
example.h:4:1: test :: uint8_t* (uint8_t)
example.h:13:1: test4_t :: uint8_t* (uint8_t)
example.h:5:1: test1 :: void (uint8_t, uint8_t*)
example.h:14:1: test5_t :: void (uint8_t, uint8_t*)
example.h:6:1: test2 :: Test_t (uint8_t, uint8_t*)
example.h:7:1: test3 :: Test_t* (Test_t, uint8_t*)
```

`test14` (`void test14();`) is the exact match and ranks first.

### 2. Find by exact signature

```sh
ctysearch -f example.h "uint8_t* (uint8_t)"
```

```text
example.h:4:1: test :: uint8_t* (uint8_t)
example.h:13:1: test4_t :: uint8_t* (uint8_t)
example.h:5:1: test1 :: void (uint8_t, uint8_t*)
example.h:14:1: test5_t :: void (uint8_t, uint8_t*)
example.h:21:5: test9 :: Test_t* (Test_t*)
example.h:6:1: test2 :: Test_t (uint8_t, uint8_t*)
example.h:15:1: test6_t :: Test_t (uint8_t, uint8_t*)
example.h:7:1: test3 :: Test_t* (Test_t, uint8_t*)
example.h:16:1: test7_t :: Test_t* (Test_t, uint8_t*)
example.h:10:1: test14 :: void ()
```

Both the plain function `test` and the typedef `test4_t` share this
signature and rank at the top.

### 3. Limit the number of results

```sh
ctysearch -f example.h -l 3 "Test_t* (Test_t*)"
```

```text
example.h:21:5: test9 :: Test_t* (Test_t*)
example.h:4:1: test :: uint8_t* (uint8_t)
example.h:7:1: test3 :: Test_t* (Test_t, uint8_t*)
```

`-l`/`--limit` caps output (default 10). `-l 3` prints exactly 3 lines.

### 4. Search for `struct` return/parameter types

```sh
ctysearch -f example.h "struct Test_t* (struct Test_t, uint8_t*)"
```

```text
example.h:8:1: test10 :: struct Test_t* (struct Test_t, uint8_t*)
example.h:17:1: test11_t :: struct Test_t* (struct Test_t, uint8_t*)
example.h:22:5: test12 :: struct Test_t* (struct Test_t*)
example.h:9:1: test13 :: enum Test_t* (enum Test_t, uint8_t*)
example.h:7:1: test3 :: Test_t* (Test_t, uint8_t*)
example.h:16:1: test7_t :: Test_t* (Test_t, uint8_t*)
example.h:6:1: test2 :: Test_t (uint8_t, uint8_t*)
example.h:15:1: test6_t :: Test_t (uint8_t, uint8_t*)
example.h:21:5: test9 :: Test_t* (Test_t*)
example.h:5:1: test1 :: void (uint8_t, uint8_t*)
```

`struct`/`enum` qualifiers are part of the signature, so `test10` (plain),
`test11_t` (typedef), and `test12` (struct member) all rank highly.

### 5. Fuzzy matching forgives small mistakes

```sh
ctysearch -f example.h "Test_t (uint8_t, uint8_t)"
```

```text
example.h:6:1: test2 :: Test_t (uint8_t, uint8_t*)
example.h:15:1: test6_t :: Test_t (uint8_t, uint8_t*)
example.h:7:1: test3 :: Test_t* (Test_t, uint8_t*)
example.h:16:1: test7_t :: Test_t* (Test_t, uint8_t*)
example.h:5:1: test1 :: void (uint8_t, uint8_t*)
example.h:14:1: test5_t :: void (uint8_t, uint8_t*)
example.h:4:1: test :: uint8_t* (uint8_t)
example.h:13:1: test4_t :: uint8_t* (uint8_t)
example.h:21:5: test9 :: Test_t* (Test_t*)
example.h:9:1: test13 :: enum Test_t* (enum Test_t, uint8_t*)
```

The query omits a `*`, yet the closest signatures (`test2`, `test6_t`) still
rank first.

### 6. Short vs long flags

```sh
ctysearch -f example.h -l 2 "void ()"
ctysearch --header example.h --limit 2 "void ()"
```

Both invocations print the same 2 lines. `--help` prints the usage text.

## Running the tests

```sh
cargo test
```

- Unit tests (`src/function.rs`, `src/parse.rs`): signature formatting,
  builder errors, parsing of `example.h` (16 unique functions, no
  duplicates), empty/single-function headers.
- Integration tests (`tests/cli.rs`): end-to-end CLI runs covering ranking,
  `--limit`, short/long flag equivalence, fuzzy matching, and error cases
  (missing args, nonexistent header, `--help`).
