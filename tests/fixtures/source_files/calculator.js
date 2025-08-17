/**
 * Simple JavaScript calculator for testing
 */
class Calculator {
    constructor() {
        this.result = 0;
    }

    add(value) {
        this.result += value;
        return this.result;
    }

    subtract(value) {
        this.result -= value;
        return this.result;
    }

    multiply(value) {
        this.result *= value;
        return this.result;
    }

    divide(value) {
        if (value === 0) {
            throw new Error("Division by zero");
        }
        this.result /= value;
        return this.result;
    }

    getResult() {
        return this.result;
    }

    clear() {
        this.result = 0;
    }
}

function calculateArea(length, width) {
    if (length <= 0 || width <= 0) {
        throw new Error("Dimensions must be positive");
    }
    return length * width;
}

module.exports = { Calculator, calculateArea };