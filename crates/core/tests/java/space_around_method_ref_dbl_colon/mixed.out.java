class A {
    void m() {
        Supplier<A> s = A :: new;
        log(A :: new);
    }
}
