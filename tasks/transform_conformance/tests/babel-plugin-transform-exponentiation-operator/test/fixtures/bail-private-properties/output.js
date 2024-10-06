class C {
    #p = 1;

    method(obj) {
        obj["x"] = Math.pow(obj["x"], 2);
        obj["y"] = Math.pow(obj["y"], 3);
        obj.#p **= 4;
    }
}
