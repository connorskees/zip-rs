This library offers the ability to unzip files in-memory.

Implementation notes

- resilience to zip bombs. this comes at the cost of denying decompression of files over 5gb in size. this threshold may be configurable at some point
- zip files are memory mapped
- there is currently not support for zip64
- parsing is zero-copy
