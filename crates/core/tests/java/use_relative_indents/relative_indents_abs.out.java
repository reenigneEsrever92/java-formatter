class RelIndent {
	void method(boolean flag) {
		int total = alpha() +
				beta() +
				gamma() +
				delta() +
				epsilon() +
				zeta() +
				eta() +
				theta();
		if (flag) {
			int inner = alpha() +
					beta() +
					gamma() +
					delta() +
					epsilon() +
					zeta() +
					eta() +
					theta();
		}
	}
}
