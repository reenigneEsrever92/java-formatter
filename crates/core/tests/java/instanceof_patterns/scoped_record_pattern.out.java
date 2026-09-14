class ScopedPatterns {
    void check(Object o) {
        if (o instanceof Outer.Inner.Record(var value)) {
            handle(value);
        } else if (o instanceof Outer.Inner.Other(var other)) {
            handle(other);
        }
        while (o instanceof Outer.Inner.Record(var value)) {
            handle(value);
        }
        do {
            handle(o);
        } while (o instanceof Outer.Inner.Record(var value));
    }
}
