class InlineBodies {
    void m(int x, int n, boolean go) {
        if (x)
            foo();
        while (go)
            step();
        for (int i = 0; i < n; i++)
            use(i);
        for (int v : list)
            take(v);
        do
            tick();
         while (go);
    }
}
