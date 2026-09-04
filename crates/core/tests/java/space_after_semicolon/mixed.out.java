class A {
    void m(int n) {
        for (int i = 0;i < n;i++) {
            use(i);
        }
        for (int i = 0, j = n;i < j;i++, j--) {
            step(i, j);
        }
    }
}
