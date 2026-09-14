import java.util.function.Function;

class Argument {
    void use() {
        stream.map(new Function<String, String>() {
            @Override
            public String apply(String s) {
                return s;
            }
        });
    }
}
