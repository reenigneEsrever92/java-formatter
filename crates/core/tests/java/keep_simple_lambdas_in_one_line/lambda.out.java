class F {
    void m() {
        list.forEach(x -> { use(x); });
        Runnable single = () -> { run(); };
        Runnable multi = () -> {
            flushBuffer(buffer);
            closeConnection(connection);
        };
    }
}
