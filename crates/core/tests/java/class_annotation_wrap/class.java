@Deprecated
class OldThing {
    int x;
}

@Entity
@Table(name = "widgets")
public class Widget {
    int y;
}

class Plain {
    int z;
}

@VeryLongAnnotationName(someArgument = "a long value here")
@AnotherLongAnnotationName(withAnotherArgument = {1, 2, 3})
class ExtremelyLongClassNameThatOverflowsTheNarrowMargin extends BaseClass implements InterfaceOne, InterfaceTwo {
    int w;
}