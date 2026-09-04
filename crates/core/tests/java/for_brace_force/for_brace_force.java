class A {
    void m() {
        for (int i = 0; i < n; i++)
            use(i);
        for (int i = 0; i < n; i++)
            if (c)
                check(i);
        for (Item item : items)
            handle(item);
    }
}
