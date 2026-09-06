public class Calculator {
    /**
     * Computes the sum of two values. The result is never negative.
     * <p>
     * This method is pure.
     *
     * @param a
     *                   the first value
     * @param longerName
     *                   the second value
     * @param flagged
     * @return the computed sum
     * @throws IOException           when reading fails
     * @throws IllegalStateException when the state is bad
     * @throws TimeoutException
     * @see #compute(int, int)
     */
    public int compute(int a, int longerName) throws IOException {
        return a + longerName;
    }
}
