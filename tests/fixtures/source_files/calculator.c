#include <stdio.h>
#include <stdlib.h>

/**
 * Simple C calculator for testing
 */

typedef struct {
    double result;
} Calculator;

Calculator* calculator_new() {
    Calculator* calc = malloc(sizeof(Calculator));
    if (calc != NULL) {
        calc->result = 0.0;
    }
    return calc;
}

void calculator_free(Calculator* calc) {
    if (calc != NULL) {
        free(calc);
    }
}

double calculator_add(Calculator* calc, double value) {
    if (calc == NULL) return 0.0;
    calc->result += value;
    return calc->result;
}

double calculator_subtract(Calculator* calc, double value) {
    if (calc == NULL) return 0.0;
    calc->result -= value;
    return calc->result;
}

double calculator_multiply(Calculator* calc, double value) {
    if (calc == NULL) return 0.0;
    calc->result *= value;
    return calc->result;
}

int calculator_divide(Calculator* calc, double value, double* result) {
    if (calc == NULL || result == NULL) return -1;
    if (value == 0.0) return -2; // Division by zero
    calc->result /= value;
    *result = calc->result;
    return 0;
}

double calculator_get_result(Calculator* calc) {
    if (calc == NULL) return 0.0;
    return calc->result;
}

void calculator_clear(Calculator* calc) {
    if (calc != NULL) {
        calc->result = 0.0;
    }
}

double calculate_area(double length, double width) {
    if (length <= 0 || width <= 0) {
        return -1; // Invalid dimensions
    }
    return length * width;
}