class NewArgument {
    void use() {
        new Outer(new Runnable() {
            public void run() {
                go();
            }
        });
    }
}

