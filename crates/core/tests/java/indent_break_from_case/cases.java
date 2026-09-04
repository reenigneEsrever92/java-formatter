class A {
    void m(int x) {
        while (true) {
            switch (x) {
                case 1:
                    foo();
                    continue;
                case 2:
                    bar();
                    return;
                default:
                    break;
            }
        }
    }
}
