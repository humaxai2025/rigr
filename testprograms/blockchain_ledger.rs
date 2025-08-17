// Complex Blockchain Ledger System - Rust
// This demonstrates advanced Rust patterns for comprehensive test case generation

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::fmt;
use std::error::Error;
use std::thread;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

// Error types
#[derive(Debug, Clone, PartialEq)]
pub enum LedgerError {
    InvalidTransaction(String),
    InsufficientBalance(String),
    InvalidBlock(String),
    ConsensusFailure(String),
    NetworkError(String),
    ValidationError(String),
    DuplicateTransaction(String),
    InvalidSignature(String),
    InvalidAddress(String),
    InvalidAmount(String),
    BlockNotFound(String),
    TransactionNotFound(String),
    MiningError(String),
    SerializationError(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LedgerError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            LedgerError::InsufficientBalance(msg) => write!(f, "Insufficient balance: {}", msg),
            LedgerError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
            LedgerError::ConsensusFailure(msg) => write!(f, "Consensus failure: {}", msg),
            LedgerError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LedgerError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            LedgerError::DuplicateTransaction(msg) => write!(f, "Duplicate transaction: {}", msg),
            LedgerError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            LedgerError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            LedgerError::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            LedgerError::BlockNotFound(msg) => write!(f, "Block not found: {}", msg),
            LedgerError::TransactionNotFound(msg) => write!(f, "Transaction not found: {}", msg),
            LedgerError::MiningError(msg) => write!(f, "Mining error: {}", msg),
            LedgerError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for LedgerError {}

pub type Result<T> = std::result::Result<T, LedgerError>;

// Core data structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    pub fn new(address: String) -> Result<Self> {
        if address.len() < 26 || address.len() > 62 {
            return Err(LedgerError::InvalidAddress(
                "Address length must be between 26 and 62 characters".to_string()
            ));
        }
        
        if !address.chars().all(|c| c.is_alphanumeric()) {
            return Err(LedgerError::InvalidAddress(
                "Address must contain only alphanumeric characters".to_string()
            ));
        }
        
        Ok(Address(address))
    }
    
    pub fn generate() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_part: u64 = rand::random();
        
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}", timestamp, random_part));
        let result = hasher.finalize();
        
        Address(format!("addr_{:x}", u128::from_be_bytes([
            result[0], result[1], result[2], result[3],
            result[4], result[5], result[6], result[7],
            result[8], result[9], result[10], result[11],
            result[12], result[13], result[14], result[15],
        ])))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amount(pub u64);

impl Amount {
    pub const MAX_SUPPLY: u64 = 21_000_000 * 100_000_000; // 21 million coins with 8 decimal places
    
    pub fn new(amount: u64) -> Result<Self> {
        if amount == 0 {
            return Err(LedgerError::InvalidAmount("Amount cannot be zero".to_string()));
        }
        
        if amount > Self::MAX_SUPPLY {
            return Err(LedgerError::InvalidAmount("Amount exceeds maximum supply".to_string()));
        }
        
        Ok(Amount(amount))
    }
    
    pub fn zero() -> Self {
        Amount(0)
    }
    
    pub fn add(&self, other: &Amount) -> Result<Amount> {
        let result = self.0.checked_add(other.0)
            .ok_or_else(|| LedgerError::InvalidAmount("Amount overflow".to_string()))?;
        
        if result > Self::MAX_SUPPLY {
            return Err(LedgerError::InvalidAmount("Amount exceeds maximum supply".to_string()));
        }
        
        Ok(Amount(result))
    }
    
    pub fn subtract(&self, other: &Amount) -> Result<Amount> {
        if self.0 < other.0 {
            return Err(LedgerError::InsufficientBalance(
                format!("Cannot subtract {} from {}", other.0, self.0)
            ));
        }
        
        Ok(Amount(self.0 - other.0))
    }
    
    pub fn multiply(&self, factor: u64) -> Result<Amount> {
        let result = self.0.checked_mul(factor)
            .ok_or_else(|| LedgerError::InvalidAmount("Amount overflow in multiplication".to_string()))?;
        
        if result > Self::MAX_SUPPLY {
            return Err(LedgerError::InvalidAmount("Amount exceeds maximum supply".to_string()));
        }
        
        Ok(Amount(result))
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: Option<Address>,
    pub to: Address,
    pub amount: Amount,
    pub fee: Amount,
    pub timestamp: u64,
    pub nonce: u64,
    pub signature: Option<String>,
    pub data: Option<Vec<u8>>,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl Transaction {
    pub fn new(
        from: Option<Address>,
        to: Address,
        amount: Amount,
        fee: Amount,
        nonce: u64,
        data: Option<Vec<u8>>,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| LedgerError::ValidationError("Invalid system time".to_string()))?
            .as_secs();
        
        let id = Self::generate_id(&from, &to, &amount, timestamp, nonce);
        
        // Validate gas parameters
        if gas_limit == 0 {
            return Err(LedgerError::InvalidTransaction("Gas limit cannot be zero".to_string()));
        }
        
        if gas_price == 0 {
            return Err(LedgerError::InvalidTransaction("Gas price cannot be zero".to_string()));
        }
        
        // Calculate maximum possible gas cost to prevent overflow
        let max_gas_cost = gas_limit.checked_mul(gas_price)
            .ok_or_else(|| LedgerError::InvalidTransaction("Gas cost overflow".to_string()))?;
        
        if max_gas_cost > Amount::MAX_SUPPLY {
            return Err(LedgerError::InvalidTransaction("Gas cost too high".to_string()));
        }
        
        Ok(Transaction {
            id,
            from,
            to,
            amount,
            fee,
            timestamp,
            nonce,
            signature: None,
            data,
            gas_limit,
            gas_price,
        })
    }
    
    pub fn coinbase(to: Address, amount: Amount, block_height: u64) -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| LedgerError::ValidationError("Invalid system time".to_string()))?
            .as_secs();
        
        let id = format!("coinbase_{}_{}", block_height, timestamp);
        
        Ok(Transaction {
            id,
            from: None,
            to,
            amount,
            fee: Amount::zero(),
            timestamp,
            nonce: 0,
            signature: None,
            data: Some(format!("Block reward for height {}", block_height).into_bytes()),
            gas_limit: 0,
            gas_price: 0,
        })
    }
    
    fn generate_id(
        from: &Option<Address>,
        to: &Address,
        amount: &Amount,
        timestamp: u64,
        nonce: u64,
    ) -> String {
        let mut hasher = Sha256::new();
        
        if let Some(from_addr) = from {
            hasher.update(&from_addr.0);
        } else {
            hasher.update("coinbase");
        }
        
        hasher.update(&to.0);
        hasher.update(amount.0.to_be_bytes());
        hasher.update(timestamp.to_be_bytes());
        hasher.update(nonce.to_be_bytes());
        
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    pub fn calculate_total_cost(&self) -> Result<Amount> {
        let gas_cost = Amount::new(
            self.gas_limit.checked_mul(self.gas_price)
                .ok_or_else(|| LedgerError::InvalidTransaction("Gas cost overflow".to_string()))?
        )?;
        
        self.amount.add(&self.fee)?.add(&gas_cost)
    }
    
    pub fn is_coinbase(&self) -> bool {
        self.from.is_none()
    }
    
    pub fn validate_basic(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(LedgerError::InvalidTransaction("Transaction ID cannot be empty".to_string()));
        }
        
        if !self.is_coinbase() && self.amount.0 == 0 {
            return Err(LedgerError::InvalidTransaction("Transaction amount cannot be zero".to_string()));
        }
        
        if self.timestamp == 0 {
            return Err(LedgerError::InvalidTransaction("Invalid timestamp".to_string()));
        }
        
        // Check if transaction is not too far in the future (allow 2 hours)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| LedgerError::ValidationError("Invalid system time".to_string()))?
            .as_secs();
        
        if self.timestamp > current_time + 7200 {
            return Err(LedgerError::InvalidTransaction("Transaction timestamp too far in future".to_string()));
        }
        
        Ok(())
    }
    
    pub fn sign(&mut self, private_key: &str) -> Result<()> {
        // Simplified signature - in real implementation would use proper cryptography
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(private_key);
        let signature = format!("{:x}", hasher.finalize());
        self.signature = Some(signature);
        Ok(())
    }
    
    pub fn verify_signature(&self, public_key: &str) -> Result<bool> {
        match &self.signature {
            Some(sig) => {
                // Simplified verification - in real implementation would use proper cryptography
                let mut hasher = Sha256::new();
                hasher.update(&self.id);
                hasher.update(public_key);
                let expected = format!("{:x}", hasher.finalize());
                Ok(sig == &expected)
            },
            None => Ok(self.is_coinbase()), // Coinbase transactions don't need signatures
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u64,
    pub height: u64,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    pub fn new(
        previous_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
        height: u64,
    ) -> Result<Self> {
        if transactions.is_empty() {
            return Err(LedgerError::InvalidBlock("Block must contain at least one transaction".to_string()));
        }
        
        // Validate all transactions
        for tx in &transactions {
            tx.validate_basic()?;
        }
        
        // Check for duplicate transactions
        let mut tx_ids = HashSet::new();
        for tx in &transactions {
            if !tx_ids.insert(&tx.id) {
                return Err(LedgerError::DuplicateTransaction(tx.id.clone()));
            }
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| LedgerError::ValidationError("Invalid system time".to_string()))?
            .as_secs();
        
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let header = BlockHeader {
            previous_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce: 0,
            height,
            version: 1,
        };
        
        let hash = Self::calculate_hash(&header);
        
        Ok(Block {
            header,
            transactions,
            hash,
        })
    }
    
    pub fn genesis() -> Result<Self> {
        let genesis_address = Address::new("genesis_address_1234567890abcdef".to_string())?;
        let genesis_amount = Amount::new(Amount::MAX_SUPPLY / 2)?; // Half of total supply
        
        let coinbase_tx = Transaction::coinbase(genesis_address, genesis_amount, 0)?;
        
        let mut block = Block::new(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            vec![coinbase_tx],
            1,
            0,
        )?;
        
        // Mine the genesis block
        block.mine()?;
        
        Ok(block)
    }
    
    fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return String::new();
        }
        
        let mut hashes: Vec<String> = transactions.iter()
            .map(|tx| tx.id.clone())
            .collect();
        
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // Duplicate if odd number
                }
                
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            
            hashes = next_level;
        }
        
        hashes.into_iter().next().unwrap_or_default()
    }
    
    fn calculate_hash(header: &BlockHeader) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&header.previous_hash);
        hasher.update(&header.merkle_root);
        hasher.update(header.timestamp.to_be_bytes());
        hasher.update(header.difficulty.to_be_bytes());
        hasher.update(header.nonce.to_be_bytes());
        hasher.update(header.height.to_be_bytes());
        hasher.update(header.version.to_be_bytes());
        
        format!("{:x}", hasher.finalize())
    }
    
    pub fn mine(&mut self) -> Result<()> {
        let target = Self::calculate_target(self.header.difficulty)?;
        let start_time = SystemTime::now();
        let timeout = Duration::from_secs(300); // 5 minute timeout
        
        loop {
            if SystemTime::now().duration_since(start_time).unwrap() > timeout {
                return Err(LedgerError::MiningError("Mining timeout exceeded".to_string()));
            }
            
            self.hash = Self::calculate_hash(&self.header);
            
            if self.hash <= target {
                return Ok(());
            }
            
            self.header.nonce = self.header.nonce.wrapping_add(1);
            
            if self.header.nonce == 0 {
                // Nonce overflow, update timestamp and continue
                self.header.timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| LedgerError::MiningError("Invalid system time".to_string()))?
                    .as_secs();
            }
        }
    }
    
    fn calculate_target(difficulty: u32) -> Result<String> {
        if difficulty == 0 {
            return Err(LedgerError::InvalidBlock("Difficulty cannot be zero".to_string()));
        }
        
        // Simplified target calculation
        let leading_zeros = std::cmp::min(difficulty, 64);
        let mut target = String::new();
        
        for _ in 0..leading_zeros {
            target.push('0');
        }
        
        for _ in leading_zeros..64 {
            target.push('f');
        }
        
        Ok(target)
    }
    
    pub fn validate(&self, previous_block: Option<&Block>) -> Result<()> {
        // Validate block structure
        if self.transactions.is_empty() {
            return Err(LedgerError::InvalidBlock("Block cannot be empty".to_string()));
        }
        
        // Validate header
        if self.header.height == 0 && previous_block.is_some() {
            return Err(LedgerError::InvalidBlock("Genesis block cannot have previous block".to_string()));
        }
        
        if self.header.height > 0 && previous_block.is_none() {
            return Err(LedgerError::InvalidBlock("Non-genesis block must have previous block".to_string()));
        }
        
        // Validate previous hash
        if let Some(prev_block) = previous_block {
            if self.header.previous_hash != prev_block.hash {
                return Err(LedgerError::InvalidBlock("Invalid previous hash".to_string()));
            }
            
            if self.header.height != prev_block.header.height + 1 {
                return Err(LedgerError::InvalidBlock("Invalid block height".to_string()));
            }
            
            if self.header.timestamp <= prev_block.header.timestamp {
                return Err(LedgerError::InvalidBlock("Block timestamp must be greater than previous block".to_string()));
            }
        }
        
        // Validate merkle root
        let calculated_merkle_root = Self::calculate_merkle_root(&self.transactions);
        if self.header.merkle_root != calculated_merkle_root {
            return Err(LedgerError::InvalidBlock("Invalid merkle root".to_string()));
        }
        
        // Validate hash
        let calculated_hash = Self::calculate_hash(&self.header);
        if self.hash != calculated_hash {
            return Err(LedgerError::InvalidBlock("Invalid block hash".to_string()));
        }
        
        // Validate proof of work
        let target = Self::calculate_target(self.header.difficulty)?;
        if self.hash > target {
            return Err(LedgerError::InvalidBlock("Block hash does not meet difficulty target".to_string()));
        }
        
        // Validate transactions
        for tx in &self.transactions {
            tx.validate_basic()?;
        }
        
        // Check for duplicate transactions
        let mut tx_ids = HashSet::new();
        for tx in &self.transactions {
            if !tx_ids.insert(&tx.id) {
                return Err(LedgerError::DuplicateTransaction(tx.id.clone()));
            }
        }
        
        // Validate coinbase transaction (first transaction should be coinbase)
        if !self.transactions[0].is_coinbase() {
            return Err(LedgerError::InvalidBlock("First transaction must be coinbase".to_string()));
        }
        
        // Ensure only one coinbase transaction
        let coinbase_count = self.transactions.iter().filter(|tx| tx.is_coinbase()).count();
        if coinbase_count != 1 {
            return Err(LedgerError::InvalidBlock("Block must have exactly one coinbase transaction".to_string()));
        }
        
        Ok(())
    }
    
    pub fn calculate_total_fees(&self) -> Amount {
        self.transactions.iter()
            .skip(1) // Skip coinbase transaction
            .map(|tx| &tx.fee)
            .fold(Amount::zero(), |acc, fee| acc.add(fee).unwrap_or(acc))
    }
    
    pub fn get_size(&self) -> Result<usize> {
        serde_json::to_string(self)
            .map(|s| s.len())
            .map_err(|_| LedgerError::SerializationError("Failed to serialize block".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub address: Address,
    pub balance: Amount,
    pub nonce: u64,
    pub created_at: u64,
    pub last_activity: u64,
}

impl Account {
    pub fn new(address: Address) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Account {
            address,
            balance: Amount::zero(),
            nonce: 0,
            created_at: timestamp,
            last_activity: timestamp,
        }
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[derive(Debug)]
pub struct LedgerStats {
    pub total_blocks: AtomicU64,
    pub total_transactions: AtomicU64,
    pub total_addresses: AtomicUsize,
    pub total_supply: AtomicU64,
    pub difficulty: AtomicU32,
    pub average_block_time: AtomicU64,
    pub last_block_time: AtomicU64,
}

impl LedgerStats {
    pub fn new() -> Self {
        LedgerStats {
            total_blocks: AtomicU64::new(0),
            total_transactions: AtomicU64::new(0),
            total_addresses: AtomicUsize::new(0),
            total_supply: AtomicU64::new(0),
            difficulty: AtomicU32::new(1),
            average_block_time: AtomicU64::new(0),
            last_block_time: AtomicU64::new(0),
        }
    }
}

#[derive(Debug)]
pub struct BlockchainLedger {
    blocks: RwLock<Vec<Block>>,
    accounts: RwLock<HashMap<Address, Account>>,
    transaction_pool: Mutex<VecDeque<Transaction>>,
    transaction_index: RwLock<HashMap<String, (u64, usize)>>, // tx_id -> (block_height, tx_index)
    block_index: RwLock<HashMap<String, u64>>, // block_hash -> height
    stats: LedgerStats,
    difficulty_adjustment_interval: u64,
    target_block_time: u64,
    max_block_size: usize,
    max_transactions_per_block: usize,
}

impl BlockchainLedger {
    pub fn new() -> Result<Self> {
        let ledger = BlockchainLedger {
            blocks: RwLock::new(Vec::new()),
            accounts: RwLock::new(HashMap::new()),
            transaction_pool: Mutex::new(VecDeque::new()),
            transaction_index: RwLock::new(HashMap::new()),
            block_index: RwLock::new(HashMap::new()),
            stats: LedgerStats::new(),
            difficulty_adjustment_interval: 2016, // Adjust every 2016 blocks
            target_block_time: 600, // 10 minutes in seconds
            max_block_size: 1_000_000, // 1MB
            max_transactions_per_block: 4000,
        };
        
        // Initialize with genesis block
        let genesis_block = Block::genesis()?;
        ledger.add_block(genesis_block)?;
        
        Ok(ledger)
    }
    
    pub fn add_transaction(&self, mut transaction: Transaction) -> Result<()> {
        transaction.validate_basic()?;
        
        // Check if transaction already exists
        let tx_index = self.transaction_index.read().unwrap();
        if tx_index.contains_key(&transaction.id) {
            return Err(LedgerError::DuplicateTransaction(transaction.id));
        }
        
        // Validate transaction against current state
        if !transaction.is_coinbase() {
            self.validate_transaction(&transaction)?;
        }
        
        // Add to transaction pool
        let mut pool = self.transaction_pool.lock().unwrap();
        pool.push_back(transaction);
        
        Ok(())
    }
    
    fn validate_transaction(&self, transaction: &Transaction) -> Result<()> {
        let accounts = self.accounts.read().unwrap();
        
        if let Some(from_addr) = &transaction.from {
            let account = accounts.get(from_addr)
                .ok_or_else(|| LedgerError::InsufficientBalance("Account not found".to_string()))?;
            
            // Check nonce
            if transaction.nonce != account.nonce + 1 {
                return Err(LedgerError::InvalidTransaction(
                    format!("Invalid nonce: expected {}, got {}", account.nonce + 1, transaction.nonce)
                ));
            }
            
            // Check balance
            let total_cost = transaction.calculate_total_cost()?;
            if account.balance.0 < total_cost.0 {
                return Err(LedgerError::InsufficientBalance(
                    format!("Account {} has insufficient balance: {} < {}", 
                           from_addr.0, account.balance.0, total_cost.0)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn create_block(&self, miner_address: Address, max_transactions: Option<usize>) -> Result<Block> {
        let mut pool = self.transaction_pool.lock().unwrap();
        
        if pool.is_empty() {
            return Err(LedgerError::InvalidBlock("No transactions available for block creation".to_string()));
        }
        
        let blocks = self.blocks.read().unwrap();
        let last_block = blocks.last()
            .ok_or_else(|| LedgerError::InvalidBlock("No genesis block found".to_string()))?;
        
        let previous_hash = last_block.hash.clone();
        let height = last_block.header.height + 1;
        let difficulty = self.calculate_next_difficulty()?;
        
        // Create coinbase transaction
        let block_reward = self.calculate_block_reward(height);
        let total_fees = pool.iter().map(|tx| &tx.fee).fold(Amount::zero(), |acc, fee| acc.add(fee).unwrap_or(acc));
        let coinbase_amount = block_reward.add(&total_fees)?;
        let coinbase_tx = Transaction::coinbase(miner_address, coinbase_amount, height)?;
        
        // Select transactions for the block
        let max_tx_count = max_transactions.unwrap_or(self.max_transactions_per_block).min(pool.len());
        let mut selected_transactions = Vec::with_capacity(max_tx_count + 1);
        selected_transactions.push(coinbase_tx);
        
        let mut current_size = 0;
        let mut processed_addresses = HashSet::new();
        
        // Priority queue based on fee per gas
        let mut sorted_transactions: Vec<_> = pool.iter().enumerate().collect();
        sorted_transactions.sort_by(|(_, a), (_, b)| {
            let a_priority = a.fee.0 as f64 / a.gas_limit as f64;
            let b_priority = b.fee.0 as f64 / b.gas_limit as f64;
            b_priority.partial_cmp(&a_priority).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let mut indices_to_remove = Vec::new();
        
        for (original_index, transaction) in sorted_transactions {
            if selected_transactions.len() > max_tx_count {
                break;
            }
            
            // Estimate transaction size
            let tx_size = serde_json::to_string(transaction)
                .map_err(|_| LedgerError::SerializationError("Failed to serialize transaction".to_string()))?
                .len();
            
            if current_size + tx_size > self.max_block_size {
                break;
            }
            
            // Validate transaction
            if let Err(_) = self.validate_transaction(transaction) {
                indices_to_remove.push(original_index);
                continue;
            }
            
            // Check for conflicting transactions (same sender)
            if let Some(from_addr) = &transaction.from {
                if processed_addresses.contains(from_addr) {
                    continue; // Skip to avoid nonce conflicts
                }
                processed_addresses.insert(from_addr.clone());
            }
            
            selected_transactions.push(transaction.clone());
            indices_to_remove.push(original_index);
            current_size += tx_size;
        }
        
        // Remove selected transactions from pool (in reverse order to maintain indices)
        indices_to_remove.sort_by(|a, b| b.cmp(a));
        for index in indices_to_remove {
            pool.remove(index);
        }
        
        drop(pool);
        drop(blocks);
        
        if selected_transactions.len() < 2 {
            return Err(LedgerError::InvalidBlock("Not enough valid transactions for block".to_string()));
        }
        
        Block::new(previous_hash, selected_transactions, difficulty, height)
    }
    
    pub fn add_block(&self, mut block: Block) -> Result<()> {
        // Validate block
        let blocks = self.blocks.read().unwrap();
        let previous_block = if block.header.height == 0 {
            None
        } else {
            blocks.last()
        };
        
        block.validate(previous_block)?;
        drop(blocks);
        
        // Apply transactions to state
        self.apply_block_transactions(&block)?;
        
        // Add block to chain
        let mut blocks = self.blocks.write().unwrap();
        let mut block_index = self.block_index.write().unwrap();
        let mut tx_index = self.transaction_index.write().unwrap();
        
        // Index transactions
        for (tx_idx, transaction) in block.transactions.iter().enumerate() {
            tx_index.insert(transaction.id.clone(), (block.header.height, tx_idx));
        }
        
        // Index block
        block_index.insert(block.hash.clone(), block.header.height);
        
        blocks.push(block.clone());
        
        // Update stats
        self.stats.total_blocks.store(blocks.len() as u64, Ordering::Relaxed);
        self.stats.total_transactions.fetch_add(block.transactions.len() as u64, Ordering::Relaxed);
        self.stats.last_block_time.store(block.header.timestamp, Ordering::Relaxed);
        
        drop(blocks);
        drop(block_index);
        drop(tx_index);
        
        Ok(())
    }
    
    fn apply_block_transactions(&self, block: &Block) -> Result<()> {
        let mut accounts = self.accounts.write().unwrap();
        
        for transaction in &block.transactions {
            if let Some(from_addr) = &transaction.from {
                // Debit sender
                let sender_account = accounts.get_mut(from_addr)
                    .ok_or_else(|| LedgerError::InsufficientBalance("Sender account not found".to_string()))?;
                
                let total_cost = transaction.calculate_total_cost()?;
                sender_account.balance = sender_account.balance.subtract(&total_cost)?;
                sender_account.nonce += 1;
                sender_account.update_activity();
            }
            
            // Credit recipient
            let recipient_account = accounts.entry(transaction.to.clone())
                .or_insert_with(|| Account::new(transaction.to.clone()));
            
            recipient_account.balance = recipient_account.balance.add(&transaction.amount)?;
            recipient_account.update_activity();
        }
        
        self.stats.total_addresses.store(accounts.len(), Ordering::Relaxed);
        
        // Calculate total supply
        let total_supply = accounts.values()
            .map(|account| account.balance.0)
            .sum::<u64>();
        self.stats.total_supply.store(total_supply, Ordering::Relaxed);
        
        Ok(())
    }
    
    fn calculate_next_difficulty(&self) -> Result<u32> {
        let blocks = self.blocks.read().unwrap();
        let current_height = blocks.len() as u64;
        
        if current_height < self.difficulty_adjustment_interval {
            return Ok(1); // Initial difficulty
        }
        
        if current_height % self.difficulty_adjustment_interval != 0 {
            return Ok(blocks.last().unwrap().header.difficulty); // No adjustment needed
        }
        
        // Calculate time taken for last interval
        let last_block = blocks.last().unwrap();
        let interval_start_block = &blocks[(current_height - self.difficulty_adjustment_interval) as usize];
        
        let actual_time = last_block.header.timestamp - interval_start_block.header.timestamp;
        let expected_time = self.difficulty_adjustment_interval * self.target_block_time;
        
        let current_difficulty = last_block.header.difficulty;
        
        // Adjust difficulty (limit changes to 4x or 1/4)
        let new_difficulty = if actual_time < expected_time / 4 {
            current_difficulty * 4
        } else if actual_time > expected_time * 4 {
            std::cmp::max(1, current_difficulty / 4)
        } else {
            let ratio = expected_time as f64 / actual_time as f64;
            std::cmp::max(1, (current_difficulty as f64 * ratio) as u32)
        };
        
        self.stats.difficulty.store(new_difficulty, Ordering::Relaxed);
        
        Ok(new_difficulty)
    }
    
    fn calculate_block_reward(&self, height: u64) -> Amount {
        // Halving every 210,000 blocks
        let halvings = height / 210_000;
        let base_reward = 50_00000000u64; // 50 coins with 8 decimal places
        
        if halvings >= 64 {
            Amount::zero()
        } else {
            Amount::new(base_reward >> halvings).unwrap_or(Amount::zero())
        }
    }
    
    pub fn get_balance(&self, address: &Address) -> Amount {
        let accounts = self.accounts.read().unwrap();
        accounts.get(address)
            .map(|account| account.balance.clone())
            .unwrap_or(Amount::zero())
    }
    
    pub fn get_account(&self, address: &Address) -> Option<Account> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(address).cloned()
    }
    
    pub fn get_block(&self, height: u64) -> Option<Block> {
        let blocks = self.blocks.read().unwrap();
        blocks.get(height as usize).cloned()
    }
    
    pub fn get_block_by_hash(&self, hash: &str) -> Option<Block> {
        let block_index = self.block_index.read().unwrap();
        if let Some(&height) = block_index.get(hash) {
            self.get_block(height)
        } else {
            None
        }
    }
    
    pub fn get_transaction(&self, tx_id: &str) -> Option<(Block, Transaction)> {
        let tx_index = self.transaction_index.read().unwrap();
        if let Some(&(block_height, tx_index)) = tx_index.get(tx_id) {
            if let Some(block) = self.get_block(block_height) {
                if let Some(transaction) = block.transactions.get(tx_index) {
                    return Some((block, transaction.clone()));
                }
            }
        }
        None
    }
    
    pub fn get_chain_height(&self) -> u64 {
        let blocks = self.blocks.read().unwrap();
        blocks.len() as u64
    }
    
    pub fn get_pending_transactions(&self) -> Vec<Transaction> {
        let pool = self.transaction_pool.lock().unwrap();
        pool.iter().cloned().collect()
    }
    
    pub fn validate_chain(&self) -> Result<()> {
        let blocks = self.blocks.read().unwrap();
        
        if blocks.is_empty() {
            return Err(LedgerError::InvalidBlock("Chain is empty".to_string()));
        }
        
        // Validate genesis block
        if blocks[0].header.height != 0 {
            return Err(LedgerError::InvalidBlock("Invalid genesis block height".to_string()));
        }
        
        // Validate chain continuity
        for i in 1..blocks.len() {
            blocks[i].validate(Some(&blocks[i - 1]))?;
        }
        
        Ok(())
    }
    
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_blocks".to_string(), self.stats.total_blocks.load(Ordering::Relaxed));
        stats.insert("total_transactions".to_string(), self.stats.total_transactions.load(Ordering::Relaxed));
        stats.insert("total_addresses".to_string(), self.stats.total_addresses.load(Ordering::Relaxed) as u64);
        stats.insert("total_supply".to_string(), self.stats.total_supply.load(Ordering::Relaxed));
        stats.insert("difficulty".to_string(), self.stats.difficulty.load(Ordering::Relaxed) as u64);
        stats.insert("last_block_time".to_string(), self.stats.last_block_time.load(Ordering::Relaxed));
        stats
    }
    
    pub fn mine_block(&self, miner_address: Address) -> Result<Block> {
        let mut block = self.create_block(miner_address, None)?;
        block.mine()?;
        self.add_block(block.clone())?;
        Ok(block)
    }
    
    pub fn transfer(&self, from: Address, to: Address, amount: Amount, fee: Amount, private_key: &str) -> Result<String> {
        let account = self.get_account(&from)
            .ok_or_else(|| LedgerError::InsufficientBalance("Account not found".to_string()))?;
        
        let mut transaction = Transaction::new(
            Some(from),
            to,
            amount,
            fee,
            account.nonce + 1,
            None,
            21000, // Standard gas limit
            1000,  // Standard gas price
        )?;
        
        transaction.sign(private_key)?;
        
        let tx_id = transaction.id.clone();
        self.add_transaction(transaction)?;
        
        Ok(tx_id)
    }
}

impl Default for BlockchainLedger {
    fn default() -> Self {
        Self::new().expect("Failed to create default blockchain ledger")
    }
}

// Additional helper functions and tests would go here in a real implementation
pub fn format_amount(amount: &Amount) -> String {
    let coins = amount.0 / 100_000_000;
    let satoshis = amount.0 % 100_000_000;
    format!("{}.{:08}", coins, satoshis)
}

pub fn parse_amount(amount_str: &str) -> Result<Amount> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    
    if parts.len() > 2 {
        return Err(LedgerError::InvalidAmount("Invalid amount format".to_string()));
    }
    
    let coins: u64 = parts[0].parse()
        .map_err(|_| LedgerError::InvalidAmount("Invalid coin amount".to_string()))?;
    
    let satoshis = if parts.len() == 2 {
        let decimal_part = format!("{:0<8}", parts[1]);
        if decimal_part.len() > 8 {
            return Err(LedgerError::InvalidAmount("Too many decimal places".to_string()));
        }
        decimal_part.parse::<u64>()
            .map_err(|_| LedgerError::InvalidAmount("Invalid decimal amount".to_string()))?
    } else {
        0
    };
    
    let total = coins.checked_mul(100_000_000)
        .and_then(|c| c.checked_add(satoshis))
        .ok_or_else(|| LedgerError::InvalidAmount("Amount overflow".to_string()))?;
    
    Amount::new(total)
}