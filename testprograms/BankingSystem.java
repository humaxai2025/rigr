// Complex Banking System - Java
// This demonstrates advanced Java patterns for comprehensive test case generation

package banking;

import java.time.LocalDateTime;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.locks.ReentrantLock;
import java.math.BigDecimal;
import java.math.RoundingMode;

public class BankingSystem {
    
    public enum AccountType {
        CHECKING, SAVINGS, CREDIT, INVESTMENT, BUSINESS
    }
    
    public enum TransactionType {
        DEPOSIT, WITHDRAWAL, TRANSFER, PAYMENT, INTEREST, FEE, REFUND
    }
    
    public enum TransactionStatus {
        PENDING, COMPLETED, FAILED, CANCELLED, REVERSED
    }
    
    public static class InsufficientFundsException extends Exception {
        private final String accountId;
        private final BigDecimal requestedAmount;
        private final BigDecimal availableBalance;
        
        public InsufficientFundsException(String accountId, BigDecimal requestedAmount, BigDecimal availableBalance) {
            super(String.format("Insufficient funds in account %s: requested %s, available %s", 
                  accountId, requestedAmount, availableBalance));
            this.accountId = accountId;
            this.requestedAmount = requestedAmount;
            this.availableBalance = availableBalance;
        }
        
        public String getAccountId() { return accountId; }
        public BigDecimal getRequestedAmount() { return requestedAmount; }
        public BigDecimal getAvailableBalance() { return availableBalance; }
    }
    
    public static class AccountNotFoundException extends Exception {
        public AccountNotFoundException(String accountId) {
            super("Account not found: " + accountId);
        }
    }
    
    public static class InvalidTransactionException extends Exception {
        public InvalidTransactionException(String message) {
            super(message);
        }
    }
    
    public static class Account {
        private final String accountId;
        private final String customerId;
        private final AccountType type;
        private BigDecimal balance;
        private BigDecimal creditLimit;
        private final LocalDateTime createdAt;
        private LocalDateTime lastActivity;
        private boolean isActive;
        private final Map<String, Object> metadata;
        private final ReentrantLock lock;
        
        public Account(String accountId, String customerId, AccountType type, BigDecimal initialBalance) {
            this.accountId = accountId;
            this.customerId = customerId;
            this.type = type;
            this.balance = initialBalance != null ? initialBalance : BigDecimal.ZERO;
            this.creditLimit = BigDecimal.ZERO;
            this.createdAt = LocalDateTime.now();
            this.lastActivity = LocalDateTime.now();
            this.isActive = true;
            this.metadata = new HashMap<>();
            this.lock = new ReentrantLock();
        }
        
        // Getters and setters
        public String getAccountId() { return accountId; }
        public String getCustomerId() { return customerId; }
        public AccountType getType() { return type; }
        public BigDecimal getBalance() { return balance; }
        public BigDecimal getCreditLimit() { return creditLimit; }
        public LocalDateTime getCreatedAt() { return createdAt; }
        public LocalDateTime getLastActivity() { return lastActivity; }
        public boolean isActive() { return isActive; }
        public Map<String, Object> getMetadata() { return new HashMap<>(metadata); }
        
        public void setBalance(BigDecimal balance) { this.balance = balance; }
        public void setCreditLimit(BigDecimal creditLimit) { this.creditLimit = creditLimit; }
        public void setActive(boolean active) { this.isActive = active; }
        public void setLastActivity(LocalDateTime lastActivity) { this.lastActivity = lastActivity; }
        
        public BigDecimal getAvailableBalance() {
            return balance.add(creditLimit);
        }
        
        public ReentrantLock getLock() { return lock; }
    }
    
    public static class Transaction {
        private final String transactionId;
        private final String fromAccountId;
        private final String toAccountId;
        private final TransactionType type;
        private final BigDecimal amount;
        private TransactionStatus status;
        private final LocalDateTime timestamp;
        private String description;
        private final Map<String, Object> metadata;
        
        public Transaction(String transactionId, String fromAccountId, String toAccountId, 
                         TransactionType type, BigDecimal amount, String description) {
            this.transactionId = transactionId;
            this.fromAccountId = fromAccountId;
            this.toAccountId = toAccountId;
            this.type = type;
            this.amount = amount;
            this.status = TransactionStatus.PENDING;
            this.timestamp = LocalDateTime.now();
            this.description = description;
            this.metadata = new HashMap<>();
        }
        
        // Getters and setters
        public String getTransactionId() { return transactionId; }
        public String getFromAccountId() { return fromAccountId; }
        public String getToAccountId() { return toAccountId; }
        public TransactionType getType() { return type; }
        public BigDecimal getAmount() { return amount; }
        public TransactionStatus getStatus() { return status; }
        public LocalDateTime getTimestamp() { return timestamp; }
        public String getDescription() { return description; }
        public Map<String, Object> getMetadata() { return new HashMap<>(metadata); }
        
        public void setStatus(TransactionStatus status) { this.status = status; }
        public void setDescription(String description) { this.description = description; }
    }
    
    public static class Customer {
        private final String customerId;
        private String name;
        private String email;
        private String phoneNumber;
        private boolean isActive;
        private final LocalDateTime createdAt;
        private final Set<String> accountIds;
        
        public Customer(String customerId, String name, String email, String phoneNumber) {
            this.customerId = customerId;
            this.name = name;
            this.email = email;
            this.phoneNumber = phoneNumber;
            this.isActive = true;
            this.createdAt = LocalDateTime.now();
            this.accountIds = new HashSet<>();
        }
        
        // Getters and setters
        public String getCustomerId() { return customerId; }
        public String getName() { return name; }
        public String getEmail() { return email; }
        public String getPhoneNumber() { return phoneNumber; }
        public boolean isActive() { return isActive; }
        public LocalDateTime getCreatedAt() { return createdAt; }
        public Set<String> getAccountIds() { return new HashSet<>(accountIds); }
        
        public void setName(String name) { this.name = name; }
        public void setEmail(String email) { this.email = email; }
        public void setPhoneNumber(String phoneNumber) { this.phoneNumber = phoneNumber; }
        public void setActive(boolean active) { this.isActive = active; }
        public void addAccountId(String accountId) { this.accountIds.add(accountId); }
        public void removeAccountId(String accountId) { this.accountIds.remove(accountId); }
    }
    
    // Banking System Implementation
    private final Map<String, Account> accounts;
    private final Map<String, Customer> customers;
    private final Map<String, Transaction> transactions;
    private final Map<AccountType, BigDecimal> interestRates;
    private final Map<AccountType, BigDecimal> maintenanceFees;
    private long transactionCounter;
    
    public BankingSystem() {
        this.accounts = new ConcurrentHashMap<>();
        this.customers = new ConcurrentHashMap<>();
        this.transactions = new ConcurrentHashMap<>();
        this.interestRates = new HashMap<>();
        this.maintenanceFees = new HashMap<>();
        this.transactionCounter = 0;
        
        initializeRatesAndFees();
    }
    
    private void initializeRatesAndFees() {
        // Interest rates (annual)
        interestRates.put(AccountType.CHECKING, new BigDecimal("0.001"));
        interestRates.put(AccountType.SAVINGS, new BigDecimal("0.025"));
        interestRates.put(AccountType.CREDIT, new BigDecimal("0.18"));
        interestRates.put(AccountType.INVESTMENT, new BigDecimal("0.05"));
        interestRates.put(AccountType.BUSINESS, new BigDecimal("0.015"));
        
        // Monthly maintenance fees
        maintenanceFees.put(AccountType.CHECKING, new BigDecimal("10.00"));
        maintenanceFees.put(AccountType.SAVINGS, new BigDecimal("5.00"));
        maintenanceFees.put(AccountType.CREDIT, new BigDecimal("0.00"));
        maintenanceFees.put(AccountType.INVESTMENT, new BigDecimal("25.00"));
        maintenanceFees.put(AccountType.BUSINESS, new BigDecimal("15.00"));
    }
    
    // Customer Management
    public Customer createCustomer(String customerId, String name, String email, String phoneNumber) 
            throws InvalidTransactionException {
        if (customerId == null || customerId.trim().isEmpty()) {
            throw new InvalidTransactionException("Customer ID cannot be null or empty");
        }
        if (name == null || name.trim().isEmpty()) {
            throw new InvalidTransactionException("Customer name cannot be null or empty");
        }
        if (!isValidEmail(email)) {
            throw new InvalidTransactionException("Invalid email format");
        }
        if (customers.containsKey(customerId)) {
            throw new InvalidTransactionException("Customer already exists: " + customerId);
        }
        
        Customer customer = new Customer(customerId, name, email, phoneNumber);
        customers.put(customerId, customer);
        return customer;
    }
    
    public Customer getCustomer(String customerId) throws AccountNotFoundException {
        Customer customer = customers.get(customerId);
        if (customer == null) {
            throw new AccountNotFoundException("Customer not found: " + customerId);
        }
        return customer;
    }
    
    // Account Management
    public Account createAccount(String customerId, AccountType type, BigDecimal initialDeposit) 
            throws InvalidTransactionException, AccountNotFoundException {
        Customer customer = getCustomer(customerId);
        
        if (initialDeposit != null && initialDeposit.compareTo(BigDecimal.ZERO) < 0) {
            throw new InvalidTransactionException("Initial deposit cannot be negative");
        }
        
        String accountId = generateAccountId(type);
        Account account = new Account(accountId, customerId, type, initialDeposit);
        
        // Set default credit limits for credit accounts
        if (type == AccountType.CREDIT) {
            account.setCreditLimit(new BigDecimal("1000.00"));
        }
        
        accounts.put(accountId, account);
        customer.addAccountId(accountId);
        
        return account;
    }
    
    public Account getAccount(String accountId) throws AccountNotFoundException {
        Account account = accounts.get(accountId);
        if (account == null) {
            throw new AccountNotFoundException(accountId);
        }
        return account;
    }
    
    public void closeAccount(String accountId) throws AccountNotFoundException, InvalidTransactionException {
        Account account = getAccount(accountId);
        
        if (account.getBalance().compareTo(BigDecimal.ZERO) != 0) {
            throw new InvalidTransactionException("Cannot close account with non-zero balance");
        }
        
        account.setActive(false);
        Customer customer = customers.get(account.getCustomerId());
        if (customer != null) {
            customer.removeAccountId(accountId);
        }
    }
    
    // Transaction Operations
    public Transaction deposit(String accountId, BigDecimal amount, String description) 
            throws AccountNotFoundException, InvalidTransactionException {
        if (amount == null || amount.compareTo(BigDecimal.ZERO) <= 0) {
            throw new InvalidTransactionException("Deposit amount must be positive");
        }
        
        Account account = getAccount(accountId);
        if (!account.isActive()) {
            throw new InvalidTransactionException("Account is not active");
        }
        
        String transactionId = generateTransactionId();
        Transaction transaction = new Transaction(transactionId, null, accountId, 
                                                TransactionType.DEPOSIT, amount, description);
        
        account.getLock().lock();
        try {
            account.setBalance(account.getBalance().add(amount));
            account.setLastActivity(LocalDateTime.now());
            transaction.setStatus(TransactionStatus.COMPLETED);
        } finally {
            account.getLock().unlock();
        }
        
        transactions.put(transactionId, transaction);
        return transaction;
    }
    
    public Transaction withdraw(String accountId, BigDecimal amount, String description) 
            throws AccountNotFoundException, InvalidTransactionException, InsufficientFundsException {
        if (amount == null || amount.compareTo(BigDecimal.ZERO) <= 0) {
            throw new InvalidTransactionException("Withdrawal amount must be positive");
        }
        
        Account account = getAccount(accountId);
        if (!account.isActive()) {
            throw new InvalidTransactionException("Account is not active");
        }
        
        if (account.getAvailableBalance().compareTo(amount) < 0) {
            throw new InsufficientFundsException(accountId, amount, account.getAvailableBalance());
        }
        
        String transactionId = generateTransactionId();
        Transaction transaction = new Transaction(transactionId, accountId, null, 
                                                TransactionType.WITHDRAWAL, amount, description);
        
        account.getLock().lock();
        try {
            account.setBalance(account.getBalance().subtract(amount));
            account.setLastActivity(LocalDateTime.now());
            transaction.setStatus(TransactionStatus.COMPLETED);
        } finally {
            account.getLock().unlock();
        }
        
        transactions.put(transactionId, transaction);
        return transaction;
    }
    
    public Transaction transfer(String fromAccountId, String toAccountId, BigDecimal amount, String description) 
            throws AccountNotFoundException, InvalidTransactionException, InsufficientFundsException {
        if (fromAccountId.equals(toAccountId)) {
            throw new InvalidTransactionException("Cannot transfer to the same account");
        }
        if (amount == null || amount.compareTo(BigDecimal.ZERO) <= 0) {
            throw new InvalidTransactionException("Transfer amount must be positive");
        }
        
        Account fromAccount = getAccount(fromAccountId);
        Account toAccount = getAccount(toAccountId);
        
        if (!fromAccount.isActive() || !toAccount.isActive()) {
            throw new InvalidTransactionException("Both accounts must be active for transfer");
        }
        
        if (fromAccount.getAvailableBalance().compareTo(amount) < 0) {
            throw new InsufficientFundsException(fromAccountId, amount, fromAccount.getAvailableBalance());
        }
        
        String transactionId = generateTransactionId();
        Transaction transaction = new Transaction(transactionId, fromAccountId, toAccountId, 
                                                TransactionType.TRANSFER, amount, description);
        
        // Lock accounts in consistent order to prevent deadlock
        Account firstLock = fromAccountId.compareTo(toAccountId) < 0 ? fromAccount : toAccount;
        Account secondLock = fromAccountId.compareTo(toAccountId) < 0 ? toAccount : fromAccount;
        
        firstLock.getLock().lock();
        try {
            secondLock.getLock().lock();
            try {
                fromAccount.setBalance(fromAccount.getBalance().subtract(amount));
                toAccount.setBalance(toAccount.getBalance().add(amount));
                fromAccount.setLastActivity(LocalDateTime.now());
                toAccount.setLastActivity(LocalDateTime.now());
                transaction.setStatus(TransactionStatus.COMPLETED);
            } finally {
                secondLock.getLock().unlock();
            }
        } finally {
            firstLock.getLock().unlock();
        }
        
        transactions.put(transactionId, transaction);
        return transaction;
    }
    
    // Interest and Fee Processing
    public void processMonthlyInterest() throws AccountNotFoundException {
        for (Account account : accounts.values()) {
            if (!account.isActive()) continue;
            
            BigDecimal interestRate = interestRates.get(account.getType());
            if (interestRate == null) continue;
            
            BigDecimal monthlyRate = interestRate.divide(new BigDecimal("12"), 6, RoundingMode.HALF_UP);
            BigDecimal interestAmount = account.getBalance().multiply(monthlyRate)
                                              .setScale(2, RoundingMode.HALF_UP);
            
            if (interestAmount.compareTo(BigDecimal.ZERO) > 0) {
                try {
                    deposit(account.getAccountId(), interestAmount, "Monthly interest");
                } catch (InvalidTransactionException e) {
                    // Log error but continue processing other accounts
                }
            }
        }
    }
    
    public void processMonthlyFees() throws AccountNotFoundException {
        for (Account account : accounts.values()) {
            if (!account.isActive()) continue;
            
            BigDecimal fee = maintenanceFees.get(account.getType());
            if (fee == null || fee.compareTo(BigDecimal.ZERO) <= 0) continue;
            
            // Waive fee for accounts with high balance
            if (account.getBalance().compareTo(new BigDecimal("1000")) >= 0) continue;
            
            try {
                withdraw(account.getAccountId(), fee, "Monthly maintenance fee");
            } catch (InvalidTransactionException | InsufficientFundsException e) {
                // Create negative balance if insufficient funds
                account.getLock().lock();
                try {
                    account.setBalance(account.getBalance().subtract(fee));
                } finally {
                    account.getLock().unlock();
                }
            }
        }
    }
    
    // Analytics and Reporting
    public Map<String, Object> generateAccountStatement(String accountId, LocalDateTime startDate, LocalDateTime endDate) 
            throws AccountNotFoundException {
        Account account = getAccount(accountId);
        
        List<Transaction> accountTransactions = new ArrayList<>();
        for (Transaction transaction : transactions.values()) {
            if (transaction.getTimestamp().isAfter(startDate) && transaction.getTimestamp().isBefore(endDate)) {
                if (accountId.equals(transaction.getFromAccountId()) || accountId.equals(transaction.getToAccountId())) {
                    accountTransactions.add(transaction);
                }
            }
        }
        
        accountTransactions.sort(Comparator.comparing(Transaction::getTimestamp));
        
        BigDecimal totalDeposits = accountTransactions.stream()
                .filter(t -> accountId.equals(t.getToAccountId()))
                .map(Transaction::getAmount)
                .reduce(BigDecimal.ZERO, BigDecimal::add);
        
        BigDecimal totalWithdrawals = accountTransactions.stream()
                .filter(t -> accountId.equals(t.getFromAccountId()))
                .map(Transaction::getAmount)
                .reduce(BigDecimal.ZERO, BigDecimal::add);
        
        Map<String, Object> statement = new HashMap<>();
        statement.put("account", account);
        statement.put("startDate", startDate);
        statement.put("endDate", endDate);
        statement.put("transactions", accountTransactions);
        statement.put("totalDeposits", totalDeposits);
        statement.put("totalWithdrawals", totalWithdrawals);
        statement.put("currentBalance", account.getBalance());
        
        return statement;
    }
    
    // Utility Methods
    private String generateAccountId(AccountType type) {
        String prefix = type.name().substring(0, Math.min(3, type.name().length()));
        return prefix + "-" + System.currentTimeMillis() + "-" + (int)(Math.random() * 1000);
    }
    
    private synchronized String generateTransactionId() {
        return "TXN-" + System.currentTimeMillis() + "-" + (++transactionCounter);
    }
    
    private boolean isValidEmail(String email) {
        if (email == null) return false;
        return email.matches("^[A-Za-z0-9+_.-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$");
    }
    
    // Risk Assessment
    public Map<String, Object> assessAccountRisk(String accountId) throws AccountNotFoundException {
        Account account = getAccount(accountId);
        
        // Calculate transaction frequency
        long recentTransactions = transactions.values().stream()
                .filter(t -> t.getTimestamp().isAfter(LocalDateTime.now().minusDays(30)))
                .filter(t -> accountId.equals(t.getFromAccountId()) || accountId.equals(t.getToAccountId()))
                .count();
        
        // Calculate large transaction percentage
        BigDecimal averageBalance = account.getBalance();
        long largeTransactions = transactions.values().stream()
                .filter(t -> accountId.equals(t.getFromAccountId()) || accountId.equals(t.getToAccountId()))
                .filter(t -> t.getAmount().compareTo(averageBalance.multiply(new BigDecimal("0.5"))) > 0)
                .count();
        
        String riskLevel = "LOW";
        if (recentTransactions > 100 || largeTransactions > 10) {
            riskLevel = "HIGH";
        } else if (recentTransactions > 50 || largeTransactions > 5) {
            riskLevel = "MEDIUM";
        }
        
        Map<String, Object> riskAssessment = new HashMap<>();
        riskAssessment.put("accountId", accountId);
        riskAssessment.put("riskLevel", riskLevel);
        riskAssessment.put("recentTransactions", recentTransactions);
        riskAssessment.put("largeTransactions", largeTransactions);
        riskAssessment.put("assessmentDate", LocalDateTime.now());
        
        return riskAssessment;
    }
}