class A {
    void m() {
        try {
            f();
        } catch (Exception e) {
            g();
        }
    }

    void f() {}

    void g() {}
}
