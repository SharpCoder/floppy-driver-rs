# Random Access Floppy

This project aims to treat a floppy disk as a random access memory source. What does that mean? Well, I plan to bitbang a floppy drive and create my own interface on top of it, next I will implement a custom allocator in my rust kernel and map it against the floppy drive. This means anytime I use the `new` keyword, it'll store the bytes on a floppy disk!

## Why?

Because I can.
