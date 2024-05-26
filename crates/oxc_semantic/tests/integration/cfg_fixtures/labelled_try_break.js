var foo = function () {
    label: try {
        return 0;
    } finally {
        break label;
    }

    return 1;
}
