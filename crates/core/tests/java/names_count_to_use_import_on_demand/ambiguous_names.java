import static a.one.Methods.run;
import static a.one.Methods.m2;
import static a.one.Methods.m3;
import static b.two.Other.run;
import static b.two.Other.x;
class Use {
    void go() {
        run();
        m2();
        m3();
        x();
    }
}
