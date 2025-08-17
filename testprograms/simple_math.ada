-- Simple Ada Mathematical Operations
-- This demonstrates basic Ada syntax for testing

with Ada.Text_IO;
with Ada.Integer_Text_IO;

procedure Simple_Math is
   
   -- Variable declarations
   A, B : Integer;
   Sum, Difference, Product : Integer;
   Quotient : Float;
   
   -- Exception handling
   Division_By_Zero : exception;
   
begin
   
   Ada.Text_IO.Put_Line("Simple Math Calculator");
   
   -- Get input values
   Ada.Text_IO.Put("Enter first number: ");
   Ada.Integer_Text_IO.Get(A);
   
   Ada.Text_IO.Put("Enter second number: ");
   Ada.Integer_Text_IO.Get(B);
   
   -- Perform calculations
   Sum := Add_Numbers(A, B);
   Difference := Subtract_Numbers(A, B);
   Product := Multiply_Numbers(A, B);
   
   -- Handle division with error checking
   if B /= 0 then
      Quotient := Divide_Numbers(A, B);
   else
      raise Division_By_Zero;
   end if;
   
   -- Display results
   Ada.Text_IO.Put_Line("Sum: " & Integer'Image(Sum));
   Ada.Text_IO.Put_Line("Difference: " & Integer'Image(Difference));
   Ada.Text_IO.Put_Line("Product: " & Integer'Image(Product));
   Ada.Text_IO.Put_Line("Quotient: " & Float'Image(Quotient));
   
exception
   when Division_By_Zero =>
      Ada.Text_IO.Put_Line("Error: Division by zero is not allowed");
   when others =>
      Ada.Text_IO.Put_Line("An unexpected error occurred");
   
end Simple_Math;

-- Function definitions
function Add_Numbers(X, Y : Integer) return Integer is
begin
   return X + Y;
end Add_Numbers;

function Subtract_Numbers(X, Y : Integer) return Integer is
begin
   return X - Y;
end Subtract_Numbers;

function Multiply_Numbers(X, Y : Integer) return Integer is
begin
   return X * Y;
end Multiply_Numbers;

function Divide_Numbers(X, Y : Integer) return Float is
begin
   if Y = 0 then
      raise Constraint_Error;
   end if;
   return Float(X) / Float(Y);
end Divide_Numbers;