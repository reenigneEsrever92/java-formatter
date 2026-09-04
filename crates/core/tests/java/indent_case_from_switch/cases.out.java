class A {
    void m(int x) {
        switch (x) {
        case 1:
            foo();
            bar();
            break;
        case 2:
            baz();
            return;
        default:
            break;
        }
        switch (x) {
        case 1 -> foo();
        case 2 -> bar();
        default -> baz();
        }
    }
}
