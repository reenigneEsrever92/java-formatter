class ScopedExpressions {
    void local(Object o) {
        boolean plain = o instanceof Outer.Inner.Record(var value);
        boolean ternary = o instanceof Outer.Inner.Record(var value) ? true : false;
        call(o instanceof Outer.Inner.Record(var value) ? 1 : 2);
    }

    int pick(Object o) {
        return o instanceof Outer.Inner.Record(var value) ? 1 : 2;
    }
}
