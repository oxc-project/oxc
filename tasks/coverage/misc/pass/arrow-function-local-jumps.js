(() => {
    while (true) { break; }
    while (true) { continue; }
    outer: while (true) { break outer; }
    outer: while (true) { continue outer; }
});
