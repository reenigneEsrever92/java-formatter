class Alpha extends AbstractAlpha implements Named,
        Sized,
        Registry<Alpha>,
        Marked {}

interface Beta extends FirstInterface,
        SecondInterface,
        ThirdInterface {}

enum Level implements Named, Sized, Comparable<Level> {
}

record Point(int x, int y) implements Named, Sized {}
