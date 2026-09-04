class OwnLineBodies {
    void m(int x, int n, boolean go) {
        if (x)
            foo();
        while (go)
            step();
        for (int i = 0; i < n; i++)
            use(i);
        do
            tick();
         while (go);
    }
}
