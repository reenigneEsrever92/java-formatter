class CommentedSwitch {
    void test(int x) {
        switch (// the switch expression
                x + 1
        ) {
            case 1 -> System.out.println("one");
            default -> System.out.println("other");
        }
    }
}
