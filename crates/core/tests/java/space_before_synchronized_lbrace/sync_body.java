class A {
    void m() {
        synchronized (this) {
            f();
        }
    }

    void f() {}
}
