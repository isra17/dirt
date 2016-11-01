[![Build Status](https://travis-ci.org/isra17/dirt.svg?branch=master)](https://travis-ci.org/isra17/dirt)
# dirt
### Dynamic Identification and Recognition Technology

Experimental software that allow function identification through their behavior.

Can be build and run with cargo:

```
$ cargo run tests/integrations/bin1
```

The current version identify most std::string methods. Only works static linked
binary export their function. Final use case should be able to run binary and
functions from IDA.

Most of DIRT emulation is being moved into an independant project: [Popcorn](https://github.com/isra17/popcorn).
