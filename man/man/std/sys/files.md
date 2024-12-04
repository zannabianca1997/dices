---
title: "File system interaction"
---
# File system interaction

The `dices` REPL can read and write utf-8 text file. At that use two intrisics are provided: `file_read` and `file_write`.

## Reading a file
`file_read` read a file from the system and loads it as a [string](man:types/strings). It receives a singular string as a parameter containing the path of the file, and return its content.

```dices mantest:ignore
#>>> let file_read = |path| "This is the content of the file"; // fake out the missing intrisic
>>> file_read("/path/to/file")
# ...
```

## Writing a file
`file_write` write a [string](man:types/strings) to a file. It receives the file path as the first parameter, and the file content as the second.

```dices mantest:ignore
#>>> let file_write = |path, content| null;                        // fake out the missing intrisic
#>>> let file_read = |path| "This is the new content of the file"; // fake out the missing intrisic
>>> file_write("/path/to/file", "This is the new content of the file");
>>> file_read("/path/to/file")
# ...
```