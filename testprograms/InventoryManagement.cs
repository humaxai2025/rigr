// Complex Inventory Management System - C#
// This demonstrates advanced C# patterns for comprehensive test case generation

using System;
using System.Collections.Generic;
using System.Collections.Concurrent;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using System.ComponentModel.DataAnnotations;

namespace InventoryManagement
{
    public enum ProductCategory
    {
        Electronics,
        Clothing,
        Books,
        HomeAndGarden,
        Sports,
        Automotive,
        Health,
        Toys
    }

    public enum StockMovementType
    {
        Inbound,
        Outbound,
        Transfer,
        Adjustment,
        Damaged,
        Returned,
        Expired
    }

    public enum AlertType
    {
        LowStock,
        OverStock,
        Expiration,
        Reorder,
        Quality,
        Theft
    }

    public enum SupplierStatus
    {
        Active,
        Inactive,
        Suspended,
        Preferred,
        Blacklisted
    }

    public class ValidationException : Exception
    {
        public string Field { get; }
        
        public ValidationException(string field, string message) : base(message)
        {
            Field = field;
        }
    }

    public class InsufficientStockException : Exception
    {
        public string ProductId { get; }
        public int RequestedQuantity { get; }
        public int AvailableQuantity { get; }

        public InsufficientStockException(string productId, int requestedQuantity, int availableQuantity)
            : base($"Insufficient stock for product {productId}: requested {requestedQuantity}, available {availableQuantity}")
        {
            ProductId = productId;
            RequestedQuantity = requestedQuantity;
            AvailableQuantity = availableQuantity;
        }
    }

    public class Product
    {
        public string Id { get; set; }
        public string Name { get; set; }
        public string Description { get; set; }
        public string SKU { get; set; }
        public ProductCategory Category { get; set; }
        public decimal Cost { get; set; }
        public decimal Price { get; set; }
        public string SupplierId { get; set; }
        public int MinimumStock { get; set; }
        public int MaximumStock { get; set; }
        public int ReorderPoint { get; set; }
        public int ReorderQuantity { get; set; }
        public DateTime? ExpirationDate { get; set; }
        public bool IsPerishable { get; set; }
        public Dictionary<string, object> Attributes { get; set; } = new Dictionary<string, object>();
        public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
        public DateTime UpdatedAt { get; set; } = DateTime.UtcNow;
        public bool IsActive { get; set; } = true;
    }

    public class StockItem
    {
        public string Id { get; set; }
        public string ProductId { get; set; }
        public string LocationId { get; set; }
        public int Quantity { get; set; }
        public int ReservedQuantity { get; set; }
        public decimal UnitCost { get; set; }
        public string BatchNumber { get; set; }
        public DateTime? ExpirationDate { get; set; }
        public DateTime LastUpdated { get; set; } = DateTime.UtcNow;

        public int AvailableQuantity => Quantity - ReservedQuantity;
    }

    public class StockMovement
    {
        public string Id { get; set; }
        public string ProductId { get; set; }
        public string LocationId { get; set; }
        public StockMovementType Type { get; set; }
        public int Quantity { get; set; }
        public decimal UnitCost { get; set; }
        public string Reference { get; set; }
        public string Notes { get; set; }
        public DateTime Timestamp { get; set; } = DateTime.UtcNow;
        public string UserId { get; set; }
        public Dictionary<string, object> Metadata { get; set; } = new Dictionary<string, object>();
    }

    public class Supplier
    {
        public string Id { get; set; }
        public string Name { get; set; }
        public string ContactPerson { get; set; }
        public string Email { get; set; }
        public string Phone { get; set; }
        public string Address { get; set; }
        public SupplierStatus Status { get; set; } = SupplierStatus.Active;
        public decimal Rating { get; set; }
        public int LeadTimeDays { get; set; }
        public decimal MinimumOrderValue { get; set; }
        public List<string> ProductCategories { get; set; } = new List<string>();
        public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
        public DateTime LastOrderDate { get; set; }
    }

    public class Location
    {
        public string Id { get; set; }
        public string Name { get; set; }
        public string Type { get; set; } // Warehouse, Store, Distribution Center
        public string Address { get; set; }
        public decimal Capacity { get; set; }
        public decimal CurrentUtilization { get; set; }
        public bool IsActive { get; set; } = true;
        public Dictionary<string, object> Configuration { get; set; } = new Dictionary<string, object>();
    }

    public class Alert
    {
        public string Id { get; set; }
        public AlertType Type { get; set; }
        public string Title { get; set; }
        public string Message { get; set; }
        public string ProductId { get; set; }
        public string LocationId { get; set; }
        public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
        public bool IsResolved { get; set; }
        public DateTime? ResolvedAt { get; set; }
        public string ResolvedBy { get; set; }
        public int Priority { get; set; } // 1-5, where 1 is highest priority
    }

    public class PurchaseOrder
    {
        public string Id { get; set; }
        public string SupplierId { get; set; }
        public List<PurchaseOrderItem> Items { get; set; } = new List<PurchaseOrderItem>();
        public decimal TotalAmount { get; set; }
        public DateTime OrderDate { get; set; } = DateTime.UtcNow;
        public DateTime? ExpectedDeliveryDate { get; set; }
        public DateTime? ActualDeliveryDate { get; set; }
        public string Status { get; set; } = "Pending";
        public string Notes { get; set; }
    }

    public class PurchaseOrderItem
    {
        public string ProductId { get; set; }
        public int Quantity { get; set; }
        public decimal UnitCost { get; set; }
        public decimal TotalCost => Quantity * UnitCost;
    }

    public class InventoryManagementSystem
    {
        private readonly ConcurrentDictionary<string, Product> _products;
        private readonly ConcurrentDictionary<string, StockItem> _stockItems;
        private readonly ConcurrentDictionary<string, Supplier> _suppliers;
        private readonly ConcurrentDictionary<string, Location> _locations;
        private readonly List<StockMovement> _stockMovements;
        private readonly List<Alert> _alerts;
        private readonly List<PurchaseOrder> _purchaseOrders;
        private readonly object _lockObject = new object();
        private long _transactionCounter = 0;

        public InventoryManagementSystem()
        {
            _products = new ConcurrentDictionary<string, Product>();
            _stockItems = new ConcurrentDictionary<string, StockItem>();
            _suppliers = new ConcurrentDictionary<string, Supplier>();
            _locations = new ConcurrentDictionary<string, Location>();
            _stockMovements = new List<StockMovement>();
            _alerts = new List<Alert>();
            _purchaseOrders = new List<PurchaseOrder>();
        }

        // Product Management
        public async Task<Product> CreateProductAsync(Product product)
        {
            await Task.Run(() => ValidateProduct(product));

            if (_products.ContainsKey(product.Id))
                throw new ValidationException("Id", $"Product with ID {product.Id} already exists");

            // Check for duplicate SKU
            if (_products.Values.Any(p => p.SKU == product.SKU && p.IsActive))
                throw new ValidationException("SKU", $"Product with SKU {product.SKU} already exists");

            product.CreatedAt = DateTime.UtcNow;
            product.UpdatedAt = DateTime.UtcNow;

            _products.TryAdd(product.Id, product);
            return product;
        }

        public Product GetProduct(string productId)
        {
            if (!_products.TryGetValue(productId, out Product product))
                throw new ArgumentException($"Product {productId} not found");

            return product;
        }

        public async Task<Product> UpdateProductAsync(string productId, Product updatedProduct)
        {
            var existingProduct = GetProduct(productId);
            
            await Task.Run(() => ValidateProduct(updatedProduct));

            // Check SKU uniqueness
            if (updatedProduct.SKU != existingProduct.SKU && 
                _products.Values.Any(p => p.SKU == updatedProduct.SKU && p.Id != productId && p.IsActive))
                throw new ValidationException("SKU", $"Product with SKU {updatedProduct.SKU} already exists");

            updatedProduct.Id = productId;
            updatedProduct.CreatedAt = existingProduct.CreatedAt;
            updatedProduct.UpdatedAt = DateTime.UtcNow;

            _products.TryUpdate(productId, updatedProduct, existingProduct);
            return updatedProduct;
        }

        public List<Product> SearchProducts(string searchTerm = null, ProductCategory? category = null, 
                                          bool? isActive = null, decimal? minPrice = null, decimal? maxPrice = null)
        {
            var query = _products.Values.AsQueryable();

            if (!string.IsNullOrEmpty(searchTerm))
            {
                searchTerm = searchTerm.ToLower();
                query = query.Where(p => p.Name.ToLower().Contains(searchTerm) || 
                                       p.Description.ToLower().Contains(searchTerm) ||
                                       p.SKU.ToLower().Contains(searchTerm));
            }

            if (category.HasValue)
                query = query.Where(p => p.Category == category.Value);

            if (isActive.HasValue)
                query = query.Where(p => p.IsActive == isActive.Value);

            if (minPrice.HasValue)
                query = query.Where(p => p.Price >= minPrice.Value);

            if (maxPrice.HasValue)
                query = query.Where(p => p.Price <= maxPrice.Value);

            return query.OrderBy(p => p.Name).ToList();
        }

        // Stock Management
        public async Task<StockItem> AddStockAsync(string productId, string locationId, int quantity, 
                                                  decimal unitCost, string batchNumber = null, DateTime? expirationDate = null)
        {
            if (quantity <= 0)
                throw new ValidationException("quantity", "Quantity must be positive");

            if (unitCost < 0)
                throw new ValidationException("unitCost", "Unit cost cannot be negative");

            var product = GetProduct(productId);
            var location = GetLocation(locationId);

            var stockItemId = GenerateId("STK");
            var stockItem = new StockItem
            {
                Id = stockItemId,
                ProductId = productId,
                LocationId = locationId,
                Quantity = quantity,
                UnitCost = unitCost,
                BatchNumber = batchNumber,
                ExpirationDate = expirationDate
            };

            _stockItems.TryAdd(stockItemId, stockItem);

            await LogStockMovementAsync(productId, locationId, StockMovementType.Inbound, 
                                       quantity, unitCost, $"Initial stock addition - Batch: {batchNumber}");

            await CheckStockLevelsAsync(productId, locationId);
            return stockItem;
        }

        public async Task ReserveStockAsync(string productId, string locationId, int quantity)
        {
            if (quantity <= 0)
                throw new ValidationException("quantity", "Quantity must be positive");

            var availableStock = GetAvailableStock(productId, locationId);
            if (availableStock < quantity)
                throw new InsufficientStockException(productId, quantity, availableStock);

            var stockItems = _stockItems.Values
                .Where(s => s.ProductId == productId && s.LocationId == locationId && s.AvailableQuantity > 0)
                .OrderBy(s => s.ExpirationDate ?? DateTime.MaxValue)
                .ToList();

            int remainingToReserve = quantity;
            foreach (var stockItem in stockItems)
            {
                if (remainingToReserve <= 0) break;

                int reserveFromThis = Math.Min(remainingToReserve, stockItem.AvailableQuantity);
                stockItem.ReservedQuantity += reserveFromThis;
                stockItem.LastUpdated = DateTime.UtcNow;
                remainingToReserve -= reserveFromThis;
            }

            await LogStockMovementAsync(productId, locationId, StockMovementType.Outbound, 
                                       quantity, 0, "Stock reserved");
        }

        public async Task ReleaseReservedStockAsync(string productId, string locationId, int quantity)
        {
            if (quantity <= 0)
                throw new ValidationException("quantity", "Quantity must be positive");

            var stockItems = _stockItems.Values
                .Where(s => s.ProductId == productId && s.LocationId == locationId && s.ReservedQuantity > 0)
                .ToList();

            int remainingToRelease = quantity;
            foreach (var stockItem in stockItems)
            {
                if (remainingToRelease <= 0) break;

                int releaseFromThis = Math.Min(remainingToRelease, stockItem.ReservedQuantity);
                stockItem.ReservedQuantity -= releaseFromThis;
                stockItem.LastUpdated = DateTime.UtcNow;
                remainingToRelease -= releaseFromThis;
            }

            await LogStockMovementAsync(productId, locationId, StockMovementType.Adjustment, 
                                       quantity, 0, "Reserved stock released");
        }

        public async Task TransferStockAsync(string productId, string fromLocationId, string toLocationId, int quantity)
        {
            if (quantity <= 0)
                throw new ValidationException("quantity", "Quantity must be positive");

            if (fromLocationId == toLocationId)
                throw new ValidationException("location", "Cannot transfer to the same location");

            var fromLocation = GetLocation(fromLocationId);
            var toLocation = GetLocation(toLocationId);

            var availableStock = GetAvailableStock(productId, fromLocationId);
            if (availableStock < quantity)
                throw new InsufficientStockException(productId, quantity, availableStock);

            // Find stock items to transfer (FIFO)
            var stockItems = _stockItems.Values
                .Where(s => s.ProductId == productId && s.LocationId == fromLocationId && s.AvailableQuantity > 0)
                .OrderBy(s => s.ExpirationDate ?? DateTime.MaxValue)
                .ToList();

            int remainingToTransfer = quantity;
            decimal averageCost = 0;
            decimal totalCost = 0;

            foreach (var stockItem in stockItems)
            {
                if (remainingToTransfer <= 0) break;

                int transferFromThis = Math.Min(remainingToTransfer, stockItem.AvailableQuantity);
                stockItem.Quantity -= transferFromThis;
                stockItem.LastUpdated = DateTime.UtcNow;

                totalCost += transferFromThis * stockItem.UnitCost;
                remainingToTransfer -= transferFromThis;
            }

            averageCost = quantity > 0 ? totalCost / quantity : 0;

            // Create new stock item at destination
            await AddStockAsync(productId, toLocationId, quantity, averageCost);

            await LogStockMovementAsync(productId, fromLocationId, StockMovementType.Transfer, 
                                       -quantity, averageCost, $"Transfer to {toLocationId}");
            await LogStockMovementAsync(productId, toLocationId, StockMovementType.Transfer, 
                                       quantity, averageCost, $"Transfer from {fromLocationId}");

            await CheckStockLevelsAsync(productId, fromLocationId);
            await CheckStockLevelsAsync(productId, toLocationId);
        }

        // Analytics and Reporting
        public Dictionary<string, object> GenerateInventoryReport(DateTime? startDate = null, DateTime? endDate = null)
        {
            startDate ??= DateTime.UtcNow.AddDays(-30);
            endDate ??= DateTime.UtcNow;

            var movements = _stockMovements
                .Where(m => m.Timestamp >= startDate && m.Timestamp <= endDate)
                .ToList();

            var totalValue = _stockItems.Values
                .Sum(s => s.Quantity * s.UnitCost);

            var categoryBreakdown = _products.Values
                .Where(p => p.IsActive)
                .GroupBy(p => p.Category)
                .ToDictionary(
                    g => g.Key.ToString(),
                    g => new
                    {
                        ProductCount = g.Count(),
                        TotalValue = g.Sum(p => GetTotalStock(p.Id) * p.Cost)
                    }
                );

            var lowStockProducts = _products.Values
                .Where(p => p.IsActive && GetTotalStock(p.Id) <= p.ReorderPoint)
                .Select(p => new { Product = p, CurrentStock = GetTotalStock(p.Id) })
                .ToList();

            var expiringProducts = _stockItems.Values
                .Where(s => s.ExpirationDate.HasValue && s.ExpirationDate <= DateTime.UtcNow.AddDays(30))
                .GroupBy(s => s.ProductId)
                .Select(g => new
                {
                    ProductId = g.Key,
                    Product = _products.GetValueOrDefault(g.Key),
                    ExpiringQuantity = g.Sum(s => s.Quantity),
                    NearestExpiration = g.Min(s => s.ExpirationDate)
                })
                .ToList();

            return new Dictionary<string, object>
            {
                ["reportPeriod"] = new { StartDate = startDate, EndDate = endDate },
                ["totalInventoryValue"] = totalValue,
                ["totalProducts"] = _products.Count,
                ["totalLocations"] = _locations.Count,
                ["totalMovements"] = movements.Count,
                ["categoryBreakdown"] = categoryBreakdown,
                ["lowStockProducts"] = lowStockProducts,
                ["expiringProducts"] = expiringProducts,
                ["activeAlerts"] = _alerts.Count(a => !a.IsResolved)
            };
        }

        public async Task<List<PurchaseOrder>> GenerateAutomaticPurchaseOrdersAsync()
        {
            var purchaseOrders = new List<PurchaseOrder>();
            
            var lowStockProducts = _products.Values
                .Where(p => p.IsActive && GetTotalStock(p.Id) <= p.ReorderPoint)
                .GroupBy(p => p.SupplierId)
                .ToList();

            foreach (var supplierGroup in lowStockProducts)
            {
                var supplier = _suppliers.GetValueOrDefault(supplierGroup.Key);
                if (supplier == null || supplier.Status != SupplierStatus.Active) continue;

                var orderId = GenerateId("PO");
                var purchaseOrder = new PurchaseOrder
                {
                    Id = orderId,
                    SupplierId = supplier.Id,
                    ExpectedDeliveryDate = DateTime.UtcNow.AddDays(supplier.LeadTimeDays)
                };

                foreach (var product in supplierGroup)
                {
                    var orderQuantity = Math.Max(product.ReorderQuantity, product.MinimumStock - GetTotalStock(product.Id));
                    purchaseOrder.Items.Add(new PurchaseOrderItem
                    {
                        ProductId = product.Id,
                        Quantity = orderQuantity,
                        UnitCost = product.Cost
                    });
                }

                purchaseOrder.TotalAmount = purchaseOrder.Items.Sum(i => i.TotalCost);

                if (purchaseOrder.TotalAmount >= supplier.MinimumOrderValue)
                {
                    _purchaseOrders.Add(purchaseOrder);
                    purchaseOrders.Add(purchaseOrder);

                    await CreateAlertAsync(AlertType.Reorder, "Automatic Purchase Order Created",
                                         $"Purchase order {orderId} created for supplier {supplier.Name}");
                }
            }

            return purchaseOrders;
        }

        // Helper Methods
        private void ValidateProduct(Product product)
        {
            if (string.IsNullOrWhiteSpace(product.Id))
                throw new ValidationException("Id", "Product ID is required");

            if (string.IsNullOrWhiteSpace(product.Name))
                throw new ValidationException("Name", "Product name is required");

            if (string.IsNullOrWhiteSpace(product.SKU))
                throw new ValidationException("SKU", "Product SKU is required");

            if (product.Cost < 0)
                throw new ValidationException("Cost", "Product cost cannot be negative");

            if (product.Price < 0)
                throw new ValidationException("Price", "Product price cannot be negative");

            if (product.MinimumStock < 0)
                throw new ValidationException("MinimumStock", "Minimum stock cannot be negative");

            if (product.MaximumStock < product.MinimumStock)
                throw new ValidationException("MaximumStock", "Maximum stock cannot be less than minimum stock");

            if (product.ReorderPoint < 0)
                throw new ValidationException("ReorderPoint", "Reorder point cannot be negative");

            if (product.ReorderQuantity <= 0)
                throw new ValidationException("ReorderQuantity", "Reorder quantity must be positive");
        }

        private Location GetLocation(string locationId)
        {
            if (!_locations.TryGetValue(locationId, out Location location))
                throw new ArgumentException($"Location {locationId} not found");

            return location;
        }

        private int GetTotalStock(string productId)
        {
            return _stockItems.Values
                .Where(s => s.ProductId == productId)
                .Sum(s => s.Quantity);
        }

        private int GetAvailableStock(string productId, string locationId)
        {
            return _stockItems.Values
                .Where(s => s.ProductId == productId && s.LocationId == locationId)
                .Sum(s => s.AvailableQuantity);
        }

        private async Task LogStockMovementAsync(string productId, string locationId, StockMovementType type, 
                                               int quantity, decimal unitCost, string notes)
        {
            var movement = new StockMovement
            {
                Id = GenerateId("MOV"),
                ProductId = productId,
                LocationId = locationId,
                Type = type,
                Quantity = quantity,
                UnitCost = unitCost,
                Notes = notes,
                UserId = "SYSTEM"
            };

            await Task.Run(() => _stockMovements.Add(movement));
        }

        private async Task CheckStockLevelsAsync(string productId, string locationId)
        {
            var product = GetProduct(productId);
            var currentStock = GetAvailableStock(productId, locationId);

            if (currentStock <= product.ReorderPoint)
            {
                await CreateAlertAsync(AlertType.LowStock, "Low Stock Alert",
                                     $"Product {product.Name} is below reorder point. Current: {currentStock}, Reorder Point: {product.ReorderPoint}",
                                     productId, locationId, 2);
            }
            else if (currentStock >= product.MaximumStock)
            {
                await CreateAlertAsync(AlertType.OverStock, "Overstock Alert",
                                     $"Product {product.Name} exceeds maximum stock. Current: {currentStock}, Maximum: {product.MaximumStock}",
                                     productId, locationId, 3);
            }
        }

        private async Task CreateAlertAsync(AlertType type, string title, string message, 
                                          string productId = null, string locationId = null, int priority = 3)
        {
            var alert = new Alert
            {
                Id = GenerateId("ALT"),
                Type = type,
                Title = title,
                Message = message,
                ProductId = productId,
                LocationId = locationId,
                Priority = priority
            };

            await Task.Run(() => _alerts.Add(alert));
        }

        private string GenerateId(string prefix)
        {
            lock (_lockObject)
            {
                return $"{prefix}-{DateTime.UtcNow:yyyyMMdd}-{++_transactionCounter:D6}";
            }
        }

        // Public methods for adding suppliers and locations (for testing)
        public void AddSupplier(Supplier supplier)
        {
            _suppliers.TryAdd(supplier.Id, supplier);
        }

        public void AddLocation(Location location)
        {
            _locations.TryAdd(location.Id, location);
        }
    }
}