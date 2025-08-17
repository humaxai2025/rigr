-- Employee Management System - PL/SQL Package
-- Comprehensive test program for PL/SQL functionality

-- Create Employee table for testing
CREATE TABLE employees (
    employee_id NUMBER PRIMARY KEY,
    first_name VARCHAR2(50) NOT NULL,
    last_name VARCHAR2(50) NOT NULL,
    email VARCHAR2(100) UNIQUE,
    hire_date DATE DEFAULT SYSDATE,
    job_id VARCHAR2(10) NOT NULL,
    salary NUMBER(8,2),
    commission_pct NUMBER(2,2),
    manager_id NUMBER,
    department_id NUMBER
);

-- Create Department table for testing
CREATE TABLE departments (
    department_id NUMBER PRIMARY KEY,
    department_name VARCHAR2(30) NOT NULL,
    manager_id NUMBER,
    location_id NUMBER
);

-- Create sequence for employee IDs
CREATE SEQUENCE emp_seq
    START WITH 1
    INCREMENT BY 1
    NOCACHE;

-- Employee Management Package Specification
CREATE OR REPLACE PACKAGE emp_management AS
    -- Custom exceptions
    EMPLOYEE_NOT_FOUND EXCEPTION;
    INVALID_SALARY EXCEPTION;
    DUPLICATE_EMAIL EXCEPTION;
    
    -- Type definitions
    TYPE emp_record_type IS RECORD (
        emp_id employees.employee_id%TYPE,
        full_name VARCHAR2(101),
        email employees.email%TYPE,
        salary employees.salary%TYPE,
        hire_date employees.hire_date%TYPE
    );
    
    TYPE emp_array_type IS TABLE OF emp_record_type INDEX BY PLS_INTEGER;
    
    -- Cursor definitions
    CURSOR high_salary_cursor IS
        SELECT employee_id, first_name || ' ' || last_name AS full_name, salary
        FROM employees
        WHERE salary > 75000
        ORDER BY salary DESC;
    
    -- Public procedures and functions
    PROCEDURE add_employee(
        p_first_name IN VARCHAR2,
        p_last_name IN VARCHAR2,
        p_email IN VARCHAR2,
        p_job_id IN VARCHAR2,
        p_salary IN NUMBER DEFAULT 50000,
        p_manager_id IN NUMBER DEFAULT NULL,
        p_department_id IN NUMBER DEFAULT 10
    );
    
    FUNCTION get_employee_count(p_department_id IN NUMBER DEFAULT NULL) RETURN NUMBER;
    
    PROCEDURE update_salary(
        p_employee_id IN NUMBER,
        p_new_salary IN NUMBER,
        p_raise_percentage IN NUMBER DEFAULT NULL
    );
    
    FUNCTION calculate_annual_bonus(
        p_employee_id IN NUMBER,
        p_bonus_rate IN NUMBER DEFAULT 0.1
    ) RETURN NUMBER;
    
    PROCEDURE delete_employee(p_employee_id IN NUMBER);
    
    PROCEDURE bulk_salary_increase(
        p_department_id IN NUMBER,
        p_percentage IN NUMBER
    );
    
    FUNCTION get_department_employees(p_dept_id IN NUMBER) RETURN emp_array_type;
    
    PROCEDURE generate_employee_report(
        p_department_id IN NUMBER DEFAULT NULL,
        p_min_salary IN NUMBER DEFAULT 0
    );
    
END emp_management;
/

-- Employee Management Package Body
CREATE OR REPLACE PACKAGE BODY emp_management AS
    
    -- Global variables
    g_max_salary CONSTANT NUMBER := 999999.99;
    g_min_salary CONSTANT NUMBER := 10000;
    g_log_enabled BOOLEAN := TRUE;
    
    -- Private logging procedure
    PROCEDURE log_action(p_action VARCHAR2, p_employee_id NUMBER DEFAULT NULL) IS
        PRAGMA AUTONOMOUS_TRANSACTION;
    BEGIN
        IF g_log_enabled THEN
            INSERT INTO emp_audit_log (
                log_date, 
                action_type, 
                employee_id, 
                performed_by
            ) VALUES (
                SYSDATE, 
                p_action, 
                p_employee_id, 
                USER
            );
            COMMIT;
        END IF;
    EXCEPTION
        WHEN OTHERS THEN
            -- Silently ignore logging errors
            NULL;
    END log_action;
    
    -- Validate email format
    FUNCTION is_valid_email(p_email VARCHAR2) RETURN BOOLEAN IS
    BEGIN
        RETURN REGEXP_LIKE(p_email, '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$');
    END is_valid_email;
    
    -- Add new employee
    PROCEDURE add_employee(
        p_first_name IN VARCHAR2,
        p_last_name IN VARCHAR2,
        p_email IN VARCHAR2,
        p_job_id IN VARCHAR2,
        p_salary IN NUMBER DEFAULT 50000,
        p_manager_id IN NUMBER DEFAULT NULL,
        p_department_id IN NUMBER DEFAULT 10
    ) IS
        v_employee_id NUMBER;
        v_email_count NUMBER;
    BEGIN
        -- Input validation
        IF p_first_name IS NULL OR LENGTH(TRIM(p_first_name)) = 0 THEN
            RAISE_APPLICATION_ERROR(-20001, 'First name cannot be null or empty');
        END IF;
        
        IF p_last_name IS NULL OR LENGTH(TRIM(p_last_name)) = 0 THEN
            RAISE_APPLICATION_ERROR(-20002, 'Last name cannot be null or empty');
        END IF;
        
        IF NOT is_valid_email(p_email) THEN
            RAISE_APPLICATION_ERROR(-20003, 'Invalid email format');
        END IF;
        
        IF p_salary < g_min_salary OR p_salary > g_max_salary THEN
            RAISE INVALID_SALARY;
        END IF;
        
        -- Check for duplicate email
        SELECT COUNT(*)
        INTO v_email_count
        FROM employees
        WHERE UPPER(email) = UPPER(p_email);
        
        IF v_email_count > 0 THEN
            RAISE DUPLICATE_EMAIL;
        END IF;
        
        -- Generate new employee ID
        SELECT emp_seq.NEXTVAL INTO v_employee_id FROM DUAL;
        
        -- Insert new employee
        INSERT INTO employees (
            employee_id,
            first_name,
            last_name,
            email,
            hire_date,
            job_id,
            salary,
            manager_id,
            department_id
        ) VALUES (
            v_employee_id,
            INITCAP(p_first_name),
            INITCAP(p_last_name),
            LOWER(p_email),
            SYSDATE,
            UPPER(p_job_id),
            p_salary,
            p_manager_id,
            p_department_id
        );
        
        log_action('INSERT', v_employee_id);
        
        DBMS_OUTPUT.PUT_LINE('Employee added successfully with ID: ' || v_employee_id);
        
    EXCEPTION
        WHEN INVALID_SALARY THEN
            RAISE_APPLICATION_ERROR(-20004, 'Salary must be between ' || g_min_salary || ' and ' || g_max_salary);
        WHEN DUPLICATE_EMAIL THEN
            RAISE_APPLICATION_ERROR(-20005, 'Email address already exists: ' || p_email);
        WHEN DUP_VAL_ON_INDEX THEN
            RAISE_APPLICATION_ERROR(-20006, 'Duplicate value error occurred');
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20000, 'Unexpected error: ' || SQLERRM);
    END add_employee;
    
    -- Get employee count
    FUNCTION get_employee_count(p_department_id IN NUMBER DEFAULT NULL) RETURN NUMBER IS
        v_count NUMBER;
    BEGIN
        IF p_department_id IS NULL THEN
            SELECT COUNT(*)
            INTO v_count
            FROM employees;
        ELSE
            SELECT COUNT(*)
            INTO v_count
            FROM employees
            WHERE department_id = p_department_id;
        END IF;
        
        RETURN v_count;
        
    EXCEPTION
        WHEN NO_DATA_FOUND THEN
            RETURN 0;
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20007, 'Error counting employees: ' || SQLERRM);
    END get_employee_count;
    
    -- Update employee salary
    PROCEDURE update_salary(
        p_employee_id IN NUMBER,
        p_new_salary IN NUMBER,
        p_raise_percentage IN NUMBER DEFAULT NULL
    ) IS
        v_current_salary NUMBER;
        v_final_salary NUMBER;
        v_employee_exists NUMBER;
    BEGIN
        -- Check if employee exists
        SELECT COUNT(*)
        INTO v_employee_exists
        FROM employees
        WHERE employee_id = p_employee_id;
        
        IF v_employee_exists = 0 THEN
            RAISE EMPLOYEE_NOT_FOUND;
        END IF;
        
        -- Get current salary
        SELECT salary
        INTO v_current_salary
        FROM employees
        WHERE employee_id = p_employee_id;
        
        -- Calculate final salary
        IF p_raise_percentage IS NOT NULL THEN
            v_final_salary := v_current_salary * (1 + p_raise_percentage / 100);
        ELSE
            v_final_salary := p_new_salary;
        END IF;
        
        -- Validate new salary
        IF v_final_salary < g_min_salary OR v_final_salary > g_max_salary THEN
            RAISE INVALID_SALARY;
        END IF;
        
        -- Update salary
        UPDATE employees
        SET salary = v_final_salary
        WHERE employee_id = p_employee_id;
        
        log_action('SALARY_UPDATE', p_employee_id);
        
        DBMS_OUTPUT.PUT_LINE('Salary updated from ' || v_current_salary || ' to ' || v_final_salary);
        
    EXCEPTION
        WHEN EMPLOYEE_NOT_FOUND THEN
            RAISE_APPLICATION_ERROR(-20008, 'Employee ID ' || p_employee_id || ' not found');
        WHEN INVALID_SALARY THEN
            RAISE_APPLICATION_ERROR(-20004, 'New salary must be between ' || g_min_salary || ' and ' || g_max_salary);
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20009, 'Error updating salary: ' || SQLERRM);
    END update_salary;
    
    -- Calculate annual bonus
    FUNCTION calculate_annual_bonus(
        p_employee_id IN NUMBER,
        p_bonus_rate IN NUMBER DEFAULT 0.1
    ) RETURN NUMBER IS
        v_salary NUMBER;
        v_hire_date DATE;
        v_years_employed NUMBER;
        v_bonus NUMBER;
    BEGIN
        SELECT salary, hire_date
        INTO v_salary, v_hire_date
        FROM employees
        WHERE employee_id = p_employee_id;
        
        v_years_employed := MONTHS_BETWEEN(SYSDATE, v_hire_date) / 12;
        
        -- Base bonus calculation
        v_bonus := v_salary * p_bonus_rate;
        
        -- Tenure multiplier
        IF v_years_employed >= 10 THEN
            v_bonus := v_bonus * 1.5;
        ELSIF v_years_employed >= 5 THEN
            v_bonus := v_bonus * 1.25;
        ELSIF v_years_employed >= 2 THEN
            v_bonus := v_bonus * 1.1;
        END IF;
        
        RETURN ROUND(v_bonus, 2);
        
    EXCEPTION
        WHEN NO_DATA_FOUND THEN
            RAISE_APPLICATION_ERROR(-20008, 'Employee ID ' || p_employee_id || ' not found');
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20010, 'Error calculating bonus: ' || SQLERRM);
    END calculate_annual_bonus;
    
    -- Delete employee
    PROCEDURE delete_employee(p_employee_id IN NUMBER) IS
        v_employee_count NUMBER;
    BEGIN
        SELECT COUNT(*)
        INTO v_employee_count
        FROM employees
        WHERE employee_id = p_employee_id;
        
        IF v_employee_count = 0 THEN
            RAISE EMPLOYEE_NOT_FOUND;
        END IF;
        
        DELETE FROM employees
        WHERE employee_id = p_employee_id;
        
        log_action('DELETE', p_employee_id);
        
        DBMS_OUTPUT.PUT_LINE('Employee ' || p_employee_id || ' deleted successfully');
        
    EXCEPTION
        WHEN EMPLOYEE_NOT_FOUND THEN
            RAISE_APPLICATION_ERROR(-20008, 'Employee ID ' || p_employee_id || ' not found');
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20011, 'Error deleting employee: ' || SQLERRM);
    END delete_employee;
    
    -- Bulk salary increase for department
    PROCEDURE bulk_salary_increase(
        p_department_id IN NUMBER,
        p_percentage IN NUMBER
    ) IS
        v_updated_count NUMBER;
    BEGIN
        UPDATE employees
        SET salary = salary * (1 + p_percentage / 100)
        WHERE department_id = p_department_id
        AND salary * (1 + p_percentage / 100) <= g_max_salary;
        
        v_updated_count := SQL%ROWCOUNT;
        
        log_action('BULK_SALARY_UPDATE');
        
        DBMS_OUTPUT.PUT_LINE('Salary increased for ' || v_updated_count || ' employees in department ' || p_department_id);
        
    EXCEPTION
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20012, 'Error in bulk salary update: ' || SQLERRM);
    END bulk_salary_increase;
    
    -- Get department employees as array
    FUNCTION get_department_employees(p_dept_id IN NUMBER) RETURN emp_array_type IS
        v_emp_array emp_array_type;
        v_index PLS_INTEGER := 1;
    BEGIN
        FOR emp_rec IN (
            SELECT employee_id, 
                   first_name || ' ' || last_name AS full_name,
                   email, 
                   salary, 
                   hire_date
            FROM employees
            WHERE department_id = p_dept_id
            ORDER BY last_name, first_name
        ) LOOP
            v_emp_array(v_index).emp_id := emp_rec.employee_id;
            v_emp_array(v_index).full_name := emp_rec.full_name;
            v_emp_array(v_index).email := emp_rec.email;
            v_emp_array(v_index).salary := emp_rec.salary;
            v_emp_array(v_index).hire_date := emp_rec.hire_date;
            
            v_index := v_index + 1;
        END LOOP;
        
        RETURN v_emp_array;
        
    EXCEPTION
        WHEN OTHERS THEN
            RAISE_APPLICATION_ERROR(-20013, 'Error retrieving department employees: ' || SQLERRM);
    END get_department_employees;
    
    -- Generate employee report
    PROCEDURE generate_employee_report(
        p_department_id IN NUMBER DEFAULT NULL,
        p_min_salary IN NUMBER DEFAULT 0
    ) IS
        v_total_salary NUMBER := 0;
        v_avg_salary NUMBER := 0;
        v_emp_count NUMBER := 0;
        v_max_salary NUMBER := 0;
        v_min_salary NUMBER := 999999;
        
        CURSOR emp_cursor IS
            SELECT e.employee_id,
                   e.first_name || ' ' || e.last_name AS full_name,
                   e.email,
                   e.salary,
                   e.hire_date,
                   d.department_name,
                   MONTHS_BETWEEN(SYSDATE, e.hire_date) AS months_employed
            FROM employees e
            JOIN departments d ON e.department_id = d.department_id
            WHERE (p_department_id IS NULL OR e.department_id = p_department_id)
            AND e.salary >= p_min_salary
            ORDER BY e.department_id, e.salary DESC;
    BEGIN
        DBMS_OUTPUT.PUT_LINE('=== EMPLOYEE REPORT ===');
        DBMS_OUTPUT.PUT_LINE('Generated on: ' || TO_CHAR(SYSDATE, 'DD-MON-YYYY HH24:MI:SS'));
        DBMS_OUTPUT.PUT_LINE('');
        
        FOR emp IN emp_cursor LOOP
            v_emp_count := v_emp_count + 1;
            v_total_salary := v_total_salary + emp.salary;
            
            IF emp.salary > v_max_salary THEN
                v_max_salary := emp.salary;
            END IF;
            
            IF emp.salary < v_min_salary THEN
                v_min_salary := emp.salary;
            END IF;
            
            DBMS_OUTPUT.PUT_LINE('Employee ID: ' || emp.employee_id);
            DBMS_OUTPUT.PUT_LINE('Name: ' || emp.full_name);
            DBMS_OUTPUT.PUT_LINE('Email: ' || emp.email);
            DBMS_OUTPUT.PUT_LINE('Department: ' || emp.department_name);
            DBMS_OUTPUT.PUT_LINE('Salary: $' || TO_CHAR(emp.salary, '999,999.99'));
            DBMS_OUTPUT.PUT_LINE('Hire Date: ' || TO_CHAR(emp.hire_date, 'DD-MON-YYYY'));
            DBMS_OUTPUT.PUT_LINE('Months Employed: ' || ROUND(emp.months_employed, 1));
            DBMS_OUTPUT.PUT_LINE('Annual Bonus: $' || TO_CHAR(calculate_annual_bonus(emp.employee_id), '999,999.99'));
            DBMS_OUTPUT.PUT_LINE('------------------------');
        END LOOP;
        
        IF v_emp_count > 0 THEN
            v_avg_salary := v_total_salary / v_emp_count;
        END IF;
        
        DBMS_OUTPUT.PUT_LINE('=== SUMMARY STATISTICS ===');
        DBMS_OUTPUT.PUT_LINE('Total Employees: ' || v_emp_count);
        DBMS_OUTPUT.PUT_LINE('Total Salary Budget: $' || TO_CHAR(v_total_salary, '999,999,999.99'));
        DBMS_OUTPUT.PUT_LINE('Average Salary: $' || TO_CHAR(v_avg_salary, '999,999.99'));
        DBMS_OUTPUT.PUT_LINE('Highest Salary: $' || TO_CHAR(v_max_salary, '999,999.99'));
        DBMS_OUTPUT.PUT_LINE('Lowest Salary: $' || TO_CHAR(v_min_salary, '999,999.99'));
        
    EXCEPTION
        WHEN OTHERS THEN
            DBMS_OUTPUT.PUT_LINE('Error generating report: ' || SQLERRM);
    END generate_employee_report;
    
END emp_management;
/

-- Test data setup and execution
BEGIN
    -- Create test departments
    INSERT INTO departments VALUES (10, 'Administration', NULL, 1700);
    INSERT INTO departments VALUES (20, 'Marketing', NULL, 1800);
    INSERT INTO departments VALUES (30, 'Purchasing', NULL, 1700);
    INSERT INTO departments VALUES (40, 'Human Resources', NULL, 2400);
    INSERT INTO departments VALUES (50, 'Shipping', NULL, 1500);
    
    COMMIT;
    
    DBMS_OUTPUT.PUT_LINE('=== TESTING PL/SQL EMPLOYEE MANAGEMENT SYSTEM ===');
    DBMS_OUTPUT.PUT_LINE('');
    
    -- Test adding employees
    DBMS_OUTPUT.PUT_LINE('1. Adding test employees...');
    emp_management.add_employee('John', 'Doe', 'john.doe@company.com', 'IT_PROG', 75000, NULL, 10);
    emp_management.add_employee('Jane', 'Smith', 'jane.smith@company.com', 'SA_MAN', 95000, NULL, 20);
    emp_management.add_employee('Bob', 'Johnson', 'bob.johnson@company.com', 'ST_CLERK', 45000, NULL, 50);
    emp_management.add_employee('Alice', 'Williams', 'alice.williams@company.com', 'HR_REP', 65000, NULL, 40);
    emp_management.add_employee('Charlie', 'Brown', 'charlie.brown@company.com', 'PU_CLERK', 40000, NULL, 30);
    
    -- Test employee count
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('2. Employee count tests...');
    DBMS_OUTPUT.PUT_LINE('Total employees: ' || emp_management.get_employee_count());
    DBMS_OUTPUT.PUT_LINE('Employees in dept 10: ' || emp_management.get_employee_count(10));
    DBMS_OUTPUT.PUT_LINE('Employees in dept 20: ' || emp_management.get_employee_count(20));
    
    -- Test salary updates
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('3. Testing salary updates...');
    emp_management.update_salary(1, 80000);
    emp_management.update_salary(2, NULL, 10); -- 10% raise
    
    -- Test bonus calculations
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('4. Testing bonus calculations...');
    FOR i IN 1..5 LOOP
        DBMS_OUTPUT.PUT_LINE('Employee ' || i || ' bonus: $' || 
                            TO_CHAR(emp_management.calculate_annual_bonus(i), '999,999.99'));
    END LOOP;
    
    -- Test bulk salary increase
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('5. Testing bulk salary increase for department 50...');
    emp_management.bulk_salary_increase(50, 5); -- 5% increase
    
    -- Test employee report
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('6. Generating comprehensive employee report...');
    emp_management.generate_employee_report();
    
    -- Test high salary cursor
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('7. High salary employees (>75K):');
    FOR emp_rec IN emp_management.high_salary_cursor LOOP
        DBMS_OUTPUT.PUT_LINE('ID: ' || emp_rec.employee_id || 
                            ', Name: ' || emp_rec.full_name || 
                            ', Salary: $' || TO_CHAR(emp_rec.salary, '999,999.99'));
    END LOOP;
    
    -- Test department employees array
    DECLARE
        v_dept_employees emp_management.emp_array_type;
    BEGIN
        DBMS_OUTPUT.PUT_LINE('');
        DBMS_OUTPUT.PUT_LINE('8. Testing department employee array for dept 20...');
        v_dept_employees := emp_management.get_department_employees(20);
        
        FOR i IN 1..v_dept_employees.COUNT LOOP
            DBMS_OUTPUT.PUT_LINE('Employee: ' || v_dept_employees(i).full_name || 
                               ', Salary: $' || TO_CHAR(v_dept_employees(i).salary, '999,999.99'));
        END LOOP;
    END;
    
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('=== PL/SQL TESTING COMPLETED SUCCESSFULLY ===');
    
EXCEPTION
    WHEN OTHERS THEN
        DBMS_OUTPUT.PUT_LINE('Error during testing: ' || SQLERRM);
        ROLLBACK;
END;
/

-- Additional test scenarios for edge cases and error handling
BEGIN
    DBMS_OUTPUT.PUT_LINE('');
    DBMS_OUTPUT.PUT_LINE('=== TESTING ERROR HANDLING ===');
    
    -- Test duplicate email
    BEGIN
        emp_management.add_employee('Test', 'User', 'john.doe@company.com', 'IT_PROG', 50000);
    EXCEPTION
        WHEN OTHERS THEN
            DBMS_OUTPUT.PUT_LINE('Expected error - Duplicate email: ' || SQLERRM);
    END;
    
    -- Test invalid salary
    BEGIN
        emp_management.add_employee('Test', 'User', 'test@company.com', 'IT_PROG', 5000);
    EXCEPTION
        WHEN OTHERS THEN
            DBMS_OUTPUT.PUT_LINE('Expected error - Invalid salary: ' || SQLERRM);
    END;
    
    -- Test non-existent employee
    BEGIN
        emp_management.update_salary(999, 60000);
    EXCEPTION
        WHEN OTHERS THEN
            DBMS_OUTPUT.PUT_LINE('Expected error - Employee not found: ' || SQLERRM);
    END;
    
    DBMS_OUTPUT.PUT_LINE('=== ERROR HANDLING TESTS COMPLETED ===');
END;
/