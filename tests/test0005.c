#include "test0005.h"

double foo(const struct quux *quux) {
    return quux->bar + quux->baz;
}
