class MultiStatement {
    void test() {
        list.forEach(x -> {
            use(x);
            more(x);
        });
    }

    void controlFlow() {
        valuesOfForm.forEach(value -> {
            if (x) {
                doSomething(value);
            }
        });
    }

    void blockComment() {
        list.forEach(x -> { /* inline */ use(x); });
    }
}
