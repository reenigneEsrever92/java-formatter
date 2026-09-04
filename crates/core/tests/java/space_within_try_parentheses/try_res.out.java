class A {
    void m() {
        try ( Reader r = open() ) {
            read(r);
        }
    }
}
