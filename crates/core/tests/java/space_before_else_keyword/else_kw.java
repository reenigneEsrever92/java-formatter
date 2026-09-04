class A {
    void m(int x) {
        if (x > 0) {
            f();
        } else {
            g();
        }
    }

    void f() {}

    void g() {}
}
