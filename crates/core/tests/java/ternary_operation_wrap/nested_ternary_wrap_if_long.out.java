class A {
    int m() {
        int x = longConditionalExpression() ?
                longConsequenceA() ? longConsequenceB() : longConsequenceC() :
                longAlternativeExpression();
        return x;
    }
}
