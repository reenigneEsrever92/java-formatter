class A {
    void m() {
        try {
            g();
        } catch (IOException e) {
            h(e);
        }
    }
}
