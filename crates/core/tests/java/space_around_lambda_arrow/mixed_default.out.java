class A {
    void m() {
        Runnable r = () -> {
            run();
        };
        log((a, b) -> a + b);
    }
}
