! Scientific Calculator - Fortran 90
! This program demonstrates advanced mathematical operations
! for comprehensive test case generation

program scientific_calculator
    implicit none
    
    ! Variable declarations
    real(kind=8) :: x, y, result, angle, base, exponent
    integer :: choice, n, factorial_result
    logical :: continue_calc
    character(len=20) :: operation
    
    continue_calc = .true.
    
    do while (continue_calc)
        call display_menu()
        read *, choice
        
        select case (choice)
            case (1)
                call basic_arithmetic()
            case (2)
                call trigonometric_functions()
            case (3)
                call logarithmic_functions()
            case (4)
                call power_functions()
            case (5)
                call statistical_functions()
            case (6)
                call matrix_operations()
            case (0)
                continue_calc = .false.
            case default
                print *, 'Invalid choice. Please try again.'
        end select
    end do
    
    print *, 'Thank you for using Scientific Calculator'
    
contains

    subroutine display_menu()
        print *, '=== Scientific Calculator ==='
        print *, '1. Basic Arithmetic'
        print *, '2. Trigonometric Functions'
        print *, '3. Logarithmic Functions'
        print *, '4. Power Functions'
        print *, '5. Statistical Functions'
        print *, '6. Matrix Operations'
        print *, '0. Exit'
        print *, 'Enter your choice: '
    end subroutine display_menu

    subroutine basic_arithmetic()
        real(kind=8) :: num1, num2
        integer :: operation_choice
        
        print *, 'Enter first number: '
        read *, num1
        print *, 'Enter second number: '
        read *, num2
        
        print *, '1. Add  2. Subtract  3. Multiply  4. Divide'
        read *, operation_choice
        
        select case (operation_choice)
            case (1)
                result = add_numbers(num1, num2)
                print *, 'Result: ', result
            case (2)
                result = subtract_numbers(num1, num2)
                print *, 'Result: ', result
            case (3)
                result = multiply_numbers(num1, num2)
                print *, 'Result: ', result
            case (4)
                if (num2 /= 0.0d0) then
                    result = divide_numbers(num1, num2)
                    print *, 'Result: ', result
                else
                    print *, 'Error: Division by zero!'
                end if
        end select
    end subroutine basic_arithmetic

    function add_numbers(a, b) result(sum)
        real(kind=8), intent(in) :: a, b
        real(kind=8) :: sum
        sum = a + b
    end function add_numbers

    function subtract_numbers(a, b) result(difference)
        real(kind=8), intent(in) :: a, b
        real(kind=8) :: difference
        difference = a - b
    end function subtract_numbers

    function multiply_numbers(a, b) result(product)
        real(kind=8), intent(in) :: a, b
        real(kind=8) :: product
        product = a * b
    end function multiply_numbers

    function divide_numbers(a, b) result(quotient)
        real(kind=8), intent(in) :: a, b
        real(kind=8) :: quotient
        if (abs(b) > 1.0d-12) then
            quotient = a / b
        else
            quotient = 0.0d0
            print *, 'Warning: Division by very small number'
        end if
    end function divide_numbers

    subroutine trigonometric_functions()
        real(kind=8) :: angle_deg, angle_rad
        
        print *, 'Enter angle in degrees: '
        read *, angle_deg
        
        angle_rad = degrees_to_radians(angle_deg)
        
        print *, 'Sin(', angle_deg, ') = ', sin(angle_rad)
        print *, 'Cos(', angle_deg, ') = ', cos(angle_rad)
        print *, 'Tan(', angle_deg, ') = ', tan(angle_rad)
        
        ! Test inverse functions
        if (abs(sin(angle_rad)) <= 1.0d0) then
            print *, 'Arcsin = ', radians_to_degrees(asin(sin(angle_rad)))
        end if
        if (abs(cos(angle_rad)) <= 1.0d0) then
            print *, 'Arccos = ', radians_to_degrees(acos(cos(angle_rad)))
        end if
        print *, 'Arctan = ', radians_to_degrees(atan(tan(angle_rad)))
    end subroutine trigonometric_functions

    function degrees_to_radians(degrees) result(radians)
        real(kind=8), intent(in) :: degrees
        real(kind=8) :: radians
        real(kind=8), parameter :: pi = 3.141592653589793d0
        radians = degrees * pi / 180.0d0
    end function degrees_to_radians

    function radians_to_degrees(radians) result(degrees)
        real(kind=8), intent(in) :: radians
        real(kind=8) :: degrees
        real(kind=8), parameter :: pi = 3.141592653589793d0
        degrees = radians * 180.0d0 / pi
    end function radians_to_degrees

    subroutine logarithmic_functions()
        real(kind=8) :: number, base_num
        
        print *, 'Enter a positive number: '
        read *, number
        
        if (number > 0.0d0) then
            print *, 'Natural log of ', number, ' = ', log(number)
            print *, 'Log base 10 of ', number, ' = ', log10(number)
            
            print *, 'Enter base for logarithm: '
            read *, base_num
            
            if (base_num > 0.0d0 .and. base_num /= 1.0d0) then
                result = logarithm_base_n(number, base_num)
                print *, 'Log base ', base_num, ' of ', number, ' = ', result
            else
                print *, 'Invalid base for logarithm'
            end if
        else
            print *, 'Error: Cannot compute logarithm of non-positive number'
        end if
    end subroutine logarithmic_functions

    function logarithm_base_n(x, base) result(log_result)
        real(kind=8), intent(in) :: x, base
        real(kind=8) :: log_result
        
        if (x > 0.0d0 .and. base > 0.0d0 .and. base /= 1.0d0) then
            log_result = log(x) / log(base)
        else
            log_result = 0.0d0
            print *, 'Error in logarithm calculation'
        end if
    end function logarithm_base_n

    subroutine power_functions()
        real(kind=8) :: base_val, exp_val
        integer :: int_exp
        
        print *, 'Enter base: '
        read *, base_val
        print *, 'Enter exponent: '
        read *, exp_val
        
        ! Handle special cases
        if (base_val == 0.0d0 .and. exp_val <= 0.0d0) then
            print *, 'Error: 0 raised to non-positive power'
        else if (base_val < 0.0d0 .and. abs(exp_val - nint(exp_val)) > 1.0d-12) then
            print *, 'Error: Negative base with non-integer exponent'
        else
            result = power_function(base_val, exp_val)
            print *, base_val, ' ^ ', exp_val, ' = ', result
            
            ! Test integer exponent
            int_exp = nint(exp_val)
            if (abs(exp_val - int_exp) < 1.0d-12) then
                result = integer_power(base_val, int_exp)
                print *, 'Integer power result: ', result
            end if
        end if
        
        ! Test square root
        if (base_val >= 0.0d0) then
            print *, 'Square root of ', base_val, ' = ', sqrt(base_val)
        end if
    end subroutine power_functions

    function power_function(base, exponent) result(power_result)
        real(kind=8), intent(in) :: base, exponent
        real(kind=8) :: power_result
        
        if (base > 0.0d0) then
            power_result = exp(exponent * log(base))
        else if (base == 0.0d0 .and. exponent > 0.0d0) then
            power_result = 0.0d0
        else if (base < 0.0d0 .and. abs(exponent - nint(exponent)) < 1.0d-12) then
            power_result = sign(1.0d0, base) * (abs(base) ** nint(exponent))
        else
            power_result = 0.0d0
            print *, 'Invalid power operation'
        end if
    end function power_function

    function integer_power(base, exponent) result(int_power_result)
        real(kind=8), intent(in) :: base
        integer, intent(in) :: exponent
        real(kind=8) :: int_power_result
        integer :: i
        
        int_power_result = 1.0d0
        
        if (exponent == 0) then
            int_power_result = 1.0d0
        else if (exponent > 0) then
            do i = 1, exponent
                int_power_result = int_power_result * base
            end do
        else
            do i = 1, abs(exponent)
                int_power_result = int_power_result * base
            end do
            if (abs(int_power_result) > 1.0d-12) then
                int_power_result = 1.0d0 / int_power_result
            else
                print *, 'Error: Division by zero in negative power'
                int_power_result = 0.0d0
            end if
        end if
    end function integer_power

    subroutine statistical_functions()
        real(kind=8), dimension(100) :: data
        real(kind=8) :: mean, variance, std_dev
        integer :: n, i
        
        print *, 'Enter number of data points (max 100): '
        read *, n
        
        if (n > 0 .and. n <= 100) then
            print *, 'Enter data points:'
            do i = 1, n
                read *, data(i)
            end do
            
            mean = calculate_mean(data, n)
            variance = calculate_variance(data, n, mean)
            std_dev = sqrt(variance)
            
            print *, 'Mean: ', mean
            print *, 'Variance: ', variance
            print *, 'Standard Deviation: ', std_dev
            print *, 'Minimum: ', minval(data(1:n))
            print *, 'Maximum: ', maxval(data(1:n))
        else
            print *, 'Invalid number of data points'
        end if
    end subroutine statistical_functions

    function calculate_mean(array, size) result(mean)
        integer, intent(in) :: size
        real(kind=8), dimension(size), intent(in) :: array
        real(kind=8) :: mean
        integer :: i
        
        mean = 0.0d0
        do i = 1, size
            mean = mean + array(i)
        end do
        mean = mean / real(size, kind=8)
    end function calculate_mean

    function calculate_variance(array, size, mean) result(variance)
        integer, intent(in) :: size
        real(kind=8), dimension(size), intent(in) :: array
        real(kind=8), intent(in) :: mean
        real(kind=8) :: variance
        integer :: i
        
        variance = 0.0d0
        do i = 1, size
            variance = variance + (array(i) - mean)**2
        end do
        variance = variance / real(size - 1, kind=8)
    end function calculate_variance

    subroutine matrix_operations()
        real(kind=8), dimension(3,3) :: matrix_a, matrix_b, result_matrix
        real(kind=8) :: determinant
        integer :: i, j
        
        print *, 'Enter 3x3 Matrix A:'
        do i = 1, 3
            do j = 1, 3
                read *, matrix_a(i,j)
            end do
        end do
        
        print *, 'Enter 3x3 Matrix B:'
        do i = 1, 3
            do j = 1, 3
                read *, matrix_b(i,j)
            end do
        end do
        
        ! Matrix addition
        call matrix_add(matrix_a, matrix_b, result_matrix)
        print *, 'Matrix A + B:'
        call print_matrix(result_matrix)
        
        ! Matrix multiplication
        call matrix_multiply(matrix_a, matrix_b, result_matrix)
        print *, 'Matrix A * B:'
        call print_matrix(result_matrix)
        
        ! Determinant of matrix A
        determinant = calculate_determinant_3x3(matrix_a)
        print *, 'Determinant of Matrix A: ', determinant
    end subroutine matrix_operations

    subroutine matrix_add(a, b, c)
        real(kind=8), dimension(3,3), intent(in) :: a, b
        real(kind=8), dimension(3,3), intent(out) :: c
        integer :: i, j
        
        do i = 1, 3
            do j = 1, 3
                c(i,j) = a(i,j) + b(i,j)
            end do
        end do
    end subroutine matrix_add

    subroutine matrix_multiply(a, b, c)
        real(kind=8), dimension(3,3), intent(in) :: a, b
        real(kind=8), dimension(3,3), intent(out) :: c
        integer :: i, j, k
        
        do i = 1, 3
            do j = 1, 3
                c(i,j) = 0.0d0
                do k = 1, 3
                    c(i,j) = c(i,j) + a(i,k) * b(k,j)
                end do
            end do
        end do
    end subroutine matrix_multiply

    function calculate_determinant_3x3(matrix) result(det)
        real(kind=8), dimension(3,3), intent(in) :: matrix
        real(kind=8) :: det
        
        det = matrix(1,1) * (matrix(2,2) * matrix(3,3) - matrix(2,3) * matrix(3,2)) - &
              matrix(1,2) * (matrix(2,1) * matrix(3,3) - matrix(2,3) * matrix(3,1)) + &
              matrix(1,3) * (matrix(2,1) * matrix(3,2) - matrix(2,2) * matrix(3,1))
    end function calculate_determinant_3x3

    subroutine print_matrix(matrix)
        real(kind=8), dimension(3,3), intent(in) :: matrix
        integer :: i, j
        
        do i = 1, 3
            write(*,'(3F10.3)') (matrix(i,j), j = 1, 3)
        end do
    end subroutine print_matrix

end program scientific_calculator