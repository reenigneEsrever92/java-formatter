class A {
    void m(int n) {
        for ( int i = 0; i < n; i++ ) {
            g(i);
        }
        for ( int x : xs ) {
            g(x);
        }
    }
}
