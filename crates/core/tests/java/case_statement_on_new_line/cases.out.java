class A {
    void m(int x) {
        switch (x) {
            case 1: foo();
                bar();
                break;
            case 2: baz();
                break;
            default: break;
        }
    }
}
