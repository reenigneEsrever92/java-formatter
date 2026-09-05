class TabIndent {

    void method(boolean flag) {
	if (flag) {
	    int total = alpha() +
		    beta() +
		    gamma() +
		    delta() +
		    epsilon() +
		    zeta();
	    call(
		alpha(),
		beta(),
		gamma());
	} else {
	    for (int i = 0; i < 10; i++) {
		list.add(i);
	    }
	}
    }

    String join(String left, String right) {
	return left +
		right +
		left +
		right +
		left +
		right +
		left +
		right +
		left +
		right;
    }
}
