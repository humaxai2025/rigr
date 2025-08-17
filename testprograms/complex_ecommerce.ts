// Complex E-commerce System - TypeScript
// This demonstrates advanced TypeScript patterns for comprehensive test case generation

interface Product {
    id: string;
    name: string;
    price: number;
    category: string;
    stock: number;
    discount?: number;
    metadata?: Record<string, any>;
}

interface Customer {
    id: string;
    email: string;
    name: string;
    address: Address;
    membershipLevel: MembershipLevel;
    creditLimit: number;
}

interface Address {
    street: string;
    city: string;
    state: string;
    zipCode: string;
    country: string;
}

enum MembershipLevel {
    BRONZE = "bronze",
    SILVER = "silver", 
    GOLD = "gold",
    PLATINUM = "platinum"
}

enum OrderStatus {
    PENDING = "pending",
    CONFIRMED = "confirmed",
    PROCESSING = "processing",
    SHIPPED = "shipped",
    DELIVERED = "delivered",
    CANCELLED = "cancelled",
    REFUNDED = "refunded"
}

interface OrderItem {
    product: Product;
    quantity: number;
    unitPrice: number;
    discount: number;
}

interface Order {
    id: string;
    customer: Customer;
    items: OrderItem[];
    status: OrderStatus;
    totalAmount: number;
    taxAmount: number;
    shippingCost: number;
    createdAt: Date;
    updatedAt: Date;
}

class ValidationError extends Error {
    constructor(public field: string, message: string) {
        super(message);
        this.name = "ValidationError";
    }
}

class InsufficientStockError extends Error {
    constructor(public productId: string, public requested: number, public available: number) {
        super(`Insufficient stock for product ${productId}: requested ${requested}, available ${available}`);
        this.name = "InsufficientStockError";
    }
}

class PaymentError extends Error {
    constructor(public code: string, message: string) {
        super(message);
        this.name = "PaymentError";
    }
}

export class ECommerceSystem {
    private products: Map<string, Product> = new Map();
    private customers: Map<string, Customer> = new Map();
    private orders: Map<string, Order> = new Map();
    private readonly taxRate = 0.08;
    private readonly shippingRates = new Map([
        ["standard", 5.99],
        ["express", 12.99],
        ["overnight", 24.99]
    ]);

    // Product Management
    addProduct(product: Product): void {
        this.validateProduct(product);
        
        if (this.products.has(product.id)) {
            throw new ValidationError("id", `Product with ID ${product.id} already exists`);
        }

        this.products.set(product.id, { ...product });
    }

    updateProductStock(productId: string, newStock: number): void {
        if (newStock < 0) {
            throw new ValidationError("stock", "Stock cannot be negative");
        }

        const product = this.products.get(productId);
        if (!product) {
            throw new ValidationError("productId", `Product ${productId} not found`);
        }

        product.stock = newStock;
    }

    getProduct(productId: string): Product | null {
        return this.products.get(productId) || null;
    }

    searchProducts(criteria: {
        category?: string;
        minPrice?: number;
        maxPrice?: number;
        inStock?: boolean;
    }): Product[] {
        const results: Product[] = [];
        
        for (const product of this.products.values()) {
            if (criteria.category && product.category !== criteria.category) continue;
            if (criteria.minPrice && product.price < criteria.minPrice) continue;
            if (criteria.maxPrice && product.price > criteria.maxPrice) continue;
            if (criteria.inStock && product.stock <= 0) continue;
            
            results.push({ ...product });
        }

        return results.sort((a, b) => a.name.localeCompare(b.name));
    }

    // Customer Management
    registerCustomer(customer: Customer): void {
        this.validateCustomer(customer);
        
        if (this.customers.has(customer.id)) {
            throw new ValidationError("id", `Customer with ID ${customer.id} already exists`);
        }

        // Check for duplicate email
        for (const existingCustomer of this.customers.values()) {
            if (existingCustomer.email === customer.email) {
                throw new ValidationError("email", `Email ${customer.email} is already registered`);
            }
        }

        this.customers.set(customer.id, { ...customer });
    }

    updateCustomerMembership(customerId: string, level: MembershipLevel): void {
        const customer = this.customers.get(customerId);
        if (!customer) {
            throw new ValidationError("customerId", `Customer ${customerId} not found`);
        }

        customer.membershipLevel = level;
        
        // Update credit limit based on membership
        customer.creditLimit = this.calculateCreditLimit(level);
    }

    // Order Processing
    async createOrder(customerId: string, items: Array<{productId: string, quantity: number}>): Promise<Order> {
        const customer = this.customers.get(customerId);
        if (!customer) {
            throw new ValidationError("customerId", `Customer ${customerId} not found`);
        }

        if (!items || items.length === 0) {
            throw new ValidationError("items", "Order must contain at least one item");
        }

        const orderItems: OrderItem[] = [];
        let subtotal = 0;

        // Validate and process each item
        for (const item of items) {
            const product = this.products.get(item.productId);
            if (!product) {
                throw new ValidationError("productId", `Product ${item.productId} not found`);
            }

            if (item.quantity <= 0) {
                throw new ValidationError("quantity", "Quantity must be positive");
            }

            if (product.stock < item.quantity) {
                throw new InsufficientStockError(item.productId, item.quantity, product.stock);
            }

            const discount = this.calculateDiscount(product, customer.membershipLevel, item.quantity);
            const unitPrice = product.price * (1 - discount);
            
            orderItems.push({
                product: { ...product },
                quantity: item.quantity,
                unitPrice,
                discount
            });

            subtotal += unitPrice * item.quantity;
        }

        const taxAmount = subtotal * this.taxRate;
        const shippingCost = this.calculateShippingCost(subtotal, customer.membershipLevel);
        const totalAmount = subtotal + taxAmount + shippingCost;

        // Check credit limit
        if (totalAmount > customer.creditLimit) {
            throw new PaymentError("CREDIT_LIMIT_EXCEEDED", 
                `Order total $${totalAmount} exceeds credit limit $${customer.creditLimit}`);
        }

        const order: Order = {
            id: this.generateOrderId(),
            customer: { ...customer },
            items: orderItems,
            status: OrderStatus.PENDING,
            totalAmount,
            taxAmount,
            shippingCost,
            createdAt: new Date(),
            updatedAt: new Date()
        };

        this.orders.set(order.id, order);

        // Reserve stock
        for (const item of orderItems) {
            const product = this.products.get(item.product.id)!;
            product.stock -= item.quantity;
        }

        return { ...order };
    }

    async processPayment(orderId: string, paymentMethod: string): Promise<void> {
        const order = this.orders.get(orderId);
        if (!order) {
            throw new ValidationError("orderId", `Order ${orderId} not found`);
        }

        if (order.status !== OrderStatus.PENDING) {
            throw new ValidationError("status", `Cannot process payment for order in ${order.status} status`);
        }

        // Simulate payment processing
        await this.simulatePaymentProcessing(order.totalAmount, paymentMethod);

        order.status = OrderStatus.CONFIRMED;
        order.updatedAt = new Date();
    }

    cancelOrder(orderId: string, reason: string): void {
        const order = this.orders.get(orderId);
        if (!order) {
            throw new ValidationError("orderId", `Order ${orderId} not found`);
        }

        if (order.status === OrderStatus.SHIPPED || order.status === OrderStatus.DELIVERED) {
            throw new ValidationError("status", `Cannot cancel order in ${order.status} status`);
        }

        // Restore stock
        for (const item of order.items) {
            const product = this.products.get(item.product.id);
            if (product) {
                product.stock += item.quantity;
            }
        }

        order.status = OrderStatus.CANCELLED;
        order.updatedAt = new Date();
    }

    // Analytics
    getOrderAnalytics(startDate: Date, endDate: Date): {
        totalOrders: number;
        totalRevenue: number;
        averageOrderValue: number;
        topProducts: Array<{productId: string, quantity: number, revenue: number}>;
        customerMetrics: {[key in MembershipLevel]: {orders: number, revenue: number}};
    } {
        const orders = Array.from(this.orders.values()).filter(
            order => order.createdAt >= startDate && order.createdAt <= endDate
        );

        const totalOrders = orders.length;
        const totalRevenue = orders.reduce((sum, order) => sum + order.totalAmount, 0);
        const averageOrderValue = totalOrders > 0 ? totalRevenue / totalOrders : 0;

        // Product analytics
        const productStats = new Map<string, {quantity: number, revenue: number}>();
        for (const order of orders) {
            for (const item of order.items) {
                const existing = productStats.get(item.product.id) || {quantity: 0, revenue: 0};
                existing.quantity += item.quantity;
                existing.revenue += item.unitPrice * item.quantity;
                productStats.set(item.product.id, existing);
            }
        }

        const topProducts = Array.from(productStats.entries())
            .map(([productId, stats]) => ({productId, ...stats}))
            .sort((a, b) => b.revenue - a.revenue)
            .slice(0, 10);

        // Customer metrics
        const customerMetrics = {
            [MembershipLevel.BRONZE]: {orders: 0, revenue: 0},
            [MembershipLevel.SILVER]: {orders: 0, revenue: 0},
            [MembershipLevel.GOLD]: {orders: 0, revenue: 0},
            [MembershipLevel.PLATINUM]: {orders: 0, revenue: 0}
        };

        for (const order of orders) {
            const level = order.customer.membershipLevel;
            customerMetrics[level].orders++;
            customerMetrics[level].revenue += order.totalAmount;
        }

        return {
            totalOrders,
            totalRevenue,
            averageOrderValue,
            topProducts,
            customerMetrics
        };
    }

    // Private helper methods
    private validateProduct(product: Product): void {
        if (!product.id || product.id.trim() === "") {
            throw new ValidationError("id", "Product ID is required");
        }
        if (!product.name || product.name.trim() === "") {
            throw new ValidationError("name", "Product name is required");
        }
        if (product.price < 0) {
            throw new ValidationError("price", "Product price cannot be negative");
        }
        if (product.stock < 0) {
            throw new ValidationError("stock", "Product stock cannot be negative");
        }
        if (product.discount && (product.discount < 0 || product.discount > 1)) {
            throw new ValidationError("discount", "Discount must be between 0 and 1");
        }
    }

    private validateCustomer(customer: Customer): void {
        if (!customer.id || customer.id.trim() === "") {
            throw new ValidationError("id", "Customer ID is required");
        }
        if (!customer.email || !this.isValidEmail(customer.email)) {
            throw new ValidationError("email", "Valid email is required");
        }
        if (!customer.name || customer.name.trim() === "") {
            throw new ValidationError("name", "Customer name is required");
        }
        if (customer.creditLimit < 0) {
            throw new ValidationError("creditLimit", "Credit limit cannot be negative");
        }
    }

    private isValidEmail(email: string): boolean {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    private calculateDiscount(product: Product, membershipLevel: MembershipLevel, quantity: number): number {
        let discount = product.discount || 0;

        // Membership discounts
        switch (membershipLevel) {
            case MembershipLevel.SILVER:
                discount += 0.05;
                break;
            case MembershipLevel.GOLD:
                discount += 0.10;
                break;
            case MembershipLevel.PLATINUM:
                discount += 0.15;
                break;
        }

        // Bulk discounts
        if (quantity >= 10) {
            discount += 0.05;
        } else if (quantity >= 5) {
            discount += 0.02;
        }

        return Math.min(discount, 0.5); // Max 50% discount
    }

    private calculateShippingCost(subtotal: number, membershipLevel: MembershipLevel): number {
        // Free shipping for premium members or large orders
        if (membershipLevel === MembershipLevel.PLATINUM || subtotal >= 100) {
            return 0;
        }

        // Reduced shipping for gold members
        if (membershipLevel === MembershipLevel.GOLD) {
            return this.shippingRates.get("standard")! * 0.5;
        }

        return this.shippingRates.get("standard")!;
    }

    private calculateCreditLimit(level: MembershipLevel): number {
        switch (level) {
            case MembershipLevel.BRONZE: return 500;
            case MembershipLevel.SILVER: return 1000;
            case MembershipLevel.GOLD: return 2500;
            case MembershipLevel.PLATINUM: return 5000;
            default: return 500;
        }
    }

    private generateOrderId(): string {
        return `ORD-${Date.now()}-${Math.random().toString(36).substr(2, 5).toUpperCase()}`;
    }

    private async simulatePaymentProcessing(amount: number, paymentMethod: string): Promise<void> {
        // Simulate network delay
        await new Promise(resolve => setTimeout(resolve, 100));

        // Simulate payment failures
        if (amount > 10000) {
            throw new PaymentError("AMOUNT_TOO_HIGH", "Amount exceeds maximum transaction limit");
        }

        if (paymentMethod === "invalid_card") {
            throw new PaymentError("INVALID_CARD", "Invalid payment card");
        }

        if (Math.random() < 0.05) { // 5% random failure rate
            throw new PaymentError("PROCESSING_ERROR", "Payment processing failed");
        }
    }
}