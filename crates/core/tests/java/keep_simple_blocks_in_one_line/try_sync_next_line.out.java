class A {
    void m() {
        try {
            foo();
        } catch (IOException e) {
            handle(e);
        } finally {
            cleanup();
        }
        try (Reader r = open()) {
            read(r);
        } catch (IOException e) {
            close();
        }
        synchronized (lock) {
            incr();
        }
        try {
            first();
            second();
        } catch (IOException e) {
            handle(e);
        }
        synchronized (lock) {
            first();
            second();
        }
    }
}
