class C {
    #p = 1;

    method(obj) {
        obj.x **= 2; // Transform
        obj['y'] **= 3; // Transform
        obj.#p **= 4; // Bail
    }
}
