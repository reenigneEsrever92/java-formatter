class A {
    void m() {
        assert someVeryLongExpressionForTheAssertCondition() :
                "some very long assertion message";
        assert someLongConditionExpression() :
                someLongMessageExpression();
        assert
                simpleCondition();
    }
}
