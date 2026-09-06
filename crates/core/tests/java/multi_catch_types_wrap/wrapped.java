class A {
    void m() {
        try {
            work();
        } catch (IOException e) {
            handle(e);
        } catch (FirstLongException
                 | SecondLongException
                 | ThirdLongException e) {
            handleLong(e);
        } catch (Alpha | Beta e) {
            handleShort(e);
        }
    }
}
