class ChainLambda {
    void test() {
        values.stream()
                .filter(v -> {
                    // keep me
                    return v > 0;
                })
                .forEach(v -> consume(v));
    }
}
