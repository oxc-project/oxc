try {
    // Some code.
} catch (e) {
    LABEL:
    {
        if (condition) {
            break LABEL;
        }
        // Remaining code.
    }
}
