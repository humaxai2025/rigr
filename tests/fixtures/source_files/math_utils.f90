! Simple Fortran math utilities for testing
program math_utils
    implicit none
    
    real :: num1, num2, result
    character(len=1) :: operation
    
    print *, 'Simple Fortran Calculator for Testing'
    print *, 'Enter first number:'
    read *, num1
    
    print *, 'Enter operation (+, -, *, /):'
    read *, operation
    
    print *, 'Enter second number:'
    read *, num2
    
    select case(operation)
        case('+')
            result = add_numbers(num1, num2)
        case('-')
            result = subtract_numbers(num1, num2)
        case('*')
            result = multiply_numbers(num1, num2)
        case('/')
            if (num2 == 0.0) then
                print *, 'Error: Division by zero'
                stop
            else
                result = divide_numbers(num1, num2)
            end if
        case default
            print *, 'Invalid operation'
            stop
    end select
    
    print *, 'Result:', result
    
end program math_utils

function add_numbers(a, b) result(sum)
    implicit none
    real, intent(in) :: a, b
    real :: sum
    sum = a + b
end function add_numbers

function subtract_numbers(a, b) result(difference)
    implicit none
    real, intent(in) :: a, b
    real :: difference
    difference = a - b
end function subtract_numbers

function multiply_numbers(a, b) result(product)
    implicit none
    real, intent(in) :: a, b
    real :: product
    product = a * b
end function multiply_numbers

function divide_numbers(a, b) result(quotient)
    implicit none
    real, intent(in) :: a, b
    real :: quotient
    quotient = a / b
end function divide_numbers