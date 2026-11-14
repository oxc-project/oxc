function breakFinally() {
    while (true) {
        try { break; } finally { cleanup(); }
    }
    after();
}
function continueFinally() {
    for (let i = 0; i < 2; i++) {
        try { continue; } finally { cleanup(); }
    }
    after();
}
function nestedFinally() {
    outer: while (condition()) {
        try {
            try { break outer; } finally { inner(); }
        } finally { outerCleanup(); }
    }
    after();
}
function innerLoop() {
    try {
        while (condition()) { break; }
        inside();
    } finally { cleanup(); }
    after();
}
function overridingFinally() {
    while (true) {
        try { break; } finally { return; }
    }
    unreachable();
}
function labeledContinue() {
    outer: while (condition()) {
        try {
            while (condition()) { continue outer; }
        } finally { cleanup(); }
    }
    after();
}
