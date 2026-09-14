class LambdaBody {
    Object use() {
        return supply(() -> new Runnable() {
            public void run() {
                go();
            }
        });
    }
}

