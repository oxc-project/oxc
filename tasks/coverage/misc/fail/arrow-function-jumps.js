while (true) { (() => { break; }); }

outer: while (true) { (() => { break outer; }); }

while (true) { (() => { continue; }); }

outer: while (true) { (() => { continue outer; }); }
