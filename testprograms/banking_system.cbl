      *****************************************************************
      * BANKING SYSTEM - ACCOUNT MANAGEMENT
      * This COBOL program demonstrates banking operations for testing
      *****************************************************************
       IDENTIFICATION DIVISION.
       PROGRAM-ID. BANKING-SYSTEM.
       AUTHOR. LEGACY-SYSTEMS-TEAM.

       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT ACCOUNT-FILE ASSIGN TO 'ACCOUNTS.DAT'
           ORGANIZATION IS INDEXED
           ACCESS MODE IS DYNAMIC
           RECORD KEY IS ACCOUNT-NUMBER.

       DATA DIVISION.
       FILE SECTION.
       FD ACCOUNT-FILE.
       01 ACCOUNT-RECORD.
           05 ACCOUNT-NUMBER      PIC 9(10).
           05 CUSTOMER-NAME       PIC X(30).
           05 ACCOUNT-BALANCE     PIC 9(10)V99.
           05 ACCOUNT-STATUS      PIC X(1).
              88 ACTIVE-ACCOUNT   VALUE 'A'.
              88 CLOSED-ACCOUNT   VALUE 'C'.
              88 FROZEN-ACCOUNT   VALUE 'F'.

       WORKING-STORAGE SECTION.
       01 WS-WORK-AREAS.
           05 WS-ACCOUNT-NUMBER   PIC 9(10).
           05 WS-TRANSACTION-AMT  PIC 9(8)V99.
           05 WS-NEW-BALANCE      PIC 9(10)V99.
           05 WS-RETURN-CODE      PIC 9(2).
           05 WS-ERROR-FLAG       PIC X(1).
              88 NO-ERROR         VALUE 'N'.
              88 ERROR-FOUND      VALUE 'Y'.

       01 WS-CONSTANTS.
           05 MIN-BALANCE         PIC 9(5)V99 VALUE 100.00.
           05 MAX-WITHDRAWAL      PIC 9(6)V99 VALUE 5000.00.
           05 OVERDRAFT-LIMIT     PIC 9(4)V99 VALUE 500.00.

       PROCEDURE DIVISION.

       MAIN-PROCESSING.
           PERFORM INITIALIZE-PROGRAM
           PERFORM PROCESS-TRANSACTIONS
           PERFORM CLEANUP-PROGRAM
           STOP RUN.

       INITIALIZE-PROGRAM.
           OPEN I-O ACCOUNT-FILE
           SET NO-ERROR TO TRUE
           MOVE ZEROS TO WS-RETURN-CODE.

       PROCESS-TRANSACTIONS.
           PERFORM DEPOSIT-FUNDS
           PERFORM WITHDRAW-FUNDS
           PERFORM CHECK-BALANCE
           PERFORM CLOSE-ACCOUNT.

       DEPOSIT-FUNDS.
           MOVE 123456789 TO WS-ACCOUNT-NUMBER
           MOVE 500.00 TO WS-TRANSACTION-AMT
           PERFORM READ-ACCOUNT-RECORD
           IF NO-ERROR
               IF ACTIVE-ACCOUNT
                   ADD WS-TRANSACTION-AMT TO ACCOUNT-BALANCE
                   PERFORM UPDATE-ACCOUNT-RECORD
               ELSE
                   SET ERROR-FOUND TO TRUE
                   MOVE 01 TO WS-RETURN-CODE
               END-IF
           END-IF.

       WITHDRAW-FUNDS.
           MOVE 123456789 TO WS-ACCOUNT-NUMBER  
           MOVE 200.00 TO WS-TRANSACTION-AMT
           PERFORM READ-ACCOUNT-RECORD
           IF NO-ERROR
               IF ACTIVE-ACCOUNT
                   COMPUTE WS-NEW-BALANCE = 
                       ACCOUNT-BALANCE - WS-TRANSACTION-AMT
                   IF WS-NEW-BALANCE >= (MIN-BALANCE * -1)
                       MOVE WS-NEW-BALANCE TO ACCOUNT-BALANCE
                       PERFORM UPDATE-ACCOUNT-RECORD
                   ELSE
                       SET ERROR-FOUND TO TRUE
                       MOVE 02 TO WS-RETURN-CODE
                   END-IF
               ELSE
                   SET ERROR-FOUND TO TRUE
                   MOVE 03 TO WS-RETURN-CODE
               END-IF
           END-IF.

       CHECK-BALANCE.
           MOVE 123456789 TO WS-ACCOUNT-NUMBER
           PERFORM READ-ACCOUNT-RECORD
           IF NO-ERROR
               DISPLAY 'Account Balance: ' ACCOUNT-BALANCE
           ELSE
               DISPLAY 'Account not found'
           END-IF.

       CLOSE-ACCOUNT.
           MOVE 123456789 TO WS-ACCOUNT-NUMBER
           PERFORM READ-ACCOUNT-RECORD
           IF NO-ERROR
               IF ACCOUNT-BALANCE = ZERO
                   MOVE 'C' TO ACCOUNT-STATUS
                   PERFORM UPDATE-ACCOUNT-RECORD
               ELSE
                   SET ERROR-FOUND TO TRUE
                   MOVE 04 TO WS-RETURN-CODE
               END-IF
           END-IF.

       READ-ACCOUNT-RECORD.
           MOVE WS-ACCOUNT-NUMBER TO ACCOUNT-NUMBER
           READ ACCOUNT-FILE
               INVALID KEY
                   SET ERROR-FOUND TO TRUE
                   MOVE 99 TO WS-RETURN-CODE
               NOT INVALID KEY
                   SET NO-ERROR TO TRUE
           END-READ.

       UPDATE-ACCOUNT-RECORD.
           REWRITE ACCOUNT-RECORD
               INVALID KEY
                   SET ERROR-FOUND TO TRUE
                   MOVE 98 TO WS-RETURN-CODE
               NOT INVALID KEY
                   SET NO-ERROR TO TRUE
           END-REWRITE.

       CLEANUP-PROGRAM.
           CLOSE ACCOUNT-FILE.