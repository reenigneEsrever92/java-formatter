class A {
    void m() {
        try {
            f();
        } finally {
            g();
        }
    }

    void f() {}

    void g() {}
}
