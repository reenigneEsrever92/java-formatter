class A {
    void m(int x) {
        switch (x) {
            case 1:
                f();
                break;
            default:
                g();
        }
    }

    void f() {}

    void g() {}
}
