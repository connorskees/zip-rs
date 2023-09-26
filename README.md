This library offers the ability to unzip files in-memory.

Implementation notes

- resilience to zip bombs through configuring decompression limit
- ZIP archives read from the file system are memory mapped
- there is currently not support for zip64
- parsing is zero-copy
