       IDENTIFICATION DIVISION.
       PROGRAM-ID. CALCULATOR.
       
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-NUM1          PIC 9(5)V99.
       01 WS-NUM2          PIC 9(5)V99.
       01 WS-RESULT        PIC 9(6)V99.
       01 WS-OPERATION     PIC X(1).
       
       PROCEDURE DIVISION.
       MAIN-PARA.
           DISPLAY "Simple COBOL Calculator for Testing".
           DISPLAY "Enter first number: " WITH NO ADVANCING.
           ACCEPT WS-NUM1.
           
           DISPLAY "Enter operation (+, -, *, /): " WITH NO ADVANCING.
           ACCEPT WS-OPERATION.
           
           DISPLAY "Enter second number: " WITH NO ADVANCING.
           ACCEPT WS-NUM2.
           
           EVALUATE WS-OPERATION
               WHEN "+"
                   ADD WS-NUM1 TO WS-NUM2 GIVING WS-RESULT
               WHEN "-"
                   SUBTRACT WS-NUM2 FROM WS-NUM1 GIVING WS-RESULT
               WHEN "*"
                   MULTIPLY WS-NUM1 BY WS-NUM2 GIVING WS-RESULT
               WHEN "/"
                   IF WS-NUM2 = 0
                       DISPLAY "Error: Division by zero"
                       GO TO END-PARA
                   ELSE
                       DIVIDE WS-NUM1 BY WS-NUM2 GIVING WS-RESULT
                   END-IF
               WHEN OTHER
                   DISPLAY "Invalid operation"
                   GO TO END-PARA
           END-EVALUATE.
           
           DISPLAY "Result: " WS-RESULT.
           
       END-PARA.
           STOP RUN.