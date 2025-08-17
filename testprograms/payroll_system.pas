program PayrollSystem;

{$mode objfpc}{$H+}

uses
  Classes, SysUtils;

type
  { Employee record structure }
  TEmployee = record
    ID: Integer;
    Name: String;
    HourlyRate: Real;
    HoursWorked: Real;
    Overtime: Real;
    GrossPay: Real;
    Tax: Real;
    NetPay: Real;
  end;

var
  Employees: array[1..100] of TEmployee;
  EmployeeCount: Integer;

{ Calculate gross pay including overtime }
function CalculateGrossPay(HourlyRate, HoursWorked, Overtime: Real): Real;
var
  RegularPay, OvertimePay: Real;
begin
  RegularPay := HourlyRate * HoursWorked;
  if Overtime > 0 then
    OvertimePay := HourlyRate * 1.5 * Overtime
  else
    OvertimePay := 0;
  
  CalculateGrossPay := RegularPay + OvertimePay;
end;

{ Calculate tax based on gross pay }
function CalculateTax(GrossPay: Real): Real;
begin
  if GrossPay <= 500 then
    CalculateTax := GrossPay * 0.10
  else if GrossPay <= 1000 then
    CalculateTax := 50 + (GrossPay - 500) * 0.15
  else if GrossPay <= 2000 then
    CalculateTax := 125 + (GrossPay - 1000) * 0.20
  else
    CalculateTax := 325 + (GrossPay - 2000) * 0.25;
end;

{ Calculate net pay after taxes }
function CalculateNetPay(GrossPay, Tax: Real): Real;
begin
  CalculateNetPay := GrossPay - Tax;
end;

{ Validate employee data }
function ValidateEmployee(var Employee: TEmployee): Boolean;
begin
  ValidateEmployee := True;
  
  if Employee.ID <= 0 then
  begin
    WriteLn('Error: Invalid employee ID');
    ValidateEmployee := False;
  end;
  
  if Length(Employee.Name) = 0 then
  begin
    WriteLn('Error: Employee name cannot be empty');
    ValidateEmployee := False;
  end;
  
  if Employee.HourlyRate <= 0 then
  begin
    WriteLn('Error: Hourly rate must be positive');
    ValidateEmployee := False;
  end;
  
  if Employee.HoursWorked < 0 then
  begin
    WriteLn('Error: Hours worked cannot be negative');
    ValidateEmployee := False;
  end;
  
  if Employee.Overtime < 0 then
  begin
    WriteLn('Error: Overtime hours cannot be negative');
    ValidateEmployee := False;
  end;
end;

{ Add new employee to the system }
procedure AddEmployee(ID: Integer; Name: String; HourlyRate, HoursWorked, Overtime: Real);
var
  NewEmployee: TEmployee;
begin
  if EmployeeCount >= 100 then
  begin
    WriteLn('Error: Maximum number of employees reached');
    Exit;
  end;
  
  NewEmployee.ID := ID;
  NewEmployee.Name := Name;
  NewEmployee.HourlyRate := HourlyRate;
  NewEmployee.HoursWorked := HoursWorked;
  NewEmployee.Overtime := Overtime;
  
  if ValidateEmployee(NewEmployee) then
  begin
    Inc(EmployeeCount);
    Employees[EmployeeCount] := NewEmployee;
    WriteLn('Employee ', Name, ' added successfully');
  end;
end;

{ Process payroll for a specific employee }
procedure ProcessEmployeePayroll(var Employee: TEmployee);
begin
  Employee.GrossPay := CalculateGrossPay(Employee.HourlyRate, Employee.HoursWorked, Employee.Overtime);
  Employee.Tax := CalculateTax(Employee.GrossPay);
  Employee.NetPay := CalculateNetPay(Employee.GrossPay, Employee.Tax);
end;

{ Process payroll for all employees }
procedure ProcessAllPayroll;
var
  i: Integer;
begin
  WriteLn('Processing payroll for all employees...');
  
  for i := 1 to EmployeeCount do
  begin
    ProcessEmployeePayroll(Employees[i]);
    WriteLn('Processed payroll for: ', Employees[i].Name);
  end;
  
  WriteLn('Payroll processing completed for ', EmployeeCount, ' employees');
end;

{ Find employee by ID }
function FindEmployeeByID(ID: Integer): Integer;
var
  i: Integer;
begin
  FindEmployeeByID := -1;
  
  for i := 1 to EmployeeCount do
  begin
    if Employees[i].ID = ID then
    begin
      FindEmployeeByID := i;
      Exit;
    end;
  end;
end;

{ Generate payroll report }
procedure GeneratePayrollReport;
var
  i: Integer;
  TotalGross, TotalTax, TotalNet: Real;
begin
  WriteLn;
  WriteLn('=== PAYROLL REPORT ===');
  WriteLn('ID', #9, 'Name', #9#9, 'Hours', #9, 'Rate', #9, 'Gross', #9, 'Tax', #9, 'Net');
  WriteLn('------------------------------------------------------------');
  
  TotalGross := 0;
  TotalTax := 0;
  TotalNet := 0;
  
  for i := 1 to EmployeeCount do
  begin
    with Employees[i] do
    begin
      WriteLn(ID:2, #9, Name:10, #9, HoursWorked:5:1, #9, HourlyRate:5:2, #9, 
              GrossPay:7:2, #9, Tax:6:2, #9, NetPay:7:2);
      
      TotalGross := TotalGross + GrossPay;
      TotalTax := TotalTax + Tax;
      TotalNet := TotalNet + NetPay;
    end;
  end;
  
  WriteLn('------------------------------------------------------------');
  WriteLn('TOTALS:', #9#9#9#9, TotalGross:7:2, #9, TotalTax:6:2, #9, TotalNet:7:2);
  WriteLn;
end;

{ Main program }
begin
  EmployeeCount := 0;
  
  WriteLn('=== Payroll System ===');
  WriteLn;
  
  { Add sample employees }
  AddEmployee(101, 'John Smith', 15.50, 40, 5);
  AddEmployee(102, 'Jane Doe', 18.00, 35, 0);
  AddEmployee(103, 'Bob Wilson', 22.75, 40, 10);
  AddEmployee(104, 'Alice Johnson', 12.00, 30, 2);
  
  { Process payroll }
  ProcessAllPayroll;
  
  { Generate report }
  GeneratePayrollReport;
  
  WriteLn('Press Enter to exit...');
  ReadLn;
end.