class TryCatchFinally {
    void m() {
        try {
            risky();
        }
        catch (IOException e) {
            log(e);
        }
        catch (Exception e) {
            log2(e);
        } finally {
            cleanup();
        }
    }
}
