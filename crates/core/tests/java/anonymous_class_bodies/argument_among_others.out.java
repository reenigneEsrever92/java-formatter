class ArgumentAmongOthers {
    void use() {
        submit(
            1,
            new Runnable() {
                public void run() {
                    go();
                }
            },
            2);
    }
}

