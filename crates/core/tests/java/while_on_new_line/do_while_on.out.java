class DoWhile {
    void m(boolean go) {
        do {
            tick();
        }
        while (go);
        do
            tick();
        while (go);
    }
}
