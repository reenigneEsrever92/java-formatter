class LambdaBlocks {
    void m(boolean go) {
        if (go) {
            step();
        }
        Runnable single = () -> {
            run();
        };
        Runnable multi = () -> {
            flushBuffer(buffer);
            closeConnection(connection);
        };
    }
}
