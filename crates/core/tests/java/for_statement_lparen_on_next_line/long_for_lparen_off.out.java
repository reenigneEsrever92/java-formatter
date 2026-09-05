class A {
    void m() {
        for (int someLongVariableName = 0;
                someLongVariableName < someLongUpperBoundExpression();
                someLongVariableName++) {
            System.out.println(someLongVariableName);
        }
        for (String someVeryLongElementName :
                someVeryLongCollectionExpression()) {
            System.out.println(someVeryLongElementName);
        }
    }
}
