"""Simple Python calculator for testing"""

class Calculator:
    def __init__(self):
        self.result = 0.0

    def add(self, value):
        """Add value to current result"""
        self.result += value
        return self.result

    def subtract(self, value):
        """Subtract value from current result"""
        self.result -= value
        return self.result

    def multiply(self, value):
        """Multiply current result by value"""
        self.result *= value
        return self.result

    def divide(self, value):
        """Divide current result by value"""
        if value == 0:
            raise ValueError("Division by zero")
        self.result /= value
        return self.result

    def get_result(self):
        """Get current result"""
        return self.result

    def clear(self):
        """Reset result to zero"""
        self.result = 0.0

def calculate_area(length, width):
    """Calculate rectangle area"""
    if length <= 0 or width <= 0:
        raise ValueError("Dimensions must be positive")
    return length * width