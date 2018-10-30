# snapshot-call

## Synopsis

Given a C function

`ret_t f(arg1_t arg1, ..., argN_t argN)`

generate a

`void snapshot_f(FILE *stream, arg1_t arg1, ..., argN_t argN)`

function, which analyzes global variables used by `f` as well as its arguments,
and prints C code to initialize them from scratch and then pass them to `f`.

## Usage

`snapshot-call stuff.c [f ...] >snapshot-stuff.h`

## How is this useful?

If `f` is part of a big project and is buggy, helps creating tests out of real
calls.
