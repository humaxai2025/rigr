/**
 * Simple Java calculator for testing
 */
public class Calculator {
    private double result;

    public Calculator() {
        this.result = 0.0;
    }

    public double add(double value) {
        this.result += value;
        return this.result;
    }

    public double subtract(double value) {
        this.result -= value;
        return this.result;
    }

    public double multiply(double value) {
        this.result *= value;
        return this.result;
    }

    public double divide(double value) throws ArithmeticException {
        if (value == 0.0) {
            throw new ArithmeticException("Division by zero");
        }
        this.result /= value;
        return this.result;
    }

    public double getResult() {
        return this.result;
    }

    public void clear() {
        this.result = 0.0;
    }

    public static double calculateArea(double length, double width) {
        if (length <= 0 || width <= 0) {
            throw new IllegalArgumentException("Dimensions must be positive");
        }
        return length * width;
    }
}