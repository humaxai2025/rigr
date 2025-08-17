/**
 * Comprehensive Financial Management System in C
 * 
 * This program demonstrates various C programming concepts including:
 * - Memory management with malloc/free
 * - Structs and unions for complex data types
 * - File I/O operations
 * - Pointer manipulation and function pointers
 * - String handling and parsing
 * - Error handling and validation
 * - Linked lists and dynamic data structures
 * - Sorting and searching algorithms
 * - Time/date handling
 * - Financial calculations with precision
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <math.h>
#include <assert.h>
#include <errno.h>
#include <ctype.h>

#define MAX_NAME_LENGTH 100
#define MAX_DESCRIPTION_LENGTH 256
#define MAX_ACCOUNTS 1000
#define MAX_TRANSACTIONS 10000
#define DATA_FILE "financial_data.dat"
#define BACKUP_FILE "financial_backup.dat"
#define PRECISION 0.01

// Error codes
typedef enum {
    SUCCESS = 0,
    ERROR_MEMORY_ALLOCATION = -1,
    ERROR_INVALID_ACCOUNT = -2,
    ERROR_INSUFFICIENT_FUNDS = -3,
    ERROR_FILE_OPERATION = -4,
    ERROR_INVALID_INPUT = -5,
    ERROR_ACCOUNT_EXISTS = -6,
    ERROR_ACCOUNT_NOT_FOUND = -7
} ErrorCode;

// Account types
typedef enum {
    ACCOUNT_CHECKING = 0,
    ACCOUNT_SAVINGS,
    ACCOUNT_INVESTMENT,
    ACCOUNT_CREDIT,
    ACCOUNT_LOAN
} AccountType;

// Transaction types
typedef enum {
    TRANSACTION_DEPOSIT = 0,
    TRANSACTION_WITHDRAWAL,
    TRANSACTION_TRANSFER,
    TRANSACTION_INTEREST,
    TRANSACTION_FEE,
    TRANSACTION_PAYMENT
} TransactionType;

// Account structure
typedef struct {
    int account_id;
    char account_name[MAX_NAME_LENGTH];
    AccountType type;
    double balance;
    double interest_rate;
    double credit_limit;
    time_t created_date;
    time_t last_modified;
    int is_active;
} Account;

// Transaction structure
typedef struct Transaction {
    int transaction_id;
    int from_account_id;
    int to_account_id;
    TransactionType type;
    double amount;
    char description[MAX_DESCRIPTION_LENGTH];
    time_t timestamp;
    struct Transaction* next;
} Transaction;

// Financial system structure
typedef struct {
    Account accounts[MAX_ACCOUNTS];
    int account_count;
    Transaction* transaction_head;
    int next_transaction_id;
    int next_account_id;
    double total_assets;
    double total_liabilities;
} FinancialSystem;

// Function pointer type for transaction validators
typedef ErrorCode (*TransactionValidator)(const FinancialSystem* system, int account_id, double amount);

// Global system instance
static FinancialSystem* g_financial_system = NULL;

// Utility functions
const char* get_account_type_string(AccountType type) {
    static const char* type_strings[] = {
        "Checking", "Savings", "Investment", "Credit", "Loan"
    };
    return (type >= 0 && type <= ACCOUNT_LOAN) ? type_strings[type] : "Unknown";
}

const char* get_transaction_type_string(TransactionType type) {
    static const char* type_strings[] = {
        "Deposit", "Withdrawal", "Transfer", "Interest", "Fee", "Payment"
    };
    return (type >= 0 && type <= TRANSACTION_PAYMENT) ? type_strings[type] : "Unknown";
}

const char* get_error_string(ErrorCode error) {
    static const char* error_strings[] = {
        "Success",
        "Memory allocation failed",
        "Invalid account",
        "Insufficient funds",
        "File operation failed",
        "Invalid input",
        "Account already exists",
        "Account not found"
    };
    int index = -error;
    return (index >= 0 && index < 8) ? error_strings[index] : "Unknown error";
}

// Memory management functions
FinancialSystem* create_financial_system() {
    FinancialSystem* system = (FinancialSystem*)malloc(sizeof(FinancialSystem));
    if (!system) {
        return NULL;
    }
    
    memset(system, 0, sizeof(FinancialSystem));
    system->next_account_id = 1001; // Start with 4-digit account numbers
    system->next_transaction_id = 1;
    system->transaction_head = NULL;
    
    return system;
}

void destroy_financial_system(FinancialSystem* system) {
    if (!system) return;
    
    // Free all transactions
    Transaction* current = system->transaction_head;
    while (current) {
        Transaction* next = current->next;
        free(current);
        current = next;
    }
    
    free(system);
}

// Account management functions
ErrorCode add_account(FinancialSystem* system, const char* name, AccountType type, 
                     double initial_balance, double interest_rate, double credit_limit) {
    if (!system || !name || system->account_count >= MAX_ACCOUNTS) {
        return ERROR_INVALID_INPUT;
    }
    
    // Check if account name already exists
    for (int i = 0; i < system->account_count; i++) {
        if (strcmp(system->accounts[i].account_name, name) == 0 && system->accounts[i].is_active) {
            return ERROR_ACCOUNT_EXISTS;
        }
    }
    
    Account* account = &system->accounts[system->account_count];
    account->account_id = system->next_account_id++;
    strncpy(account->account_name, name, MAX_NAME_LENGTH - 1);
    account->account_name[MAX_NAME_LENGTH - 1] = '\0';
    account->type = type;
    account->balance = initial_balance;
    account->interest_rate = interest_rate;
    account->credit_limit = credit_limit;
    account->created_date = time(NULL);
    account->last_modified = time(NULL);
    account->is_active = 1;
    
    system->account_count++;
    
    // Update system totals
    if (initial_balance > 0) {
        system->total_assets += initial_balance;
    } else {
        system->total_liabilities += -initial_balance;
    }
    
    return SUCCESS;
}

Account* find_account_by_id(const FinancialSystem* system, int account_id) {
    if (!system) return NULL;
    
    for (int i = 0; i < system->account_count; i++) {
        if (system->accounts[i].account_id == account_id && system->accounts[i].is_active) {
            return &system->accounts[i];
        }
    }
    return NULL;
}

Account* find_account_by_name(const FinancialSystem* system, const char* name) {
    if (!system || !name) return NULL;
    
    for (int i = 0; i < system->account_count; i++) {
        if (strcmp(system->accounts[i].account_name, name) == 0 && system->accounts[i].is_active) {
            return &system->accounts[i];
        }
    }
    return NULL;
}

ErrorCode close_account(FinancialSystem* system, int account_id) {
    if (!system) return ERROR_INVALID_INPUT;
    
    Account* account = find_account_by_id(system, account_id);
    if (!account) return ERROR_ACCOUNT_NOT_FOUND;
    
    if (fabs(account->balance) > PRECISION) {
        return ERROR_INVALID_INPUT; // Cannot close account with non-zero balance
    }
    
    account->is_active = 0;
    account->last_modified = time(NULL);
    
    return SUCCESS;
}

// Transaction validation functions
ErrorCode validate_deposit(const FinancialSystem* system, int account_id, double amount) {
    if (amount <= 0) return ERROR_INVALID_INPUT;
    Account* account = find_account_by_id(system, account_id);
    return account ? SUCCESS : ERROR_ACCOUNT_NOT_FOUND;
}

ErrorCode validate_withdrawal(const FinancialSystem* system, int account_id, double amount) {
    if (amount <= 0) return ERROR_INVALID_INPUT;
    
    Account* account = find_account_by_id(system, account_id);
    if (!account) return ERROR_ACCOUNT_NOT_FOUND;
    
    double available_balance = account->balance;
    if (account->type == ACCOUNT_CREDIT) {
        available_balance += account->credit_limit;
    }
    
    return (amount <= available_balance) ? SUCCESS : ERROR_INSUFFICIENT_FUNDS;
}

ErrorCode validate_transfer(const FinancialSystem* system, int from_account_id, double amount) {
    return validate_withdrawal(system, from_account_id, amount);
}

// Transaction management functions
ErrorCode add_transaction(FinancialSystem* system, int from_account_id, int to_account_id, 
                         TransactionType type, double amount, const char* description) {
    if (!system || amount <= 0) return ERROR_INVALID_INPUT;
    
    // Validate transaction based on type
    TransactionValidator validators[] = {
        validate_deposit,    // TRANSACTION_DEPOSIT
        validate_withdrawal, // TRANSACTION_WITHDRAWAL
        validate_transfer,   // TRANSACTION_TRANSFER
        validate_deposit,    // TRANSACTION_INTEREST
        validate_withdrawal, // TRANSACTION_FEE
        validate_withdrawal  // TRANSACTION_PAYMENT
    };
    
    if (type >= 0 && type <= TRANSACTION_PAYMENT) {
        ErrorCode validation_result = validators[type](system, from_account_id, amount);
        if (validation_result != SUCCESS) {
            return validation_result;
        }
    }
    
    // Create new transaction
    Transaction* transaction = (Transaction*)malloc(sizeof(Transaction));
    if (!transaction) return ERROR_MEMORY_ALLOCATION;
    
    transaction->transaction_id = system->next_transaction_id++;
    transaction->from_account_id = from_account_id;
    transaction->to_account_id = to_account_id;
    transaction->type = type;
    transaction->amount = amount;
    strncpy(transaction->description, description ? description : "", MAX_DESCRIPTION_LENGTH - 1);
    transaction->description[MAX_DESCRIPTION_LENGTH - 1] = '\0';
    transaction->timestamp = time(NULL);
    transaction->next = system->transaction_head;
    system->transaction_head = transaction;
    
    // Update account balances
    Account* from_account = find_account_by_id(system, from_account_id);
    Account* to_account = find_account_by_id(system, to_account_id);
    
    switch (type) {
        case TRANSACTION_DEPOSIT:
            if (to_account) {
                to_account->balance += amount;
                to_account->last_modified = time(NULL);
                system->total_assets += amount;
            }
            break;
            
        case TRANSACTION_WITHDRAWAL:
        case TRANSACTION_FEE:
        case TRANSACTION_PAYMENT:
            if (from_account) {
                from_account->balance -= amount;
                from_account->last_modified = time(NULL);
                system->total_assets -= amount;
            }
            break;
            
        case TRANSACTION_TRANSFER:
            if (from_account && to_account) {
                from_account->balance -= amount;
                to_account->balance += amount;
                from_account->last_modified = time(NULL);
                to_account->last_modified = time(NULL);
            }
            break;
            
        case TRANSACTION_INTEREST:
            if (to_account) {
                double interest = to_account->balance * to_account->interest_rate / 100.0 / 365.0;
                to_account->balance += interest;
                to_account->last_modified = time(NULL);
                system->total_assets += interest;
            }
            break;
    }
    
    return SUCCESS;
}

// Financial calculations
double calculate_compound_interest(double principal, double rate, double time, int compound_frequency) {
    if (compound_frequency <= 0) compound_frequency = 1;
    return principal * pow(1.0 + rate / (100.0 * compound_frequency), compound_frequency * time);
}

double calculate_loan_payment(double principal, double annual_rate, int years) {
    if (annual_rate == 0) return principal / (years * 12);
    
    double monthly_rate = annual_rate / 100.0 / 12.0;
    int total_payments = years * 12;
    
    return (principal * monthly_rate * pow(1 + monthly_rate, total_payments)) / 
           (pow(1 + monthly_rate, total_payments) - 1);
}

double calculate_account_value_at_date(const FinancialSystem* system, int account_id, time_t target_date) {
    Account* account = find_account_by_id(system, account_id);
    if (!account) return 0.0;
    
    double value = 0.0;
    
    // Calculate value by replaying transactions up to target date
    Transaction* current = system->transaction_head;
    while (current) {
        if (current->timestamp <= target_date) {
            if (current->to_account_id == account_id) {
                value += current->amount;
            }
            if (current->from_account_id == account_id) {
                value -= current->amount;
            }
        }
        current = current->next;
    }
    
    return value;
}

// Reporting functions
void print_account_summary(const Account* account) {
    if (!account) return;
    
    printf("\n--- Account Summary ---\n");
    printf("Account ID: %d\n", account->account_id);
    printf("Name: %s\n", account->account_name);
    printf("Type: %s\n", get_account_type_string(account->type));
    printf("Balance: $%.2f\n", account->balance);
    printf("Interest Rate: %.2f%%\n", account->interest_rate);
    
    if (account->type == ACCOUNT_CREDIT) {
        printf("Credit Limit: $%.2f\n", account->credit_limit);
        printf("Available Credit: $%.2f\n", account->credit_limit + account->balance);
    }
    
    printf("Created: %s", ctime(&account->created_date));
    printf("Status: %s\n", account->is_active ? "Active" : "Closed");
}

void print_transaction_history(const FinancialSystem* system, int account_id, int limit) {
    if (!system) return;
    
    printf("\n--- Transaction History ---\n");
    printf("Account ID: %d\n", account_id);
    printf("%-6s %-12s %-10s %-10s %-15s %s\n", 
           "ID", "Type", "Amount", "Account", "Date", "Description");
    printf("---------------------------------------------------------------------\n");
    
    int count = 0;
    Transaction* current = system->transaction_head;
    
    while (current && (limit <= 0 || count < limit)) {
        if (current->from_account_id == account_id || current->to_account_id == account_id) {
            char date_str[26];
            struct tm* tm_info = localtime(&current->timestamp);
            strftime(date_str, 26, "%Y-%m-%d %H:%M", tm_info);
            
            int other_account = (current->from_account_id == account_id) ? 
                              current->to_account_id : current->from_account_id;
            
            printf("%-6d %-12s $%-9.2f %-10d %-15s %s\n",
                   current->transaction_id,
                   get_transaction_type_string(current->type),
                   current->amount,
                   other_account,
                   date_str,
                   current->description);
            count++;
        }
        current = current->next;
    }
}

void print_system_summary(const FinancialSystem* system) {
    if (!system) return;
    
    printf("\n=== Financial System Summary ===\n");
    printf("Total Accounts: %d\n", system->account_count);
    printf("Total Assets: $%.2f\n", system->total_assets);
    printf("Total Liabilities: $%.2f\n", system->total_liabilities);
    printf("Net Worth: $%.2f\n", system->total_assets - system->total_liabilities);
    
    // Count transactions
    int transaction_count = 0;
    Transaction* current = system->transaction_head;
    while (current) {
        transaction_count++;
        current = current->next;
    }
    printf("Total Transactions: %d\n", transaction_count);
    
    printf("\n--- Account Breakdown by Type ---\n");
    int type_counts[5] = {0};
    double type_balances[5] = {0.0};
    
    for (int i = 0; i < system->account_count; i++) {
        if (system->accounts[i].is_active) {
            AccountType type = system->accounts[i].type;
            type_counts[type]++;
            type_balances[type] += system->accounts[i].balance;
        }
    }
    
    for (int i = 0; i < 5; i++) {
        if (type_counts[i] > 0) {
            printf("%s: %d accounts, Total: $%.2f\n", 
                   get_account_type_string((AccountType)i), 
                   type_counts[i], 
                   type_balances[i]);
        }
    }
}

// File I/O functions
ErrorCode save_system_to_file(const FinancialSystem* system, const char* filename) {
    if (!system || !filename) return ERROR_INVALID_INPUT;
    
    FILE* file = fopen(filename, "wb");
    if (!file) return ERROR_FILE_OPERATION;
    
    // Write system metadata
    fwrite(&system->account_count, sizeof(int), 1, file);
    fwrite(&system->next_account_id, sizeof(int), 1, file);
    fwrite(&system->next_transaction_id, sizeof(int), 1, file);
    fwrite(&system->total_assets, sizeof(double), 1, file);
    fwrite(&system->total_liabilities, sizeof(double), 1, file);
    
    // Write accounts
    fwrite(system->accounts, sizeof(Account), system->account_count, file);
    
    // Count and write transactions
    int transaction_count = 0;
    Transaction* current = system->transaction_head;
    while (current) {
        transaction_count++;
        current = current->next;
    }
    
    fwrite(&transaction_count, sizeof(int), 1, file);
    
    current = system->transaction_head;
    while (current) {
        fwrite(current, sizeof(Transaction) - sizeof(Transaction*), 1, file);
        current = current->next;
    }
    
    fclose(file);
    return SUCCESS;
}

ErrorCode load_system_from_file(FinancialSystem* system, const char* filename) {
    if (!system || !filename) return ERROR_INVALID_INPUT;
    
    FILE* file = fopen(filename, "rb");
    if (!file) return ERROR_FILE_OPERATION;
    
    // Clear existing transaction list
    Transaction* current = system->transaction_head;
    while (current) {
        Transaction* next = current->next;
        free(current);
        current = next;
    }
    system->transaction_head = NULL;
    
    // Read system metadata
    fread(&system->account_count, sizeof(int), 1, file);
    fread(&system->next_account_id, sizeof(int), 1, file);
    fread(&system->next_transaction_id, sizeof(int), 1, file);
    fread(&system->total_assets, sizeof(double), 1, file);
    fread(&system->total_liabilities, sizeof(double), 1, file);
    
    // Read accounts
    fread(system->accounts, sizeof(Account), system->account_count, file);
    
    // Read transactions
    int transaction_count;
    fread(&transaction_count, sizeof(int), 1, file);
    
    Transaction* prev = NULL;
    for (int i = 0; i < transaction_count; i++) {
        Transaction* transaction = (Transaction*)malloc(sizeof(Transaction));
        if (!transaction) {
            fclose(file);
            return ERROR_MEMORY_ALLOCATION;
        }
        
        fread(transaction, sizeof(Transaction) - sizeof(Transaction*), 1, file);
        transaction->next = NULL;
        
        if (prev) {
            prev->next = transaction;
        } else {
            system->transaction_head = transaction;
        }
        prev = transaction;
    }
    
    fclose(file);
    return SUCCESS;
}

// Utility functions for data processing
void sort_accounts_by_balance(Account* accounts, int count, int descending) {
    for (int i = 0; i < count - 1; i++) {
        for (int j = 0; j < count - i - 1; j++) {
            int should_swap = descending ? 
                (accounts[j].balance < accounts[j + 1].balance) :
                (accounts[j].balance > accounts[j + 1].balance);
                
            if (should_swap) {
                Account temp = accounts[j];
                accounts[j] = accounts[j + 1];
                accounts[j + 1] = temp;
            }
        }
    }
}

int compare_transactions_by_date(const void* a, const void* b) {
    const Transaction* ta = (const Transaction*)a;
    const Transaction* tb = (const Transaction*)b;
    return (int)(tb->timestamp - ta->timestamp); // Most recent first
}

// Main demonstration program
int main(int argc, char* argv[]) {
    printf("=== Comprehensive Financial Management System ===\n");
    printf("Initializing system...\n");
    
    // Create financial system
    g_financial_system = create_financial_system();
    if (!g_financial_system) {
        fprintf(stderr, "Failed to create financial system\n");
        return 1;
    }
    
    // Try to load existing data
    if (load_system_from_file(g_financial_system, DATA_FILE) == SUCCESS) {
        printf("Loaded existing financial data\n");
    } else {
        printf("Creating new financial system\n");
        
        // Create sample accounts
        printf("\n1. Creating sample accounts...\n");
        
        ErrorCode result;
        result = add_account(g_financial_system, "Primary Checking", ACCOUNT_CHECKING, 5000.00, 0.1, 0.0);
        printf("Added Primary Checking: %s\n", get_error_string(result));
        
        result = add_account(g_financial_system, "Emergency Savings", ACCOUNT_SAVINGS, 15000.00, 2.5, 0.0);
        printf("Added Emergency Savings: %s\n", get_error_string(result));
        
        result = add_account(g_financial_system, "Investment Portfolio", ACCOUNT_INVESTMENT, 50000.00, 7.2, 0.0);
        printf("Added Investment Portfolio: %s\n", get_error_string(result));
        
        result = add_account(g_financial_system, "Credit Card", ACCOUNT_CREDIT, -2500.00, 18.9, 10000.0);
        printf("Added Credit Card: %s\n", get_error_string(result));
        
        result = add_account(g_financial_system, "Home Mortgage", ACCOUNT_LOAN, -250000.00, 4.5, 0.0);
        printf("Added Home Mortgage: %s\n", get_error_string(result));
    }
    
    // Demonstrate transactions
    printf("\n2. Processing sample transactions...\n");
    
    Account* checking = find_account_by_name(g_financial_system, "Primary Checking");
    Account* savings = find_account_by_name(g_financial_system, "Emergency Savings");
    Account* credit = find_account_by_name(g_financial_system, "Credit Card");
    
    if (checking && savings && credit) {
        // Salary deposit
        add_transaction(g_financial_system, 0, checking->account_id, 
                       TRANSACTION_DEPOSIT, 3500.00, "Monthly Salary");
        
        // Transfer to savings
        add_transaction(g_financial_system, checking->account_id, savings->account_id,
                       TRANSACTION_TRANSFER, 1000.00, "Monthly Savings");
        
        // Pay credit card
        add_transaction(g_financial_system, checking->account_id, credit->account_id,
                       TRANSACTION_PAYMENT, 500.00, "Credit Card Payment");
        
        // ATM withdrawal
        add_transaction(g_financial_system, checking->account_id, 0,
                       TRANSACTION_WITHDRAWAL, 200.00, "ATM Withdrawal");
        
        // Bank fee
        add_transaction(g_financial_system, checking->account_id, 0,
                       TRANSACTION_FEE, 25.00, "Monthly Maintenance Fee");
        
        printf("Processed 5 sample transactions\n");
    }
    
    // Display system summary
    print_system_summary(g_financial_system);
    
    // Display individual account summaries
    printf("\n3. Account Details:\n");
    for (int i = 0; i < g_financial_system->account_count; i++) {
        if (g_financial_system->accounts[i].is_active) {
            print_account_summary(&g_financial_system->accounts[i]);
        }
    }
    
    // Show transaction history
    if (checking) {
        print_transaction_history(g_financial_system, checking->account_id, 10);
    }
    
    // Demonstrate financial calculations
    printf("\n4. Financial Calculations:\n");
    
    double future_value = calculate_compound_interest(10000.0, 5.0, 10.0, 12);
    printf("$10,000 at 5%% interest for 10 years: $%.2f\n", future_value);
    
    double monthly_payment = calculate_loan_payment(250000.0, 4.5, 30);
    printf("Monthly payment for $250,000 loan at 4.5%% for 30 years: $%.2f\n", monthly_payment);
    
    // Test error handling
    printf("\n5. Testing error handling...\n");
    
    ErrorCode error = add_transaction(g_financial_system, checking ? checking->account_id : 0, 0,
                                    TRANSACTION_WITHDRAWAL, 100000.00, "Large Withdrawal");
    printf("Large withdrawal attempt: %s\n", get_error_string(error));
    
    error = add_account(g_financial_system, "Primary Checking", ACCOUNT_CHECKING, 0.0, 0.0, 0.0);
    printf("Duplicate account creation: %s\n", get_error_string(error));
    
    // Save system state
    printf("\n6. Saving system data...\n");
    error = save_system_to_file(g_financial_system, DATA_FILE);
    printf("Save to file: %s\n", get_error_string(error));
    
    // Create backup
    error = save_system_to_file(g_financial_system, BACKUP_FILE);
    printf("Create backup: %s\n", get_error_string(error));
    
    // Performance test
    printf("\n7. Performance testing...\n");
    clock_t start = clock();
    
    for (int i = 0; i < 1000; i++) {
        char account_name[50];
        sprintf(account_name, "Test Account %d", i);
        add_account(g_financial_system, account_name, ACCOUNT_CHECKING, 1000.0, 1.0, 0.0);
    }
    
    clock_t end = clock();
    double cpu_time = ((double)(end - start)) / CLOCKS_PER_SEC;
    printf("Created 1000 accounts in %.3f seconds\n", cpu_time);
    
    // Memory usage information
    printf("\n8. Memory Usage:\n");
    printf("System structure: %zu bytes\n", sizeof(FinancialSystem));
    printf("Account structure: %zu bytes\n", sizeof(Account));
    printf("Transaction structure: %zu bytes\n", sizeof(Transaction));
    
    // Cleanup
    printf("\nCleaning up and exiting...\n");
    destroy_financial_system(g_financial_system);
    g_financial_system = NULL;
    
    printf("Financial Management System demonstration completed successfully!\n");
    printf("\nKey C features demonstrated:\n");
    printf("- Dynamic memory allocation and management\n");
    printf("- Complex data structures (structs, unions, linked lists)\n");
    printf("- File I/O operations with binary data\n");
    printf("- Pointer manipulation and function pointers\n");
    printf("- String handling and parsing\n");
    printf("- Error handling with custom error codes\n");
    printf("- Mathematical calculations and precision handling\n");
    printf("- Time/date operations\n");
    printf("- Sorting and searching algorithms\n");
    printf("- Performance measurement\n");
    
    return 0;
}