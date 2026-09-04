class TryFinally {
    void m() {
        try {
            risky();
        }
        finally {
            cleanup();
        }
        try {
            risky();
        } catch (IOException e) {
            log(e);
        }
        finally {
            cleanup();
        }
    }
}
