class A {
    <T> T cast(Object o) {
        return (T) o;
    }

    void m() {
        String s = this.<String>cast("x");
        java.util.List<String> l = new <String>java.util.ArrayList<String>();
        java.util.stream.Collector<String, ?, java.util.List<String>> c =
            java.util.stream.Collectors.<String, java.util.List<String>>toList();
        StringBuilder sb = new StringBuilder();
        sb.append("a")
            .<String>append("b")
            .append("c");
    }
}
