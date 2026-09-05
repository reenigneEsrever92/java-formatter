class A {
    void m(int n) {
        for (int i = 0, j = n; i < j; i++, j--) {
            use(i, j);
        }
    }
}
