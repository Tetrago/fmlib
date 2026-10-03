# fmlib

Naive [FM-index](https://en.wikipedia.org/wiki/FM-index) implementation. Indexing speed heavily limited
by trivial sort, which was not a design priority.

## Usage

```sh
cargo run -p fmcli -- <file-path>
>> my search query
...
>> :3 ... # Return only three matches
...
>> :: # Search only for a semicolon
```
