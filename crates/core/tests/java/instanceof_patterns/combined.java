class Patterns {
    void check(Object o) {
        if (o instanceof String s && s.length() > 0) {
            System.out.println(s);
        }
        while (o instanceof List<String> list) {
            list.clear();
        }
        boolean b = o instanceof Point p ? true : false;
        Object v = o instanceof Integer i ? i : o;
    }
}