class A {
    void m() throws Exception {
        try(java.io.StringReader r = new java.io.StringReader("x")) {
            f(r);
        }
    }

    void f(java.io.Reader r) {}
}
