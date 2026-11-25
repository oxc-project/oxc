// A declaration as a `for` BODY terminates itself even when the head has no
// init: the head-vs-body test is by span, not by parent kind.

for (;;) var noInit = 1

for (init;;) var withInit = 2
