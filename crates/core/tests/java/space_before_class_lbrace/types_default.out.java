class Outer {
    interface I {
        void m();
    }

    enum E {A, B}

    record R(int x) {}

    void use() {
        Runnable r = new Runnable() {
            public void run() {}
        };
    }
}
