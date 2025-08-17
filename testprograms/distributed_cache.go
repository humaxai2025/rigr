// Complex Distributed Cache System - Go
// This demonstrates advanced Go patterns for comprehensive test case generation

package cache

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"hash/fnv"
	"log"
	"sync"
	"sync/atomic"
	"time"
)

// Error definitions
var (
	ErrKeyNotFound     = errors.New("key not found")
	ErrKeyExpired      = errors.New("key expired")
	ErrInvalidKey      = errors.New("invalid key")
	ErrInvalidValue    = errors.New("invalid value")
	ErrCacheFull       = errors.New("cache is full")
	ErrNodeUnavailable = errors.New("node unavailable")
	ErrReplicationFailed = errors.New("replication failed")
)

// CacheItem represents an item stored in the cache
type CacheItem struct {
	Key        string                 `json:"key"`
	Value      interface{}            `json:"value"`
	ExpiresAt  *time.Time            `json:"expires_at,omitempty"`
	CreatedAt  time.Time             `json:"created_at"`
	UpdatedAt  time.Time             `json:"updated_at"`
	AccessCount int64                `json:"access_count"`
	LastAccess time.Time             `json:"last_access"`
	Size       int64                 `json:"size"`
	Metadata   map[string]interface{} `json:"metadata,omitempty"`
	Tags       []string              `json:"tags,omitempty"`
}

// IsExpired checks if the cache item has expired
func (item *CacheItem) IsExpired() bool {
	if item.ExpiresAt == nil {
		return false
	}
	return time.Now().After(*item.ExpiresAt)
}

// Touch updates the last access time and increments access count
func (item *CacheItem) Touch() {
	atomic.AddInt64(&item.AccessCount, 1)
	item.LastAccess = time.Now()
}

// NodeInfo represents information about a cache node
type NodeInfo struct {
	ID           string            `json:"id"`
	Address      string            `json:"address"`
	Port         int               `json:"port"`
	IsHealthy    bool              `json:"is_healthy"`
	LastHeartbeat time.Time        `json:"last_heartbeat"`
	LoadFactor   float64           `json:"load_factor"`
	Version      string            `json:"version"`
	Metadata     map[string]string `json:"metadata,omitempty"`
}

// CacheStats holds statistics about cache operations
type CacheStats struct {
	Hits              int64     `json:"hits"`
	Misses            int64     `json:"misses"`
	Sets              int64     `json:"sets"`
	Deletes           int64     `json:"deletes"`
	Evictions         int64     `json:"evictions"`
	TotalItems        int64     `json:"total_items"`
	TotalSize         int64     `json:"total_size"`
	MemoryUsage       int64     `json:"memory_usage"`
	LastResetTime     time.Time `json:"last_reset_time"`
	AverageAccessTime float64   `json:"average_access_time"`
}

// HitRatio calculates the cache hit ratio
func (stats *CacheStats) HitRatio() float64 {
	total := stats.Hits + stats.Misses
	if total == 0 {
		return 0.0
	}
	return float64(stats.Hits) / float64(total)
}

// EvictionPolicy defines how items should be evicted when cache is full
type EvictionPolicy int

const (
	LRU EvictionPolicy = iota // Least Recently Used
	LFU                       // Least Frequently Used
	FIFO                      // First In, First Out
	TTL                       // Time To Live based
	Random                    // Random eviction
)

// ConsistencyLevel defines the level of consistency required for operations
type ConsistencyLevel int

const (
	Eventual ConsistencyLevel = iota // Eventual consistency
	Strong                           // Strong consistency
	Quorum                           // Quorum-based consistency
)

// CacheConfig holds configuration for the distributed cache
type CacheConfig struct {
	MaxSize           int64             `json:"max_size"`
	DefaultTTL        time.Duration     `json:"default_ttl"`
	EvictionPolicy    EvictionPolicy    `json:"eviction_policy"`
	ReplicationFactor int               `json:"replication_factor"`
	ConsistencyLevel  ConsistencyLevel  `json:"consistency_level"`
	CompressionEnabled bool             `json:"compression_enabled"`
	EncryptionEnabled bool              `json:"encryption_enabled"`
	MonitoringEnabled bool              `json:"monitoring_enabled"`
	PersistenceEnabled bool             `json:"persistence_enabled"`
	BackupInterval    time.Duration     `json:"backup_interval"`
	MaxNodes          int               `json:"max_nodes"`
	HeartbeatInterval time.Duration     `json:"heartbeat_interval"`
	RequestTimeout    time.Duration     `json:"request_timeout"`
}

// DefaultConfig returns a default configuration
func DefaultConfig() *CacheConfig {
	return &CacheConfig{
		MaxSize:           1024 * 1024 * 100, // 100MB
		DefaultTTL:        time.Hour,
		EvictionPolicy:    LRU,
		ReplicationFactor: 2,
		ConsistencyLevel:  Eventual,
		CompressionEnabled: false,
		EncryptionEnabled: false,
		MonitoringEnabled: true,
		PersistenceEnabled: false,
		BackupInterval:    time.Hour * 6,
		MaxNodes:          10,
		HeartbeatInterval: time.Second * 30,
		RequestTimeout:    time.Second * 5,
	}
}

// DistributedCache represents the main distributed cache system
type DistributedCache struct {
	config        *CacheConfig
	nodes         map[string]*NodeInfo
	localCache    map[string]*CacheItem
	stats         *CacheStats
	mutex         sync.RWMutex
	nodesMutex    sync.RWMutex
	statsMutex    sync.RWMutex
	nodeID        string
	isRunning     bool
	shutdownChan  chan struct{}
	eventLog      []CacheEvent
	eventLogMutex sync.RWMutex
	hashRing      *ConsistentHashRing
}

// CacheEvent represents an event in the cache system
type CacheEvent struct {
	Type      string                 `json:"type"`
	Key       string                 `json:"key,omitempty"`
	NodeID    string                 `json:"node_id"`
	Timestamp time.Time              `json:"timestamp"`
	Metadata  map[string]interface{} `json:"metadata,omitempty"`
}

// ConsistentHashRing implements consistent hashing for node selection
type ConsistentHashRing struct {
	nodes    map[uint32]string
	keys     []uint32
	mutex    sync.RWMutex
	replicas int
}

// NewConsistentHashRing creates a new consistent hash ring
func NewConsistentHashRing(replicas int) *ConsistentHashRing {
	return &ConsistentHashRing{
		nodes:    make(map[uint32]string),
		replicas: replicas,
	}
}

// AddNode adds a node to the hash ring
func (ring *ConsistentHashRing) AddNode(nodeID string) {
	ring.mutex.Lock()
	defer ring.mutex.Unlock()

	for i := 0; i < ring.replicas; i++ {
		hash := ring.hashKey(fmt.Sprintf("%s:%d", nodeID, i))
		ring.nodes[hash] = nodeID
		ring.keys = append(ring.keys, hash)
	}
	ring.sortKeys()
}

// RemoveNode removes a node from the hash ring
func (ring *ConsistentHashRing) RemoveNode(nodeID string) {
	ring.mutex.Lock()
	defer ring.mutex.Unlock()

	for i := 0; i < ring.replicas; i++ {
		hash := ring.hashKey(fmt.Sprintf("%s:%d", nodeID, i))
		delete(ring.nodes, hash)
		
		// Remove from keys slice
		for j, key := range ring.keys {
			if key == hash {
				ring.keys = append(ring.keys[:j], ring.keys[j+1:]...)
				break
			}
		}
	}
}

// GetNode returns the node responsible for a given key
func (ring *ConsistentHashRing) GetNode(key string) (string, error) {
	ring.mutex.RLock()
	defer ring.mutex.RUnlock()

	if len(ring.keys) == 0 {
		return "", ErrNodeUnavailable
	}

	hash := ring.hashKey(key)
	
	// Find the first node with hash >= key hash
	idx := ring.binarySearch(hash)
	return ring.nodes[ring.keys[idx]], nil
}

// GetNodes returns multiple nodes for replication
func (ring *ConsistentHashRing) GetNodes(key string, count int) ([]string, error) {
	ring.mutex.RLock()
	defer ring.mutex.RUnlock()

	if len(ring.keys) == 0 {
		return nil, ErrNodeUnavailable
	}

	uniqueNodes := make(map[string]bool)
	nodes := make([]string, 0, count)
	hash := ring.hashKey(key)
	
	idx := ring.binarySearch(hash)
	
	for len(nodes) < count && len(uniqueNodes) < len(ring.nodes) {
		nodeID := ring.nodes[ring.keys[idx]]
		if !uniqueNodes[nodeID] {
			uniqueNodes[nodeID] = true
			nodes = append(nodes, nodeID)
		}
		idx = (idx + 1) % len(ring.keys)
	}
	
	return nodes, nil
}

func (ring *ConsistentHashRing) hashKey(key string) uint32 {
	h := fnv.New32a()
	h.Write([]byte(key))
	return h.Sum32()
}

func (ring *ConsistentHashRing) sortKeys() {
	// Simple bubble sort for small arrays
	for i := 0; i < len(ring.keys)-1; i++ {
		for j := 0; j < len(ring.keys)-i-1; j++ {
			if ring.keys[j] > ring.keys[j+1] {
				ring.keys[j], ring.keys[j+1] = ring.keys[j+1], ring.keys[j]
			}
		}
	}
}

func (ring *ConsistentHashRing) binarySearch(hash uint32) int {
	left, right := 0, len(ring.keys)-1
	
	for left <= right {
		mid := (left + right) / 2
		if ring.keys[mid] == hash {
			return mid
		} else if ring.keys[mid] < hash {
			left = mid + 1
		} else {
			right = mid - 1
		}
	}
	
	// If exact match not found, return the next higher value
	if left < len(ring.keys) {
		return left
	}
	return 0 // Wrap around to first node
}

// NewDistributedCache creates a new distributed cache instance
func NewDistributedCache(nodeID string, config *CacheConfig) (*DistributedCache, error) {
	if nodeID == "" {
		return nil, errors.New("node ID cannot be empty")
	}
	
	if config == nil {
		config = DefaultConfig()
	}

	if err := validateConfig(config); err != nil {
		return nil, fmt.Errorf("invalid config: %w", err)
	}

	cache := &DistributedCache{
		config:       config,
		nodes:        make(map[string]*NodeInfo),
		localCache:   make(map[string]*CacheItem),
		stats:        &CacheStats{LastResetTime: time.Now()},
		nodeID:       nodeID,
		shutdownChan: make(chan struct{}),
		eventLog:     make([]CacheEvent, 0),
		hashRing:     NewConsistentHashRing(config.ReplicationFactor * 10),
	}

	// Add self to nodes
	cache.nodes[nodeID] = &NodeInfo{
		ID:           nodeID,
		IsHealthy:    true,
		LastHeartbeat: time.Now(),
		LoadFactor:   0.0,
		Version:      "1.0.0",
	}
	
	cache.hashRing.AddNode(nodeID)
	return cache, nil
}

// Start starts the distributed cache
func (dc *DistributedCache) Start(ctx context.Context) error {
	if dc.isRunning {
		return errors.New("cache is already running")
	}

	dc.isRunning = true
	dc.logEvent("CACHE_STARTED", "", map[string]interface{}{
		"node_id": dc.nodeID,
		"config":  dc.config,
	})

	// Start background goroutines
	go dc.heartbeatLoop(ctx)
	go dc.evictionLoop(ctx)
	go dc.monitoringLoop(ctx)

	if dc.config.PersistenceEnabled {
		go dc.persistenceLoop(ctx)
	}

	return nil
}

// Stop stops the distributed cache
func (dc *DistributedCache) Stop() error {
	if !dc.isRunning {
		return errors.New("cache is not running")
	}

	dc.isRunning = false
	close(dc.shutdownChan)

	dc.logEvent("CACHE_STOPPED", "", map[string]interface{}{
		"node_id": dc.nodeID,
	})

	return nil
}

// Set stores a key-value pair in the cache
func (dc *DistributedCache) Set(ctx context.Context, key string, value interface{}, ttl time.Duration) error {
	if err := dc.validateKey(key); err != nil {
		return err
	}

	if value == nil {
		return ErrInvalidValue
	}

	// Calculate item size
	size, err := dc.calculateSize(value)
	if err != nil {
		return fmt.Errorf("failed to calculate size: %w", err)
	}

	// Check if cache has space
	if dc.getCurrentSize()+size > dc.config.MaxSize {
		if err := dc.evictItems(size); err != nil {
			return err
		}
	}

	// Create cache item
	item := &CacheItem{
		Key:       key,
		Value:     value,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		LastAccess: time.Now(),
		Size:      size,
		Metadata:  make(map[string]interface{}),
	}

	if ttl > 0 {
		expiresAt := time.Now().Add(ttl)
		item.ExpiresAt = &expiresAt
	} else if dc.config.DefaultTTL > 0 {
		expiresAt := time.Now().Add(dc.config.DefaultTTL)
		item.ExpiresAt = &expiresAt
	}

	// Store locally
	dc.mutex.Lock()
	dc.localCache[key] = item
	dc.mutex.Unlock()

	// Update stats
	dc.updateStats(func(stats *CacheStats) {
		atomic.AddInt64(&stats.Sets, 1)
		atomic.AddInt64(&stats.TotalItems, 1)
		atomic.AddInt64(&stats.TotalSize, size)
	})

	// Replicate to other nodes if needed
	if dc.config.ReplicationFactor > 1 {
		go dc.replicateSet(ctx, key, value, ttl)
	}

	dc.logEvent("SET", key, map[string]interface{}{
		"size": size,
		"ttl":  ttl,
	})

	return nil
}

// Get retrieves a value from the cache
func (dc *DistributedCache) Get(ctx context.Context, key string) (interface{}, error) {
	if err := dc.validateKey(key); err != nil {
		return nil, err
	}

	startTime := time.Now()
	defer func() {
		duration := time.Since(startTime)
		dc.updateAverageAccessTime(duration)
	}()

	dc.mutex.RLock()
	item, exists := dc.localCache[key]
	dc.mutex.RUnlock()

	if !exists {
		// Try to get from other nodes
		if value, err := dc.getFromOtherNodes(ctx, key); err == nil {
			dc.updateStats(func(stats *CacheStats) {
				atomic.AddInt64(&stats.Hits, 1)
			})
			return value, nil
		}

		dc.updateStats(func(stats *CacheStats) {
			atomic.AddInt64(&stats.Misses, 1)
		})
		return nil, ErrKeyNotFound
	}

	if item.IsExpired() {
		dc.Delete(ctx, key)
		dc.updateStats(func(stats *CacheStats) {
			atomic.AddInt64(&stats.Misses, 1)
		})
		return nil, ErrKeyExpired
	}

	// Update access information
	item.Touch()

	dc.updateStats(func(stats *CacheStats) {
		atomic.AddInt64(&stats.Hits, 1)
	})

	dc.logEvent("GET", key, map[string]interface{}{
		"hit": true,
	})

	return item.Value, nil
}

// Delete removes a key from the cache
func (dc *DistributedCache) Delete(ctx context.Context, key string) error {
	if err := dc.validateKey(key); err != nil {
		return err
	}

	dc.mutex.Lock()
	item, exists := dc.localCache[key]
	if exists {
		delete(dc.localCache, key)
	}
	dc.mutex.Unlock()

	if exists {
		dc.updateStats(func(stats *CacheStats) {
			atomic.AddInt64(&stats.Deletes, 1)
			atomic.AddInt64(&stats.TotalItems, -1)
			atomic.AddInt64(&stats.TotalSize, -item.Size)
		})

		dc.logEvent("DELETE", key, nil)

		// Replicate deletion to other nodes
		if dc.config.ReplicationFactor > 1 {
			go dc.replicateDelete(ctx, key)
		}
	}

	return nil
}

// Exists checks if a key exists in the cache
func (dc *DistributedCache) Exists(ctx context.Context, key string) (bool, error) {
	if err := dc.validateKey(key); err != nil {
		return false, err
	}

	dc.mutex.RLock()
	item, exists := dc.localCache[key]
	dc.mutex.RUnlock()

	if !exists {
		return false, nil
	}

	if item.IsExpired() {
		dc.Delete(ctx, key)
		return false, nil
	}

	return true, nil
}

// Keys returns all keys matching a pattern
func (dc *DistributedCache) Keys(pattern string) ([]string, error) {
	dc.mutex.RLock()
	defer dc.mutex.RUnlock()

	keys := make([]string, 0)
	for key, item := range dc.localCache {
		if item.IsExpired() {
			continue
		}

		// Simple pattern matching (supports * wildcard)
		if pattern == "*" || dc.matchPattern(key, pattern) {
			keys = append(keys, key)
		}
	}

	return keys, nil
}

// GetStats returns current cache statistics
func (dc *DistributedCache) GetStats() *CacheStats {
	dc.statsMutex.RLock()
	defer dc.statsMutex.RUnlock()

	// Create a copy to avoid race conditions
	statsCopy := *dc.stats
	return &statsCopy
}

// ResetStats resets all statistics
func (dc *DistributedCache) ResetStats() {
	dc.updateStats(func(stats *CacheStats) {
		*stats = CacheStats{LastResetTime: time.Now()}
	})
}

// GetNodeInfo returns information about cache nodes
func (dc *DistributedCache) GetNodeInfo() map[string]*NodeInfo {
	dc.nodesMutex.RLock()
	defer dc.nodesMutex.RUnlock()

	nodes := make(map[string]*NodeInfo)
	for id, info := range dc.nodes {
		nodeInfo := *info // Copy
		nodes[id] = &nodeInfo
	}

	return nodes
}

// FlushAll removes all items from the cache
func (dc *DistributedCache) FlushAll(ctx context.Context) error {
	dc.mutex.Lock()
	dc.localCache = make(map[string]*CacheItem)
	dc.mutex.Unlock()

	dc.updateStats(func(stats *CacheStats) {
		stats.TotalItems = 0
		stats.TotalSize = 0
	})

	dc.logEvent("FLUSH_ALL", "", nil)

	// Replicate flush to other nodes
	if dc.config.ReplicationFactor > 1 {
		go dc.replicateFlush(ctx)
	}

	return nil
}

// Private helper methods

func (dc *DistributedCache) validateKey(key string) error {
	if key == "" {
		return ErrInvalidKey
	}
	if len(key) > 255 {
		return ErrInvalidKey
	}
	return nil
}

func (dc *DistributedCache) calculateSize(value interface{}) (int64, error) {
	data, err := json.Marshal(value)
	if err != nil {
		return 0, err
	}
	return int64(len(data)), nil
}

func (dc *DistributedCache) getCurrentSize() int64 {
	dc.statsMutex.RLock()
	defer dc.statsMutex.RUnlock()
	return dc.stats.TotalSize
}

func (dc *DistributedCache) evictItems(neededSpace int64) error {
	switch dc.config.EvictionPolicy {
	case LRU:
		return dc.evictLRU(neededSpace)
	case LFU:
		return dc.evictLFU(neededSpace)
	case FIFO:
		return dc.evictFIFO(neededSpace)
	case TTL:
		return dc.evictTTL(neededSpace)
	case Random:
		return dc.evictRandom(neededSpace)
	default:
		return dc.evictLRU(neededSpace)
	}
}

func (dc *DistributedCache) evictLRU(neededSpace int64) error {
	dc.mutex.Lock()
	defer dc.mutex.Unlock()

	// Create a slice of items sorted by last access time
	type itemWithKey struct {
		key  string
		item *CacheItem
	}

	items := make([]itemWithKey, 0, len(dc.localCache))
	for key, item := range dc.localCache {
		items = append(items, itemWithKey{key: key, item: item})
	}

	// Sort by last access time (oldest first)
	for i := 0; i < len(items)-1; i++ {
		for j := i + 1; j < len(items); j++ {
			if items[i].item.LastAccess.After(items[j].item.LastAccess) {
				items[i], items[j] = items[j], items[i]
			}
		}
	}

	freedSpace := int64(0)
	evicted := 0

	for _, itemWithKey := range items {
		if freedSpace >= neededSpace {
			break
		}

		delete(dc.localCache, itemWithKey.key)
		freedSpace += itemWithKey.item.Size
		evicted++

		dc.logEvent("EVICTED", itemWithKey.key, map[string]interface{}{
			"policy": "LRU",
		})
	}

	dc.updateStats(func(stats *CacheStats) {
		atomic.AddInt64(&stats.Evictions, int64(evicted))
		atomic.AddInt64(&stats.TotalItems, -int64(evicted))
		atomic.AddInt64(&stats.TotalSize, -freedSpace)
	})

	if freedSpace < neededSpace {
		return ErrCacheFull
	}

	return nil
}

func (dc *DistributedCache) evictLFU(neededSpace int64) error {
	// Implementation similar to LRU but sorts by access count
	dc.mutex.Lock()
	defer dc.mutex.Unlock()

	type itemWithKey struct {
		key  string
		item *CacheItem
	}

	items := make([]itemWithKey, 0, len(dc.localCache))
	for key, item := range dc.localCache {
		items = append(items, itemWithKey{key: key, item: item})
	}

	// Sort by access count (least frequent first)
	for i := 0; i < len(items)-1; i++ {
		for j := i + 1; j < len(items); j++ {
			if items[i].item.AccessCount > items[j].item.AccessCount {
				items[i], items[j] = items[j], items[i]
			}
		}
	}

	freedSpace := int64(0)
	evicted := 0

	for _, itemWithKey := range items {
		if freedSpace >= neededSpace {
			break
		}

		delete(dc.localCache, itemWithKey.key)
		freedSpace += itemWithKey.item.Size
		evicted++

		dc.logEvent("EVICTED", itemWithKey.key, map[string]interface{}{
			"policy": "LFU",
		})
	}

	dc.updateStats(func(stats *CacheStats) {
		atomic.AddInt64(&stats.Evictions, int64(evicted))
		atomic.AddInt64(&stats.TotalItems, -int64(evicted))
		atomic.AddInt64(&stats.TotalSize, -freedSpace)
	})

	if freedSpace < neededSpace {
		return ErrCacheFull
	}

	return nil
}

func (dc *DistributedCache) evictFIFO(neededSpace int64) error {
	// Implementation similar to LRU but sorts by creation time
	dc.mutex.Lock()
	defer dc.mutex.Unlock()

	type itemWithKey struct {
		key  string
		item *CacheItem
	}

	items := make([]itemWithKey, 0, len(dc.localCache))
	for key, item := range dc.localCache {
		items = append(items, itemWithKey{key: key, item: item})
	}

	// Sort by creation time (oldest first)
	for i := 0; i < len(items)-1; i++ {
		for j := i + 1; j < len(items); j++ {
			if items[i].item.CreatedAt.After(items[j].item.CreatedAt) {
				items[i], items[j] = items[j], items[i]
			}
		}
	}

	freedSpace := int64(0)
	evicted := 0

	for _, itemWithKey := range items {
		if freedSpace >= neededSpace {
			break
		}

		delete(dc.localCache, itemWithKey.key)
		freedSpace += itemWithKey.item.Size
		evicted++

		dc.logEvent("EVICTED", itemWithKey.key, map[string]interface{}{
			"policy": "FIFO",
		})
	}

	dc.updateStats(func(stats *CacheStats) {
		atomic.AddInt64(&stats.Evictions, int64(evicted))
		atomic.AddInt64(&stats.TotalItems, -int64(evicted))
		atomic.AddInt64(&stats.TotalSize, -freedSpace)
	})

	if freedSpace < neededSpace {
		return ErrCacheFull
	}

	return nil
}

func (dc *DistributedCache) evictTTL(neededSpace int64) error {
	// First try to evict expired items
	dc.cleanupExpiredItems()

	// If still need space, fall back to LRU
	if dc.getCurrentSize()+neededSpace > dc.config.MaxSize {
		return dc.evictLRU(neededSpace)
	}

	return nil
}

func (dc *DistributedCache) evictRandom(neededSpace int64) error {
	dc.mutex.Lock()
	defer dc.mutex.Unlock()

	freedSpace := int64(0)
	evicted := 0

	for key, item := range dc.localCache {
		if freedSpace >= neededSpace {
			break
		}

		delete(dc.localCache, key)
		freedSpace += item.Size
		evicted++

		dc.logEvent("EVICTED", key, map[string]interface{}{
			"policy": "Random",
		})
	}

	dc.updateStats(func(stats *CacheStats) {
		atomic.AddInt64(&stats.Evictions, int64(evicted))
		atomic.AddInt64(&stats.TotalItems, -int64(evicted))
		atomic.AddInt64(&stats.TotalSize, -freedSpace)
	})

	if freedSpace < neededSpace {
		return ErrCacheFull
	}

	return nil
}

func (dc *DistributedCache) cleanupExpiredItems() {
	dc.mutex.Lock()
	defer dc.mutex.Unlock()

	expiredKeys := make([]string, 0)
	freedSpace := int64(0)

	for key, item := range dc.localCache {
		if item.IsExpired() {
			expiredKeys = append(expiredKeys, key)
			freedSpace += item.Size
		}
	}

	for _, key := range expiredKeys {
		delete(dc.localCache, key)
		dc.logEvent("EXPIRED", key, nil)
	}

	if len(expiredKeys) > 0 {
		dc.updateStats(func(stats *CacheStats) {
			atomic.AddInt64(&stats.TotalItems, -int64(len(expiredKeys)))
			atomic.AddInt64(&stats.TotalSize, -freedSpace)
		})
	}
}

func (dc *DistributedCache) updateStats(fn func(*CacheStats)) {
	dc.statsMutex.Lock()
	defer dc.statsMutex.Unlock()
	fn(dc.stats)
}

func (dc *DistributedCache) updateAverageAccessTime(duration time.Duration) {
	dc.updateStats(func(stats *CacheStats) {
		// Simple moving average
		totalOperations := stats.Hits + stats.Misses
		if totalOperations == 0 {
			stats.AverageAccessTime = duration.Seconds()
		} else {
			stats.AverageAccessTime = (stats.AverageAccessTime*float64(totalOperations-1) + duration.Seconds()) / float64(totalOperations)
		}
	})
}

func (dc *DistributedCache) logEvent(eventType, key string, metadata map[string]interface{}) {
	if !dc.config.MonitoringEnabled {
		return
	}

	event := CacheEvent{
		Type:      eventType,
		Key:       key,
		NodeID:    dc.nodeID,
		Timestamp: time.Now(),
		Metadata:  metadata,
	}

	dc.eventLogMutex.Lock()
	defer dc.eventLogMutex.Unlock()

	dc.eventLog = append(dc.eventLog, event)

	// Keep only last 1000 events
	if len(dc.eventLog) > 1000 {
		dc.eventLog = dc.eventLog[len(dc.eventLog)-1000:]
	}
}

func (dc *DistributedCache) matchPattern(key, pattern string) bool {
	// Simple pattern matching - only supports * wildcard
	if pattern == "*" {
		return true
	}
	
	// For now, just check exact match if no wildcard
	return key == pattern
}

// Background loops

func (dc *DistributedCache) heartbeatLoop(ctx context.Context) {
	ticker := time.NewTicker(dc.config.HeartbeatInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-dc.shutdownChan:
			return
		case <-ticker.C:
			dc.sendHeartbeat()
		}
	}
}

func (dc *DistributedCache) evictionLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Minute) // Run every minute
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-dc.shutdownChan:
			return
		case <-ticker.C:
			dc.cleanupExpiredItems()
		}
	}
}

func (dc *DistributedCache) monitoringLoop(ctx context.Context) {
	if !dc.config.MonitoringEnabled {
		return
	}

	ticker := time.NewTicker(time.Minute * 5) // Monitor every 5 minutes
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-dc.shutdownChan:
			return
		case <-ticker.C:
			dc.collectMetrics()
		}
	}
}

func (dc *DistributedCache) persistenceLoop(ctx context.Context) {
	if !dc.config.PersistenceEnabled {
		return
	}

	ticker := time.NewTicker(dc.config.BackupInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-dc.shutdownChan:
			return
		case <-ticker.C:
			dc.persistData()
		}
	}
}

func (dc *DistributedCache) sendHeartbeat() {
	dc.nodesMutex.Lock()
	defer dc.nodesMutex.Unlock()

	if node, exists := dc.nodes[dc.nodeID]; exists {
		node.LastHeartbeat = time.Now()
		node.LoadFactor = dc.calculateLoadFactor()
	}
}

func (dc *DistributedCache) calculateLoadFactor() float64 {
	if dc.config.MaxSize == 0 {
		return 0.0
	}
	return float64(dc.getCurrentSize()) / float64(dc.config.MaxSize)
}

func (dc *DistributedCache) collectMetrics() {
	// Log current metrics
	stats := dc.GetStats()
	log.Printf("Cache Metrics - Hits: %d, Misses: %d, Hit Ratio: %.2f, Items: %d, Size: %d",
		stats.Hits, stats.Misses, stats.HitRatio(), stats.TotalItems, stats.TotalSize)
}

func (dc *DistributedCache) persistData() {
	// This would implement actual persistence to disk
	log.Printf("Persisting cache data for node %s", dc.nodeID)
}

// Replication methods (simplified implementations)

func (dc *DistributedCache) replicateSet(ctx context.Context, key string, value interface{}, ttl time.Duration) {
	nodes, err := dc.hashRing.GetNodes(key, dc.config.ReplicationFactor)
	if err != nil {
		log.Printf("Failed to get replication nodes for key %s: %v", key, err)
		return
	}

	for _, nodeID := range nodes {
		if nodeID != dc.nodeID {
			// In a real implementation, this would send over network
			dc.logEvent("REPLICATE_SET", key, map[string]interface{}{
				"target_node": nodeID,
			})
		}
	}
}

func (dc *DistributedCache) replicateDelete(ctx context.Context, key string) {
	nodes, err := dc.hashRing.GetNodes(key, dc.config.ReplicationFactor)
	if err != nil {
		return
	}

	for _, nodeID := range nodes {
		if nodeID != dc.nodeID {
			dc.logEvent("REPLICATE_DELETE", key, map[string]interface{}{
				"target_node": nodeID,
			})
		}
	}
}

func (dc *DistributedCache) replicateFlush(ctx context.Context) {
	dc.nodesMutex.RLock()
	defer dc.nodesMutex.RUnlock()

	for nodeID := range dc.nodes {
		if nodeID != dc.nodeID {
			dc.logEvent("REPLICATE_FLUSH", "", map[string]interface{}{
				"target_node": nodeID,
			})
		}
	}
}

func (dc *DistributedCache) getFromOtherNodes(ctx context.Context, key string) (interface{}, error) {
	// In a real implementation, this would query other nodes over network
	return nil, ErrKeyNotFound
}

func validateConfig(config *CacheConfig) error {
	if config.MaxSize <= 0 {
		return errors.New("max size must be positive")
	}
	if config.ReplicationFactor < 1 {
		return errors.New("replication factor must be at least 1")
	}
	if config.MaxNodes < 1 {
		return errors.New("max nodes must be at least 1")
	}
	if config.HeartbeatInterval <= 0 {
		return errors.New("heartbeat interval must be positive")
	}
	if config.RequestTimeout <= 0 {
		return errors.New("request timeout must be positive")
	}
	return nil
}