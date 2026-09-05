class A {
    int width = 10, height = 20;

    void m(int n) {
        int lo = 0, hi = n;
        for (int i = lo, j = hi; i < j; i++, j--) {
            step(i, j);
        }
    }
}
