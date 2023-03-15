# dupes
Basic duplicate file finder via sha1 hashing
## Features
- Recursive directory search for duplicate files
- Multithreaded via a threadpool
## Example Output
```
$ dupes <directory>
test/helloworld.txt = test/test2.txt
test/odyssey.mb.txt = test/odyssey2.txt
test/test3.txt = test/test2.txt
```
