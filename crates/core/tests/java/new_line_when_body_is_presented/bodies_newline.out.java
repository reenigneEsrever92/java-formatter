class A {
    void m(boolean ready, int n) {
        if (ready)
        { use(); } else
        { skip(); }
        while (n > 0)
        { tick(); }
        for (int i = 0; i < n; i++)
        { touch(i); }
    }

    int get()
    { return 1; }

    void read() throws IOException {
        try
        { work(); } catch (IOException e)
        { handle(e); }
    }
}
