// Large JavaScript file for testing chunk processing
class LargeApplication {
    constructor() {
        this.data = [];
        this.config = {};
        this.utils = new UtilityClass();
    }

    // Function 1
    processData(input) {
        if (!input || typeof input !== 'object') {
            throw new Error('Invalid input data');
        }
        
        const processed = input.map(item => {
            return {
                id: item.id,
                name: item.name.toUpperCase(),
                processed: true,
                timestamp: new Date().toISOString()
            };
        });
        
        return processed;
    }

    // Function 2
    validateData(data) {
        const errors = [];
        
        if (!Array.isArray(data)) {
            errors.push('Data must be an array');
        }
        
        data.forEach((item, index) => {
            if (!item.id) {
                errors.push(`Item ${index} missing id`);
            }
            if (!item.name) {
                errors.push(`Item ${index} missing name`);
            }
            if (typeof item.name !== 'string') {
                errors.push(`Item ${index} name must be string`);
            }
        });
        
        return errors;
    }

    // Function 3
    saveData(data) {
        try {
            const validationErrors = this.validateData(data);
            if (validationErrors.length > 0) {
                throw new Error('Validation failed: ' + validationErrors.join(', '));
            }
            
            const processed = this.processData(data);
            this.data = [...this.data, ...processed];
            return { success: true, count: processed.length };
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    // Function 4
    searchData(query) {
        if (!query || typeof query !== 'string') {
            return [];
        }
        
        return this.data.filter(item => 
            item.name.toLowerCase().includes(query.toLowerCase()) ||
            item.id.toString().includes(query)
        );
    }

    // Function 5
    updateData(id, updates) {
        const index = this.data.findIndex(item => item.id === id);
        if (index === -1) {
            throw new Error(`Item with id ${id} not found`);
        }
        
        this.data[index] = { ...this.data[index], ...updates };
        return this.data[index];
    }

    // Function 6
    deleteData(id) {
        const initialLength = this.data.length;
        this.data = this.data.filter(item => item.id !== id);
        return this.data.length < initialLength;
    }

    // Function 7
    exportData(format) {
        switch (format.toLowerCase()) {
            case 'json':
                return JSON.stringify(this.data, null, 2);
            case 'csv':
                const headers = Object.keys(this.data[0] || {});
                const csvRows = this.data.map(item => 
                    headers.map(header => item[header]).join(',')
                );
                return [headers.join(','), ...csvRows].join('\n');
            default:
                throw new Error('Unsupported format');
        }
    }

    // Function 8
    importData(content, format) {
        try {
            let imported;
            switch (format.toLowerCase()) {
                case 'json':
                    imported = JSON.parse(content);
                    break;
                case 'csv':
                    const lines = content.split('\n');
                    const headers = lines[0].split(',');
                    imported = lines.slice(1).map(line => {
                        const values = line.split(',');
                        return headers.reduce((obj, header, index) => {
                            obj[header] = values[index];
                            return obj;
                        }, {});
                    });
                    break;
                default:
                    throw new Error('Unsupported import format');
            }
            
            return this.saveData(imported);
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    // Function 9
    getStatistics() {
        return {
            totalItems: this.data.length,
            processedItems: this.data.filter(item => item.processed).length,
            latestTimestamp: this.data.reduce((latest, item) => 
                item.timestamp > latest ? item.timestamp : latest, ''
            ),
            averageNameLength: this.data.length > 0 
                ? this.data.reduce((sum, item) => sum + item.name.length, 0) / this.data.length 
                : 0
        };
    }

    // Function 10
    clearData() {
        this.data = [];
        return { success: true, message: 'All data cleared' };
    }
}

// Utility class for additional complexity
class UtilityClass {
    formatDate(date) {
        return new Intl.DateTimeFormat('en-US').format(new Date(date));
    }

    generateId() {
        return Math.random().toString(36).substr(2, 9);
    }

    sanitizeString(str) {
        return str.replace(/[^\w\s]/gi, '').trim();
    }

    calculateHash(data) {
        return btoa(JSON.stringify(data)).slice(0, 8);
    }
}

module.exports = { LargeApplication, UtilityClass };