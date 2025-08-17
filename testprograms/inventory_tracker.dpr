program InventoryTracker;

{$APPTYPE CONSOLE}

{
  Comprehensive Inventory Management System - Delphi Console Application
  Demonstrates Object Pascal features including:
  - Object-oriented programming with classes and inheritance
  - Generic collections and custom data structures
  - Exception handling and custom exceptions
  - Database-like operations with file I/O
  - Advanced string handling and formatting
  - Event-driven architecture with callbacks
  - Memory management and resource handling
}

uses
  System.SysUtils,
  System.Classes,
  System.Generics.Collections,
  System.IOUtils,
  System.JSON,
  System.DateUtils,
  System.StrUtils,
  System.Math;

type
  // Forward declarations
  TInventoryItem = class;
  TInventoryManager = class;
  
  // Custom exception classes
  EInventoryException = class(Exception);
  EItemNotFoundException = class(EInventoryException);
  EInsufficientStockException = class(EInventoryException);
  EDuplicateItemException = class(EInventoryException);
  EInvalidDataException = class(EInventoryException);

  // Enumerated types
  TItemCategory = (icElectronics, icClothing, icBooks, icFood, icToys, icSports, icMedicine, icOther);
  TStockStatus = (ssInStock, ssLowStock, ssOutOfStock, ssDiscontinued);
  TTransactionType = (ttPurchase, ttSale, ttAdjustment, ttReturn, ttTransfer);
  
  // Event types
  TStockLevelChangedEvent = procedure(Sender: TObject; const ItemCode: string; 
    OldQuantity, NewQuantity: Integer) of object;
  TLowStockAlertEvent = procedure(Sender: TObject; Item: TInventoryItem) of object;
  TItemAddedEvent = procedure(Sender: TObject; Item: TInventoryItem) of object;

  // Record types for data structures
  TSupplier = record
    ID: Integer;
    Name: string;
    ContactPerson: string;
    Email: string;
    Phone: string;
    Address: string;
    Rating: Double;
    constructor Create(AID: Integer; const AName, AContact, AEmail, APhone, AAddress: string; ARating: Double);
  end;

  TTransactionRecord = record
    TransactionID: string;
    ItemCode: string;
    TransactionType: TTransactionType;
    Quantity: Integer;
    UnitPrice: Currency;
    TotalAmount: Currency;
    TransactionDate: TDateTime;
    Notes: string;
    UserID: string;
    constructor Create(const ATransactionID, AItemCode: string; AType: TTransactionType;
      AQuantity: Integer; AUnitPrice: Currency; const ANotes, AUserID: string);
  end;

  // Generic collection types
  TSupplierList = TList<TSupplier>;
  TTransactionList = TList<TTransactionRecord>;
  TStringDictionary = TDictionary<string, string>;

  // Main inventory item class
  TInventoryItem = class
  private
    FItemCode: string;
    FName: string;
    FDescription: string;
    FCategory: TItemCategory;
    FQuantity: Integer;
    FMinimumStock: Integer;
    FUnitPrice: Currency;
    FCostPrice: Currency;
    FSupplier: TSupplier;
    FDateAdded: TDateTime;
    FLastModified: TDateTime;
    FIsActive: Boolean;
    FLocation: string;
    FBarcode: string;
    FWeight: Double;
    FDimensions: string;
    FExpirationDate: TDateTime;
    FCustomAttributes: TStringDictionary;
    
    procedure SetQuantity(const Value: Integer);
    procedure SetUnitPrice(const Value: Currency);
    function GetStockStatus: TStockStatus;
    function GetStockValue: Currency;
    function GetProfitMargin: Double;
    
  public
    constructor Create(const AItemCode, AName: string; ACategory: TItemCategory);
    destructor Destroy; override;
    
    // Properties
    property ItemCode: string read FItemCode;
    property Name: string read FName write FName;
    property Description: string read FDescription write FDescription;
    property Category: TItemCategory read FCategory write FCategory;
    property Quantity: Integer read FQuantity write SetQuantity;
    property MinimumStock: Integer read FMinimumStock write FMinimumStock;
    property UnitPrice: Currency read FUnitPrice write SetUnitPrice;
    property CostPrice: Currency read FCostPrice write FCostPrice;
    property Supplier: TSupplier read FSupplier write FSupplier;
    property DateAdded: TDateTime read FDateAdded;
    property LastModified: TDateTime read FLastModified;
    property IsActive: Boolean read FIsActive write FIsActive;
    property Location: string read FLocation write FLocation;
    property Barcode: string read FBarcode write FBarcode;
    property Weight: Double read FWeight write FWeight;
    property Dimensions: string read FDimensions write FDimensions;
    property ExpirationDate: TDateTime read FExpirationDate write FExpirationDate;
    property CustomAttributes: TStringDictionary read FCustomAttributes;
    
    // Calculated properties
    property StockStatus: TStockStatus read GetStockStatus;
    property StockValue: Currency read GetStockValue;
    property ProfitMargin: Double read GetProfitMargin;
    
    // Methods
    procedure UpdateQuantity(Delta: Integer; const Reason: string = '');
    function IsExpired: Boolean;
    function IsExpiringSoon(Days: Integer = 30): Boolean;
    function ToJSON: TJSONObject;
    procedure FromJSON(JSONObj: TJSONObject);
    function ToString: string; override;
    function Clone: TInventoryItem;
    procedure AddCustomAttribute(const Key, Value: string);
    function GetCustomAttribute(const Key: string): string;
    procedure Validate;
  end;

  // Inventory manager class with advanced features
  TInventoryManager = class
  private
    FItems: TObjectDictionary<string, TInventoryItem>;
    FTransactions: TTransactionList;
    FSuppliers: TSupplierList;
    FDataFileName: string;
    FAutoSave: Boolean;
    FNextTransactionID: Integer;
    
    // Events
    FOnStockLevelChanged: TStockLevelChangedEvent;
    FOnLowStockAlert: TLowStockAlertEvent;
    FOnItemAdded: TItemAddedEvent;
    
    procedure DoStockLevelChanged(const ItemCode: string; OldQuantity, NewQuantity: Integer);
    procedure DoLowStockAlert(Item: TInventoryItem);
    procedure DoItemAdded(Item: TInventoryItem);
    function GenerateTransactionID: string;
    procedure ValidateItemCode(const ItemCode: string);
    
  protected
    procedure SaveToFile;
    procedure LoadFromFile;
    
  public
    constructor Create(const ADataFileName: string = 'inventory.json');
    destructor Destroy; override;
    
    // Properties
    property AutoSave: Boolean read FAutoSave write FAutoSave;
    property Items: TObjectDictionary<string, TInventoryItem> read FItems;
    property Transactions: TTransactionList read FTransactions;
    property Suppliers: TSupplierList read FSuppliers;
    
    // Events
    property OnStockLevelChanged: TStockLevelChangedEvent read FOnStockLevelChanged write FOnStockLevelChanged;
    property OnLowStockAlert: TLowStockAlertEvent read FOnLowStockAlert write FOnLowStockAlert;
    property OnItemAdded: TItemAddedEvent read FOnItemAdded write FOnItemAdded;
    
    // Item management methods
    procedure AddItem(Item: TInventoryItem); overload;
    function AddItem(const ItemCode, Name: string; Category: TItemCategory;
      Quantity: Integer; UnitPrice, CostPrice: Currency): TInventoryItem; overload;
    procedure RemoveItem(const ItemCode: string);
    function FindItem(const ItemCode: string): TInventoryItem;
    function ItemExists(const ItemCode: string): Boolean;
    procedure UpdateItem(const ItemCode: string; UpdateProc: TProc<TInventoryItem>);
    
    // Stock management methods
    procedure AdjustStock(const ItemCode: string; Delta: Integer; const Reason: string = '');
    procedure SellItem(const ItemCode: string; Quantity: Integer; const CustomerID: string = '');
    procedure PurchaseItem(const ItemCode: string; Quantity: Integer; UnitCost: Currency; const SupplierRef: string = '');
    procedure TransferStock(const FromItemCode, ToItemCode: string; Quantity: Integer);
    procedure SetMinimumStock(const ItemCode: string; MinStock: Integer);
    
    // Supplier management
    function AddSupplier(const Name, Contact, Email, Phone, Address: string; Rating: Double = 0.0): Integer;
    procedure UpdateSupplier(SupplierID: Integer; const Name, Contact, Email, Phone, Address: string; Rating: Double);
    function FindSupplier(SupplierID: Integer): TSupplier;
    
    // Query methods
    function GetItemsByCategory(Category: TItemCategory): TArray<TInventoryItem>;
    function GetLowStockItems: TArray<TInventoryItem>;
    function GetExpiringItems(Days: Integer = 30): TArray<TInventoryItem>;
    function GetItemsBySupplier(SupplierID: Integer): TArray<TInventoryItem>;
    function SearchItems(const SearchTerm: string; SearchInDescription: Boolean = True): TArray<TInventoryItem>;
    
    // Reporting methods
    function GetTotalInventoryValue: Currency;
    function GetCategoryValue(Category: TItemCategory): Currency;
    function GetInventoryReport: string;
    function GetLowStockReport: string;
    function GetTransactionReport(const StartDate, EndDate: TDateTime): string;
    function GetSupplierReport: string;
    
    // Statistics methods
    function GetTotalItems: Integer;
    function GetTotalQuantity: Integer;
    function GetAverageStockValue: Currency;
    function GetTopSellingItems(Count: Integer = 10): TArray<TInventoryItem>;
    function GetTransactionsByType(TransactionType: TTransactionType): TArray<TTransactionRecord>;
    
    // Utility methods
    procedure ExportToCSV(const FileName: string);
    procedure ImportFromCSV(const FileName: string);
    procedure BackupData(const BackupFileName: string = '');
    procedure RestoreData(const BackupFileName: string);
    procedure ClearAllData;
    procedure ValidateAllItems;
    procedure RecalculateStockLevels;
  end;

// Helper functions
function CategoryToString(Category: TItemCategory): string;
function StringToCategory(const CategoryStr: string): TItemCategory;
function TransactionTypeToString(TransactionType: TTransactionType): string;
function FormatCurrency(Value: Currency): string;
function GenerateItemCode(const Prefix: string = 'ITM'): string;

// Implementation

{ TSupplier }

constructor TSupplier.Create(AID: Integer; const AName, AContact, AEmail, APhone, AAddress: string; ARating: Double);
begin
  ID := AID;
  Name := AName;
  ContactPerson := AContact;
  Email := AEmail;
  Phone := APhone;
  Address := AAddress;
  Rating := ARating;
end;

{ TTransactionRecord }

constructor TTransactionRecord.Create(const ATransactionID, AItemCode: string; AType: TTransactionType;
  AQuantity: Integer; AUnitPrice: Currency; const ANotes, AUserID: string);
begin
  TransactionID := ATransactionID;
  ItemCode := AItemCode;
  TransactionType := AType;
  Quantity := AQuantity;
  UnitPrice := AUnitPrice;
  TotalAmount := AQuantity * AUnitPrice;
  TransactionDate := Now;
  Notes := ANotes;
  UserID := AUserID;
end;

{ TInventoryItem }

constructor TInventoryItem.Create(const AItemCode, AName: string; ACategory: TItemCategory);
begin
  inherited Create;
  
  if Trim(AItemCode) = '' then
    raise EInvalidDataException.Create('Item code cannot be empty');
  if Trim(AName) = '' then
    raise EInvalidDataException.Create('Item name cannot be empty');
    
  FItemCode := UpperCase(Trim(AItemCode));
  FName := Trim(AName);
  FCategory := ACategory;
  FQuantity := 0;
  FMinimumStock := 0;
  FUnitPrice := 0;
  FCostPrice := 0;
  FDateAdded := Now;
  FLastModified := Now;
  FIsActive := True;
  FExpirationDate := 0;
  FCustomAttributes := TStringDictionary.Create;
end;

destructor TInventoryItem.Destroy;
begin
  FCustomAttributes.Free;
  inherited Destroy;
end;

procedure TInventoryItem.SetQuantity(const Value: Integer);
begin
  if Value < 0 then
    raise EInvalidDataException.CreateFmt('Quantity cannot be negative: %d', [Value]);
    
  FQuantity := Value;
  FLastModified := Now;
end;

procedure TInventoryItem.SetUnitPrice(const Value: Currency);
begin
  if Value < 0 then
    raise EInvalidDataException.CreateFmt('Unit price cannot be negative: %m', [Value]);
    
  FUnitPrice := Value;
  FLastModified := Now;
end;

function TInventoryItem.GetStockStatus: TStockStatus;
begin
  if not FIsActive then
    Result := ssDiscontinued
  else if FQuantity = 0 then
    Result := ssOutOfStock
  else if FQuantity <= FMinimumStock then
    Result := ssLowStock
  else
    Result := ssInStock;
end;

function TInventoryItem.GetStockValue: Currency;
begin
  Result := FQuantity * FCostPrice;
end;

function TInventoryItem.GetProfitMargin: Double;
begin
  if FCostPrice = 0 then
    Result := 0
  else
    Result := ((FUnitPrice - FCostPrice) / FCostPrice) * 100;
end;

procedure TInventoryItem.UpdateQuantity(Delta: Integer; const Reason: string);
var
  NewQuantity: Integer;
begin
  NewQuantity := FQuantity + Delta;
  if NewQuantity < 0 then
    raise EInsufficientStockException.CreateFmt('Insufficient stock. Current: %d, Requested: %d', 
      [FQuantity, Abs(Delta)]);
      
  SetQuantity(NewQuantity);
end;

function TInventoryItem.IsExpired: Boolean;
begin
  Result := (FExpirationDate <> 0) and (FExpirationDate < Now);
end;

function TInventoryItem.IsExpiringSoon(Days: Integer): Boolean;
begin
  Result := (FExpirationDate <> 0) and (FExpirationDate < IncDay(Now, Days));
end;

function TInventoryItem.ToJSON: TJSONObject;
var
  CustomAttrsJSON: TJSONObject;
  Key: string;
begin
  Result := TJSONObject.Create;
  try
    Result.AddPair('itemCode', FItemCode);
    Result.AddPair('name', FName);
    Result.AddPair('description', FDescription);
    Result.AddPair('category', Ord(FCategory));
    Result.AddPair('quantity', FQuantity);
    Result.AddPair('minimumStock', FMinimumStock);
    Result.AddPair('unitPrice', CurrToStr(FUnitPrice));
    Result.AddPair('costPrice', CurrToStr(FCostPrice));
    Result.AddPair('dateAdded', DateTimeToStr(FDateAdded));
    Result.AddPair('lastModified', DateTimeToStr(FLastModified));
    Result.AddPair('isActive', FIsActive);
    Result.AddPair('location', FLocation);
    Result.AddPair('barcode', FBarcode);
    Result.AddPair('weight', FWeight);
    Result.AddPair('dimensions', FDimensions);
    if FExpirationDate <> 0 then
      Result.AddPair('expirationDate', DateTimeToStr(FExpirationDate));
      
    // Add custom attributes
    if FCustomAttributes.Count > 0 then
    begin
      CustomAttrsJSON := TJSONObject.Create;
      for Key in FCustomAttributes.Keys do
        CustomAttrsJSON.AddPair(Key, FCustomAttributes[Key]);
      Result.AddPair('customAttributes', CustomAttrsJSON);
    end;
  except
    Result.Free;
    raise;
  end;
end;

procedure TInventoryItem.FromJSON(JSONObj: TJSONObject);
var
  CustomAttrsJSON: TJSONObject;
  Pair: TJSONPair;
begin
  if not Assigned(JSONObj) then
    raise EInvalidDataException.Create('JSON object cannot be nil');
    
  // Load basic properties
  FName := JSONObj.GetValue('name').Value;
  FDescription := JSONObj.GetValue('description').Value;
  FCategory := TItemCategory(StrToIntDef(JSONObj.GetValue('category').Value, 0));
  FQuantity := StrToIntDef(JSONObj.GetValue('quantity').Value, 0);
  FMinimumStock := StrToIntDef(JSONObj.GetValue('minimumStock').Value, 0);
  FUnitPrice := StrToCurrDef(JSONObj.GetValue('unitPrice').Value, 0);
  FCostPrice := StrToCurrDef(JSONObj.GetValue('costPrice').Value, 0);
  FDateAdded := StrToDateTimeDef(JSONObj.GetValue('dateAdded').Value, Now);
  FLastModified := StrToDateTimeDef(JSONObj.GetValue('lastModified').Value, Now);
  FIsActive := StrToBoolDef(JSONObj.GetValue('isActive').Value, True);
  FLocation := JSONObj.GetValue('location').Value;
  FBarcode := JSONObj.GetValue('barcode').Value;
  FWeight := StrToFloatDef(JSONObj.GetValue('weight').Value, 0);
  FDimensions := JSONObj.GetValue('dimensions').Value;
  
  if JSONObj.GetValue('expirationDate') <> nil then
    FExpirationDate := StrToDateTimeDef(JSONObj.GetValue('expirationDate').Value, 0);
    
  // Load custom attributes
  CustomAttrsJSON := JSONObj.GetValue('customAttributes') as TJSONObject;
  if Assigned(CustomAttrsJSON) then
  begin
    FCustomAttributes.Clear;
    for Pair in CustomAttrsJSON do
      FCustomAttributes.Add(Pair.JsonString.Value, Pair.JsonValue.Value);
  end;
end;

function TInventoryItem.ToString: string;
begin
  Result := Format('[%s] %s - Qty: %d, Price: %s, Status: %s', 
    [FItemCode, FName, FQuantity, FormatCurrency(FUnitPrice), 
     Copy(GetEnumName(TypeInfo(TStockStatus), Ord(StockStatus)), 3, MaxInt)]);
end;

function TInventoryItem.Clone: TInventoryItem;
begin
  Result := TInventoryItem.Create(FItemCode + '_COPY', FName, FCategory);
  Result.FDescription := FDescription;
  Result.FQuantity := FQuantity;
  Result.FMinimumStock := FMinimumStock;
  Result.FUnitPrice := FUnitPrice;
  Result.FCostPrice := FCostPrice;
  Result.FSupplier := FSupplier;
  Result.FIsActive := FIsActive;
  Result.FLocation := FLocation;
  Result.FBarcode := FBarcode;
  Result.FWeight := FWeight;
  Result.FDimensions := FDimensions;
  Result.FExpirationDate := FExpirationDate;
  
  // Copy custom attributes
  for var Key in FCustomAttributes.Keys do
    Result.FCustomAttributes.Add(Key, FCustomAttributes[Key]);
end;

procedure TInventoryItem.AddCustomAttribute(const Key, Value: string);
begin
  if Trim(Key) = '' then
    raise EInvalidDataException.Create('Attribute key cannot be empty');
    
  FCustomAttributes.AddOrSetValue(Key, Value);
  FLastModified := Now;
end;

function TInventoryItem.GetCustomAttribute(const Key: string): string;
begin
  if not FCustomAttributes.TryGetValue(Key, Result) then
    Result := '';
end;

procedure TInventoryItem.Validate;
begin
  if Trim(FItemCode) = '' then
    raise EInvalidDataException.Create('Item code is required');
  if Trim(FName) = '' then
    raise EInvalidDataException.Create('Item name is required');
  if FQuantity < 0 then
    raise EInvalidDataException.Create('Quantity cannot be negative');
  if FUnitPrice < 0 then
    raise EInvalidDataException.Create('Unit price cannot be negative');
  if FCostPrice < 0 then
    raise EInvalidDataException.Create('Cost price cannot be negative');
  if FMinimumStock < 0 then
    raise EInvalidDataException.Create('Minimum stock cannot be negative');
end;

{ TInventoryManager }

constructor TInventoryManager.Create(const ADataFileName: string);
begin
  inherited Create;
  
  FItems := TObjectDictionary<string, TInventoryItem>.Create([doOwnsValues]);
  FTransactions := TTransactionList.Create;
  FSuppliers := TSupplierList.Create;
  FDataFileName := ADataFileName;
  FAutoSave := True;
  FNextTransactionID := 1;
  
  // Load existing data if file exists
  if TFile.Exists(FDataFileName) then
  begin
    try
      LoadFromFile;
    except
      on E: Exception do
        WriteLn('Warning: Could not load data file: ', E.Message);
    end;
  end;
  
  // Add some default suppliers for testing
  AddSupplier('Global Electronics Inc.', 'John Smith', 'john@globalelectronics.com', '+1-555-0101', '123 Tech Street, Silicon Valley, CA', 4.5);
  AddSupplier('Fashion Forward Ltd.', 'Sarah Johnson', 'sarah@fashionforward.com', '+1-555-0102', '456 Style Ave, New York, NY', 4.2);
  AddSupplier('BookWorld Distributors', 'Mike Wilson', 'mike@bookworld.com', '+1-555-0103', '789 Literature Blvd, Chicago, IL', 4.8);
end;

destructor TInventoryManager.Destroy;
begin
  if FAutoSave then
    SaveToFile;
    
  FItems.Free;
  FTransactions.Free;
  FSuppliers.Free;
  inherited Destroy;
end;

procedure TInventoryManager.DoStockLevelChanged(const ItemCode: string; OldQuantity, NewQuantity: Integer);
begin
  if Assigned(FOnStockLevelChanged) then
    FOnStockLevelChanged(Self, ItemCode, OldQuantity, NewQuantity);
end;

procedure TInventoryManager.DoLowStockAlert(Item: TInventoryItem);
begin
  if Assigned(FOnLowStockAlert) then
    FOnLowStockAlert(Self, Item);
end;

procedure TInventoryManager.DoItemAdded(Item: TInventoryItem);
begin
  if Assigned(FOnItemAdded) then
    FOnItemAdded(Self, Item);
end;

function TInventoryManager.GenerateTransactionID: string;
begin
  Result := Format('TXN-%s-%06d', [FormatDateTime('YYYYMMDD', Now), FNextTransactionID]);
  Inc(FNextTransactionID);
end;

procedure TInventoryManager.ValidateItemCode(const ItemCode: string);
begin
  if Trim(ItemCode) = '' then
    raise EInvalidDataException.Create('Item code cannot be empty');
end;

procedure TInventoryManager.SaveToFile;
var
  JSONRoot, ItemsArray, TransactionsArray: TJSONObject;
  Item: TInventoryItem;
  Transaction: TTransactionRecord;
begin
  JSONRoot := TJSONObject.Create;
  try
    // Save metadata
    JSONRoot.AddPair('version', '1.0');
    JSONRoot.AddPair('lastSaved', DateTimeToStr(Now));
    JSONRoot.AddPair('nextTransactionID', FNextTransactionID);
    
    // Save items
    ItemsArray := TJSONObject.Create;
    for Item in FItems.Values do
      ItemsArray.AddPair(Item.ItemCode, Item.ToJSON);
    JSONRoot.AddPair('items', ItemsArray);
    
    // Save transactions would go here (simplified for this example)
    
    TFile.WriteAllText(FDataFileName, JSONRoot.ToString, TEncoding.UTF8);
  finally
    JSONRoot.Free;
  end;
end;

procedure TInventoryManager.LoadFromFile;
var
  JSONText: string;
  JSONRoot, ItemsObj: TJSONObject;
  ItemPair: TJSONPair;
  Item: TInventoryItem;
  ItemJSON: TJSONObject;
begin
  if not TFile.Exists(FDataFileName) then
    Exit;
    
  try
    JSONText := TFile.ReadAllText(FDataFileName, TEncoding.UTF8);
    JSONRoot := TJSONObject.ParseJSONValue(JSONText) as TJSONObject;
    try
      if not Assigned(JSONRoot) then
        Exit;
        
      // Load metadata
      if JSONRoot.GetValue('nextTransactionID') <> nil then
        FNextTransactionID := StrToIntDef(JSONRoot.GetValue('nextTransactionID').Value, 1);
        
      // Load items
      ItemsObj := JSONRoot.GetValue('items') as TJSONObject;
      if Assigned(ItemsObj) then
      begin
        FItems.Clear;
        for ItemPair in ItemsObj do
        begin
          ItemJSON := ItemPair.JsonValue as TJSONObject;
          if Assigned(ItemJSON) then
          begin
            Item := TInventoryItem.Create(
              ItemPair.JsonString.Value,
              ItemJSON.GetValue('name').Value,
              TItemCategory(StrToIntDef(ItemJSON.GetValue('category').Value, 0))
            );
            try
              Item.FromJSON(ItemJSON);
              FItems.Add(Item.ItemCode, Item);
            except
              Item.Free;
              raise;
            end;
          end;
        end;
      end;
    finally
      JSONRoot.Free;
    end;
  except
    on E: Exception do
      WriteLn('Error loading inventory data: ', E.Message);
  end;
end;

procedure TInventoryManager.AddItem(Item: TInventoryItem);
begin
  if not Assigned(Item) then
    raise EInvalidDataException.Create('Item cannot be nil');
    
  Item.Validate;
  
  if FItems.ContainsKey(Item.ItemCode) then
    raise EDuplicateItemException.CreateFmt('Item with code "%s" already exists', [Item.ItemCode]);
    
  FItems.Add(Item.ItemCode, Item);
  DoItemAdded(Item);
  
  if FAutoSave then
    SaveToFile;
end;

function TInventoryManager.AddItem(const ItemCode, Name: string; Category: TItemCategory;
  Quantity: Integer; UnitPrice, CostPrice: Currency): TInventoryItem;
begin
  Result := TInventoryItem.Create(ItemCode, Name, Category);
  try
    Result.Quantity := Quantity;
    Result.UnitPrice := UnitPrice;
    Result.CostPrice := CostPrice;
    AddItem(Result);
  except
    Result.Free;
    raise;
  end;
end;

procedure TInventoryManager.RemoveItem(const ItemCode: string);
begin
  ValidateItemCode(ItemCode);
  
  if not FItems.ContainsKey(UpperCase(ItemCode)) then
    raise EItemNotFoundException.CreateFmt('Item "%s" not found', [ItemCode]);
    
  FItems.Remove(UpperCase(ItemCode));
  
  if FAutoSave then
    SaveToFile;
end;

function TInventoryManager.FindItem(const ItemCode: string): TInventoryItem;
begin
  ValidateItemCode(ItemCode);
  
  if not FItems.TryGetValue(UpperCase(ItemCode), Result) then
    raise EItemNotFoundException.CreateFmt('Item "%s" not found', [ItemCode]);
end;

function TInventoryManager.ItemExists(const ItemCode: string): Boolean;
begin
  Result := FItems.ContainsKey(UpperCase(ItemCode));
end;

procedure TInventoryManager.UpdateItem(const ItemCode: string; UpdateProc: TProc<TInventoryItem>);
var
  Item: TInventoryItem;
begin
  Item := FindItem(ItemCode);
  UpdateProc(Item);
  
  if FAutoSave then
    SaveToFile;
end;

procedure TInventoryManager.AdjustStock(const ItemCode: string; Delta: Integer; const Reason: string);
var
  Item: TInventoryItem;
  OldQuantity: Integer;
  TransactionRec: TTransactionRecord;
begin
  Item := FindItem(ItemCode);
  OldQuantity := Item.Quantity;
  
  Item.UpdateQuantity(Delta, Reason);
  
  // Record transaction
  TransactionRec := TTransactionRecord.Create(
    GenerateTransactionID,
    ItemCode,
    ttAdjustment,
    Delta,
    0,
    Reason,
    'SYSTEM'
  );
  FTransactions.Add(TransactionRec);
  
  DoStockLevelChanged(ItemCode, OldQuantity, Item.Quantity);
  
  // Check for low stock alert
  if Item.StockStatus = ssLowStock then
    DoLowStockAlert(Item);
    
  if FAutoSave then
    SaveToFile;
end;

procedure TInventoryManager.SellItem(const ItemCode: string; Quantity: Integer; const CustomerID: string);
var
  Item: TInventoryItem;
  TransactionRec: TTransactionRecord;
begin
  if Quantity <= 0 then
    raise EInvalidDataException.Create('Sale quantity must be positive');
    
  Item := FindItem(ItemCode);
  
  if Item.Quantity < Quantity then
    raise EInsufficientStockException.CreateFmt('Insufficient stock for item "%s". Available: %d, Requested: %d',
      [ItemCode, Item.Quantity, Quantity]);
      
  AdjustStock(ItemCode, -Quantity, Format('Sale to customer: %s', [CustomerID]));
  
  // Record sale transaction
  TransactionRec := TTransactionRecord.Create(
    GenerateTransactionID,
    ItemCode,
    ttSale,
    Quantity,
    Item.UnitPrice,
    Format('Sale to customer: %s', [CustomerID]),
    'SYSTEM'
  );
  FTransactions.Add(TransactionRec);
end;

procedure TInventoryManager.PurchaseItem(const ItemCode: string; Quantity: Integer; UnitCost: Currency; const SupplierRef: string);
var
  TransactionRec: TTransactionRecord;
begin
  if Quantity <= 0 then
    raise EInvalidDataException.Create('Purchase quantity must be positive');
  if UnitCost < 0 then
    raise EInvalidDataException.Create('Unit cost cannot be negative');
    
  AdjustStock(ItemCode, Quantity, Format('Purchase from supplier: %s', [SupplierRef]));
  
  // Update cost price if provided
  if UnitCost > 0 then
    UpdateItem(ItemCode, procedure(Item: TInventoryItem)
      begin
        Item.CostPrice := UnitCost;
      end);
  
  // Record purchase transaction
  TransactionRec := TTransactionRecord.Create(
    GenerateTransactionID,
    ItemCode,
    ttPurchase,
    Quantity,
    UnitCost,
    Format('Purchase from supplier: %s', [SupplierRef]),
    'SYSTEM'
  );
  FTransactions.Add(TransactionRec);
end;

procedure TInventoryManager.TransferStock(const FromItemCode, ToItemCode: string; Quantity: Integer);
var
  FromItem, ToItem: TInventoryItem;
begin
  if Quantity <= 0 then
    raise EInvalidDataException.Create('Transfer quantity must be positive');
    
  FromItem := FindItem(FromItemCode);
  ToItem := FindItem(ToItemCode);
  
  if FromItem.Quantity < Quantity then
    raise EInsufficientStockException.CreateFmt('Insufficient stock in source item "%s"', [FromItemCode]);
    
  AdjustStock(FromItemCode, -Quantity, Format('Transfer to %s', [ToItemCode]));
  AdjustStock(ToItemCode, Quantity, Format('Transfer from %s', [FromItemCode]));
end;

procedure TInventoryManager.SetMinimumStock(const ItemCode: string; MinStock: Integer);
begin
  if MinStock < 0 then
    raise EInvalidDataException.Create('Minimum stock cannot be negative');
    
  UpdateItem(ItemCode, procedure(Item: TInventoryItem)
    begin
      Item.MinimumStock := MinStock;
    end);
end;

function TInventoryManager.AddSupplier(const Name, Contact, Email, Phone, Address: string; Rating: Double): Integer;
var
  Supplier: TSupplier;
begin
  Result := FSuppliers.Count + 1;
  Supplier := TSupplier.Create(Result, Name, Contact, Email, Phone, Address, Rating);
  FSuppliers.Add(Supplier);
end;

procedure TInventoryManager.UpdateSupplier(SupplierID: Integer; const Name, Contact, Email, Phone, Address: string; Rating: Double);
var
  I: Integer;
  Supplier: TSupplier;
begin
  for I := 0 to FSuppliers.Count - 1 do
    if FSuppliers[I].ID = SupplierID then
    begin
      Supplier := TSupplier.Create(SupplierID, Name, Contact, Email, Phone, Address, Rating);
      FSuppliers[I] := Supplier;
      Exit;
    end;
  raise EItemNotFoundException.CreateFmt('Supplier with ID %d not found', [SupplierID]);
end;

function TInventoryManager.FindSupplier(SupplierID: Integer): TSupplier;
var
  I: Integer;
begin
  for I := 0 to FSuppliers.Count - 1 do
    if FSuppliers[I].ID = SupplierID then
    begin
      Result := FSuppliers[I];
      Exit;
    end;
  raise EItemNotFoundException.CreateFmt('Supplier with ID %d not found', [SupplierID]);
end;

function TInventoryManager.GetItemsByCategory(Category: TItemCategory): TArray<TInventoryItem>;
var
  Item: TInventoryItem;
  Items: TList<TInventoryItem>;
begin
  Items := TList<TInventoryItem>.Create;
  try
    for Item in FItems.Values do
      if Item.Category = Category then
        Items.Add(Item);
    Result := Items.ToArray;
  finally
    Items.Free;
  end;
end;

function TInventoryManager.GetLowStockItems: TArray<TInventoryItem>;
var
  Item: TInventoryItem;
  Items: TList<TInventoryItem>;
begin
  Items := TList<TInventoryItem>.Create;
  try
    for Item in FItems.Values do
      if Item.StockStatus = ssLowStock then
        Items.Add(Item);
    Result := Items.ToArray;
  finally
    Items.Free;
  end;
end;

function TInventoryManager.GetExpiringItems(Days: Integer): TArray<TInventoryItem>;
var
  Item: TInventoryItem;
  Items: TList<TInventoryItem>;
begin
  Items := TList<TInventoryItem>.Create;
  try
    for Item in FItems.Values do
      if Item.IsExpiringSoon(Days) then
        Items.Add(Item);
    Result := Items.ToArray;
  finally
    Items.Free;
  end;
end;

function TInventoryManager.GetItemsBySupplier(SupplierID: Integer): TArray<TInventoryItem>;
var
  Item: TInventoryItem;
  Items: TList<TInventoryItem>;
begin
  Items := TList<TInventoryItem>.Create;
  try
    for Item in FItems.Values do
      if Item.Supplier.ID = SupplierID then
        Items.Add(Item);
    Result := Items.ToArray;
  finally
    Items.Free;
  end;
end;

function TInventoryManager.SearchItems(const SearchTerm: string; SearchInDescription: Boolean): TArray<TInventoryItem>;
var
  Item: TInventoryItem;
  Items: TList<TInventoryItem>;
  UpperSearchTerm: string;
begin
  Items := TList<TInventoryItem>.Create;
  try
    UpperSearchTerm := UpperCase(SearchTerm);
    for Item in FItems.Values do
    begin
      if (Pos(UpperSearchTerm, UpperCase(Item.Name)) > 0) or
         (Pos(UpperSearchTerm, UpperCase(Item.ItemCode)) > 0) or
         (SearchInDescription and (Pos(UpperSearchTerm, UpperCase(Item.Description)) > 0)) then
        Items.Add(Item);
    end;
    Result := Items.ToArray;
  finally
    Items.Free;
  end;
end;

function TInventoryManager.GetTotalInventoryValue: Currency;
var
  Item: TInventoryItem;
begin
  Result := 0;
  for Item in FItems.Values do
    Result := Result + Item.StockValue;
end;

function TInventoryManager.GetCategoryValue(Category: TItemCategory): Currency;
var
  Items: TArray<TInventoryItem>;
  Item: TInventoryItem;
begin
  Result := 0;
  Items := GetItemsByCategory(Category);
  for Item in Items do
    Result := Result + Item.StockValue;
end;

function TInventoryManager.GetInventoryReport: string;
var
  Report: TStringBuilder;
  Item: TInventoryItem;
  Category: TItemCategory;
begin
  Report := TStringBuilder.Create;
  try
    Report.AppendLine('=== COMPREHENSIVE INVENTORY REPORT ===');
    Report.AppendLine(Format('Generated on: %s', [DateTimeToStr(Now)]));
    Report.AppendLine(Format('Total Items: %d', [FItems.Count]));
    Report.AppendLine(Format('Total Inventory Value: %s', [FormatCurrency(GetTotalInventoryValue)]));
    Report.AppendLine('');
    
    // Category breakdown
    Report.AppendLine('INVENTORY BY CATEGORY:');
    for Category := Low(TItemCategory) to High(TItemCategory) do
    begin
      Report.AppendLine(Format('  %s: %s', 
        [CategoryToString(Category), FormatCurrency(GetCategoryValue(Category))]));
    end;
    Report.AppendLine('');
    
    // Item details
    Report.AppendLine('ITEM DETAILS:');
    for Item in FItems.Values do
    begin
      Report.AppendLine(Format('Code: %s | Name: %s | Qty: %d | Value: %s | Status: %s',
        [Item.ItemCode, Item.Name, Item.Quantity, FormatCurrency(Item.StockValue),
         Copy(GetEnumName(TypeInfo(TStockStatus), Ord(Item.StockStatus)), 3, MaxInt)]));
    end;
    
    Result := Report.ToString;
  finally
    Report.Free;
  end;
end;

function TInventoryManager.GetLowStockReport: string;
var
  Report: TStringBuilder;
  Items: TArray<TInventoryItem>;
  Item: TInventoryItem;
begin
  Report := TStringBuilder.Create;
  try
    Items := GetLowStockItems;
    
    Report.AppendLine('=== LOW STOCK ALERT REPORT ===');
    Report.AppendLine(Format('Generated on: %s', [DateTimeToStr(Now)]));
    Report.AppendLine(Format('Items requiring attention: %d', [Length(Items)]));
    Report.AppendLine('');
    
    if Length(Items) = 0 then
    begin
      Report.AppendLine('No items are currently low on stock.');
    end
    else
    begin
      Report.AppendLine('ITEMS REQUIRING RESTOCKING:');
      for Item in Items do
      begin
        Report.AppendLine(Format('%s (%s) - Current: %d, Minimum: %d, Shortage: %d',
          [Item.ItemCode, Item.Name, Item.Quantity, Item.MinimumStock, 
           Max(0, Item.MinimumStock - Item.Quantity)]));
      end;
    end;
    
    Result := Report.ToString;
  finally
    Report.Free;
  end;
end;

function TInventoryManager.GetTransactionReport(const StartDate, EndDate: TDateTime): string;
var
  Report: TStringBuilder;
  Transaction: TTransactionRecord;
  TransactionCount: Integer;
  TotalValue: Currency;
begin
  Report := TStringBuilder.Create;
  try
    TransactionCount := 0;
    TotalValue := 0;
    
    Report.AppendLine('=== TRANSACTION REPORT ===');
    Report.AppendLine(Format('Period: %s to %s', [DateToStr(StartDate), DateToStr(EndDate)]));
    Report.AppendLine('');
    
    for Transaction in FTransactions do
    begin
      if (Transaction.TransactionDate >= StartDate) and (Transaction.TransactionDate <= EndDate) then
      begin
        Inc(TransactionCount);
        TotalValue := TotalValue + Transaction.TotalAmount;
        
        Report.AppendLine(Format('%s | %s | %s | Qty: %d | Amount: %s | %s',
          [Transaction.TransactionID, 
           DateTimeToStr(Transaction.TransactionDate),
           TransactionTypeToString(Transaction.TransactionType),
           Transaction.Quantity,
           FormatCurrency(Transaction.TotalAmount),
           Transaction.Notes]));
      end;
    end;
    
    Report.AppendLine('');
    Report.AppendLine(Format('Total Transactions: %d', [TransactionCount]));
    Report.AppendLine(Format('Total Value: %s', [FormatCurrency(TotalValue)]));
    
    Result := Report.ToString;
  finally
    Report.Free;
  end;
end;

function TInventoryManager.GetSupplierReport: string;
var
  Report: TStringBuilder;
  Supplier: TSupplier;
  SupplierItems: TArray<TInventoryItem>;
begin
  Report := TStringBuilder.Create;
  try
    Report.AppendLine('=== SUPPLIER REPORT ===');
    Report.AppendLine(Format('Generated on: %s', [DateTimeToStr(Now)]));
    Report.AppendLine(Format('Total Suppliers: %d', [FSuppliers.Count]));
    Report.AppendLine('');
    
    for Supplier in FSuppliers do
    begin
      SupplierItems := GetItemsBySupplier(Supplier.ID);
      
      Report.AppendLine(Format('Supplier ID: %d', [Supplier.ID]));
      Report.AppendLine(Format('Name: %s', [Supplier.Name]));
      Report.AppendLine(Format('Contact: %s (%s)', [Supplier.ContactPerson, Supplier.Email]));
      Report.AppendLine(Format('Phone: %s', [Supplier.Phone]));
      Report.AppendLine(Format('Rating: %.1f/5.0', [Supplier.Rating]));
      Report.AppendLine(Format('Items Supplied: %d', [Length(SupplierItems)]));
      Report.AppendLine('------------------------');
    end;
    
    Result := Report.ToString;
  finally
    Report.Free;
  end;
end;

function TInventoryManager.GetTotalItems: Integer;
begin
  Result := FItems.Count;
end;

function TInventoryManager.GetTotalQuantity: Integer;
var
  Item: TInventoryItem;
begin
  Result := 0;
  for Item in FItems.Values do
    Result := Result + Item.Quantity;
end;

function TInventoryManager.GetAverageStockValue: Currency;
begin
  if FItems.Count = 0 then
    Result := 0
  else
    Result := GetTotalInventoryValue / FItems.Count;
end;

function TInventoryManager.GetTopSellingItems(Count: Integer): TArray<TInventoryItem>;
var
  SalesMap: TDictionary<string, Integer>;
  Transaction: TTransactionRecord;
  ItemCode: string;
  SalesCount: Integer;
  SortedItems: TList<TPair<string, Integer>>;
  Item: TInventoryItem;
  ItemList: TList<TInventoryItem>;
  I: Integer;
begin
  SalesMap := TDictionary<string, Integer>.Create;
  SortedItems := TList<TPair<string, Integer>>.Create;
  ItemList := TList<TInventoryItem>.Create;
  try
    // Count sales transactions for each item
    for Transaction in FTransactions do
    begin
      if Transaction.TransactionType = ttSale then
      begin
        ItemCode := Transaction.ItemCode;
        if SalesMap.TryGetValue(ItemCode, SalesCount) then
          SalesMap[ItemCode] := SalesCount + Transaction.Quantity
        else
          SalesMap.Add(ItemCode, Transaction.Quantity);
      end;
    end;
    
    // Convert to sortable list
    for ItemCode in SalesMap.Keys do
      SortedItems.Add(TPair<string, Integer>.Create(ItemCode, SalesMap[ItemCode]));
    
    // Sort by sales count (descending)
    SortedItems.Sort(TComparer<TPair<string, Integer>>.Construct(
      function(const Left, Right: TPair<string, Integer>): Integer
      begin
        Result := Right.Value - Left.Value; // Descending order
      end));
    
    // Build result array with actual items
    for I := 0 to Min(Count - 1, SortedItems.Count - 1) do
    begin
      if FItems.TryGetValue(SortedItems[I].Key, Item) then
        ItemList.Add(Item);
    end;
    
    Result := ItemList.ToArray;
  finally
    SalesMap.Free;
    SortedItems.Free;
    ItemList.Free;
  end;
end;

function TInventoryManager.GetTransactionsByType(TransactionType: TTransactionType): TArray<TTransactionRecord>;
var
  Transaction: TTransactionRecord;
  Transactions: TList<TTransactionRecord>;
begin
  Transactions := TList<TTransactionRecord>.Create;
  try
    for Transaction in FTransactions do
      if Transaction.TransactionType = TransactionType then
        Transactions.Add(Transaction);
    Result := Transactions.ToArray;
  finally
    Transactions.Free;
  end;
end;

procedure TInventoryManager.ExportToCSV(const FileName: string);
var
  CSVLines: TStringList;
  Item: TInventoryItem;
begin
  CSVLines := TStringList.Create;
  try
    // Header
    CSVLines.Add('ItemCode,Name,Description,Category,Quantity,MinimumStock,UnitPrice,CostPrice,StockValue,Location,Barcode');
    
    // Data rows
    for Item in FItems.Values do
    begin
      CSVLines.Add(Format('"%s","%s","%s","%s",%d,%d,%.2f,%.2f,%.2f,"%s","%s"',
        [Item.ItemCode, Item.Name, Item.Description, CategoryToString(Item.Category),
         Item.Quantity, Item.MinimumStock, Item.UnitPrice, Item.CostPrice,
         Item.StockValue, Item.Location, Item.Barcode]));
    end;
    
    CSVLines.SaveToFile(FileName, TEncoding.UTF8);
  finally
    CSVLines.Free;
  end;
end;

procedure TInventoryManager.ImportFromCSV(const FileName: string);
var
  CSVLines: TStringList;
  LineData: TStringList;
  I: Integer;
  Line: string;
  Item: TInventoryItem;
  ItemCode, Name, Description, CategoryStr, Location, Barcode: string;
  Category: TItemCategory;
  Quantity, MinimumStock: Integer;
  UnitPrice, CostPrice: Currency;
begin
  if not TFile.Exists(FileName) then
    raise EInvalidDataException.CreateFmt('CSV file not found: %s', [FileName]);
    
  CSVLines := TStringList.Create;
  LineData := TStringList.Create;
  try
    CSVLines.LoadFromFile(FileName, TEncoding.UTF8);
    
    // Skip header line if present
    for I := 1 to CSVLines.Count - 1 do
    begin
      Line := CSVLines[I];
      if Trim(Line) = '' then
        Continue;
        
      // Simple CSV parsing (assumes quoted fields)
      LineData.Clear;
      LineData.Delimiter := ',';
      LineData.QuoteChar := '"';
      LineData.DelimitedText := Line;
      
      if LineData.Count >= 11 then
      begin
        try
          ItemCode := Trim(LineData[0]);
          Name := Trim(LineData[1]);
          Description := Trim(LineData[2]);
          CategoryStr := Trim(LineData[3]);
          Quantity := StrToIntDef(LineData[4], 0);
          MinimumStock := StrToIntDef(LineData[5], 0);
          UnitPrice := StrToCurrDef(LineData[6], 0);
          CostPrice := StrToCurrDef(LineData[7], 0);
          Location := Trim(LineData[9]);
          Barcode := Trim(LineData[10]);
          
          // Convert category string to enum
          Category := StringToCategory(CategoryStr);
          
          // Create and add item if it doesn't exist
          if not ItemExists(ItemCode) then
          begin
            Item := TInventoryItem.Create(ItemCode, Name, Category);
            try
              Item.Description := Description;
              Item.Quantity := Quantity;
              Item.MinimumStock := MinimumStock;
              Item.UnitPrice := UnitPrice;
              Item.CostPrice := CostPrice;
              Item.Location := Location;
              Item.Barcode := Barcode;
              
              AddItem(Item);
            except
              Item.Free;
              raise;
            end;
          end;
        except
          on E: Exception do
            WriteLn(Format('Error importing line %d: %s', [I + 1, E.Message]));
        end;
      end;
    end;
  finally
    CSVLines.Free;
    LineData.Free;
  end;
end;

procedure TInventoryManager.BackupData(const BackupFileName: string);
var
  BackupName: string;
begin
  if BackupFileName = '' then
    BackupName := ChangeFileExt(FDataFileName, Format('.backup.%s.json', [FormatDateTime('YYYYMMDD_HHNNSS', Now)]))
  else
    BackupName := BackupFileName;
    
  TFile.Copy(FDataFileName, BackupName, True);
end;

procedure TInventoryManager.RestoreData(const BackupFileName: string);
begin
  if not TFile.Exists(BackupFileName) then
    raise EInvalidDataException.CreateFmt('Backup file not found: %s', [BackupFileName]);
    
  TFile.Copy(BackupFileName, FDataFileName, True);
  LoadFromFile;
end;

procedure TInventoryManager.ClearAllData;
begin
  FItems.Clear;
  FTransactions.Clear;
  
  if FAutoSave then
    SaveToFile;
end;

procedure TInventoryManager.ValidateAllItems;
var
  Item: TInventoryItem;
  ErrorCount: Integer;
begin
  ErrorCount := 0;
  
  for Item in FItems.Values do
  begin
    try
      Item.Validate;
    except
      on E: Exception do
      begin
        Inc(ErrorCount);
        WriteLn(Format('Validation error for item %s: %s', [Item.ItemCode, E.Message]));
      end;
    end;
  end;
  
  WriteLn(Format('Validation completed. %d errors found.', [ErrorCount]));
end;

procedure TInventoryManager.RecalculateStockLevels;
var
  StockLevels: TDictionary<string, Integer>;
  Transaction: TTransactionRecord;
  Item: TInventoryItem;
  ItemCode: string;
  CalculatedQuantity, ActualQuantity: Integer;
  DiscrepancyCount: Integer;
begin
  StockLevels := TDictionary<string, Integer>.Create;
  try
    WriteLn('Recalculating stock levels based on transaction history...');
    DiscrepancyCount := 0;
    
    // Initialize all items with zero quantity
    for Item in FItems.Values do
      StockLevels.Add(Item.ItemCode, 0);
    
    // Process all transactions chronologically
    for Transaction in FTransactions do
    begin
      ItemCode := Transaction.ItemCode;
      if StockLevels.ContainsKey(ItemCode) then
      begin
        case Transaction.TransactionType of
          ttPurchase, ttAdjustment:
            StockLevels[ItemCode] := StockLevels[ItemCode] + Transaction.Quantity;
          ttSale:
            StockLevels[ItemCode] := StockLevels[ItemCode] - Transaction.Quantity;
          ttReturn:
            StockLevels[ItemCode] := StockLevels[ItemCode] + Transaction.Quantity;
          ttTransfer:
            // Transfer transactions are handled as separate adjustment transactions
            StockLevels[ItemCode] := StockLevels[ItemCode] + Transaction.Quantity;
        end;
        
        // Ensure stock doesn't go below zero
        if StockLevels[ItemCode] < 0 then
          StockLevels[ItemCode] := 0;
      end;
    end;
    
    // Compare calculated quantities with actual quantities and update if needed
    for Item in FItems.Values do
    begin
      if StockLevels.TryGetValue(Item.ItemCode, CalculatedQuantity) then
      begin
        ActualQuantity := Item.Quantity;
        if CalculatedQuantity <> ActualQuantity then
        begin
          Inc(DiscrepancyCount);
          WriteLn(Format('Discrepancy found for %s (%s): Actual=%d, Calculated=%d - Correcting...',
            [Item.ItemCode, Item.Name, ActualQuantity, CalculatedQuantity]));
          
          // Update the item quantity (bypassing validation to allow direct correction)
          Item.FQuantity := CalculatedQuantity;
          Item.FLastModified := Now;
        end;
      end;
    end;
    
    WriteLn(Format('Stock level recalculation completed. %d discrepancies corrected.', [DiscrepancyCount]));
    
    if FAutoSave then
      SaveToFile;
      
  finally
    StockLevels.Free;
  end;
end;

// Helper functions

function CategoryToString(Category: TItemCategory): string;
const
  CategoryNames: array[TItemCategory] of string = (
    'Electronics', 'Clothing', 'Books', 'Food', 'Toys', 'Sports', 'Medicine', 'Other');
begin
  Result := CategoryNames[Category];
end;

function StringToCategory(const CategoryStr: string): TItemCategory;
var
  Category: TItemCategory;
begin
  for Category := Low(TItemCategory) to High(TItemCategory) do
    if SameText(CategoryToString(Category), CategoryStr) then
    begin
      Result := Category;
      Exit;
    end;
  Result := icOther;
end;

function TransactionTypeToString(TransactionType: TTransactionType): string;
const
  TransactionNames: array[TTransactionType] of string = (
    'Purchase', 'Sale', 'Adjustment', 'Return', 'Transfer');
begin
  Result := TransactionNames[TransactionType];
end;

function FormatCurrency(Value: Currency): string;
begin
  Result := Format('$%.2f', [Value]);
end;

function GenerateItemCode(const Prefix: string): string;
begin
  Result := Format('%s%s%04d', [Prefix, FormatDateTime('YYMMDD', Now), Random(10000)]);
end;

// Event handlers for demonstration
procedure OnStockLevelChanged(Sender: TObject; const ItemCode: string; OldQuantity, NewQuantity: Integer);
begin
  WriteLn(Format('Stock level changed for %s: %d -> %d', [ItemCode, OldQuantity, NewQuantity]));
end;

procedure OnLowStockAlert(Sender: TObject; Item: TInventoryItem);
begin
  WriteLn(Format('LOW STOCK ALERT: %s (%s) - Current: %d, Minimum: %d',
    [Item.ItemCode, Item.Name, Item.Quantity, Item.MinimumStock]));
end;

procedure OnItemAdded(Sender: TObject; Item: TInventoryItem);
begin
  WriteLn(Format('Item added: %s - %s', [Item.ItemCode, Item.Name]));
end;

// Main program demonstration
var
  InventoryManager: TInventoryManager;
  Item: TInventoryItem;
  Items: TArray<TInventoryItem>;
  I: Integer;

begin
  try
    WriteLn('=== COMPREHENSIVE DELPHI INVENTORY MANAGEMENT SYSTEM ===');
    WriteLn('Initializing inventory management system...');
    WriteLn('');
    
    // Initialize the inventory manager
    InventoryManager := TInventoryManager.Create('test_inventory.json');
    try
      // Set up event handlers
      InventoryManager.OnStockLevelChanged := OnStockLevelChanged;
      InventoryManager.OnLowStockAlert := OnLowStockAlert;
      InventoryManager.OnItemAdded := OnItemAdded;
      
      WriteLn('1. Adding sample inventory items...');
      
      // Add sample items
      InventoryManager.AddItem('LAPTOP001', 'Dell Inspiron 15 3000', icElectronics, 25, 899.99, 650.00);
      InventoryManager.AddItem('TSHIRT001', 'Cotton T-Shirt Blue', icClothing, 150, 29.99, 15.00);
      InventoryManager.AddItem('BOOK001', 'Programming in Delphi', icBooks, 75, 49.99, 30.00);
      InventoryManager.AddItem('PHONE001', 'Smartphone XL', icElectronics, 50, 699.99, 450.00);
      InventoryManager.AddItem('JEANS001', 'Denim Jeans', icClothing, 80, 79.99, 40.00);
      
      // Set minimum stock levels
      InventoryManager.SetMinimumStock('LAPTOP001', 10);
      InventoryManager.SetMinimumStock('TSHIRT001', 50);
      InventoryManager.SetMinimumStock('BOOK001', 20);
      InventoryManager.SetMinimumStock('PHONE001', 15);
      InventoryManager.SetMinimumStock('JEANS001', 25);
      
      WriteLn('');
      WriteLn('2. Testing inventory operations...');
      
      // Test sales
      WriteLn('Processing sales...');
      InventoryManager.SellItem('LAPTOP001', 5, 'CUST001');
      InventoryManager.SellItem('TSHIRT001', 25, 'CUST002');
      InventoryManager.SellItem('PHONE001', 8, 'CUST003');
      
      // Test purchases
      WriteLn('Processing purchases...');
      InventoryManager.PurchaseItem('LAPTOP001', 15, 640.00, 'SUPP001');
      InventoryManager.PurchaseItem('BOOK001', 30, 28.00, 'SUPP003');
      
      // Test stock adjustments
      WriteLn('Processing stock adjustments...');
      InventoryManager.AdjustStock('TSHIRT001', -10, 'Damaged items removed');
      InventoryManager.AdjustStock('JEANS001', 20, 'Found in warehouse');
      
      // Test transfers
      WriteLn('Processing stock transfer...');
      try
        InventoryManager.TransferStock('TSHIRT001', 'JEANS001', 15);
      except
        on E: Exception do
          WriteLn('Transfer failed (expected): ', E.Message);
      end;
      
      WriteLn('');
      WriteLn('3. Generating reports...');
      
      // Display inventory report
      WriteLn(InventoryManager.GetInventoryReport);
      WriteLn('');
      
      // Display low stock report
      WriteLn(InventoryManager.GetLowStockReport);
      WriteLn('');
      
      // Display transaction report
      WriteLn(InventoryManager.GetTransactionReport(Date - 1, Date + 1));
      WriteLn('');
      
      // Display supplier report
      WriteLn(InventoryManager.GetSupplierReport);
      WriteLn('');
      
      WriteLn('4. Testing search and filtering...');
      
      // Search items
      Items := InventoryManager.SearchItems('Dell');
      WriteLn(Format('Search results for "Dell": %d items found', [Length(Items)]));
      for Item in Items do
        WriteLn('  ' + Item.ToString);
      WriteLn('');
      
      // Get items by category
      Items := InventoryManager.GetItemsByCategory(icElectronics);
      WriteLn(Format('Electronics category: %d items', [Length(Items)]));
      for Item in Items do
        WriteLn('  ' + Item.ToString);
      WriteLn('');
      
      // Get low stock items
      Items := InventoryManager.GetLowStockItems;
      WriteLn(Format('Low stock items: %d items', [Length(Items)]));
      for Item in Items do
        WriteLn('  ' + Item.ToString);
      WriteLn('');
      
      WriteLn('5. Testing advanced features...');
      
      // Test custom attributes
      Item := InventoryManager.FindItem('LAPTOP001');
      Item.AddCustomAttribute('Warranty', '2 years');
      Item.AddCustomAttribute('Color', 'Silver');
      Item.AddCustomAttribute('RAM', '8GB');
      WriteLn('Added custom attributes to LAPTOP001');
      WriteLn(Format('Warranty: %s', [Item.GetCustomAttribute('Warranty')]));
      WriteLn(Format('RAM: %s', [Item.GetCustomAttribute('RAM')]));
      WriteLn('');
      
      // Test item cloning
      var ClonedItem := Item.Clone;
      try
        WriteLn(Format('Cloned item: %s', [ClonedItem.ToString]));
      finally
        ClonedItem.Free;
      end;
      WriteLn('');
      
      WriteLn('6. Testing error handling...');
      
      // Test various error conditions
      try
        InventoryManager.AddItem('LAPTOP001', 'Duplicate Item', icElectronics, 10, 100, 50);
      except
        on E: EDuplicateItemException do
          WriteLn('Expected error - Duplicate item: ', E.Message);
      end;
      
      try
        InventoryManager.SellItem('NONEXISTENT', 1, 'CUST004');
      except
        on E: EItemNotFoundException do
          WriteLn('Expected error - Item not found: ', E.Message);
      end;
      
      try
        InventoryManager.SellItem('LAPTOP001', 1000, 'CUST005');
      except
        on E: EInsufficientStockException do
          WriteLn('Expected error - Insufficient stock: ', E.Message);
      end;
      
      try
        var InvalidItem := TInventoryItem.Create('', 'Invalid Item', icOther);
        InvalidItem.Free;
      except
        on E: EInvalidDataException do
          WriteLn('Expected error - Invalid data: ', E.Message);
      end;
      
      WriteLn('');
      WriteLn('7. Performance and statistics...');
      
      WriteLn(Format('Total items in inventory: %d', [InventoryManager.GetTotalItems]));
      WriteLn(Format('Total quantity across all items: %d', [InventoryManager.GetTotalQuantity]));
      WriteLn(Format('Total inventory value: %s', [FormatCurrency(InventoryManager.GetTotalInventoryValue)]));
      WriteLn(Format('Average stock value per item: %s', [FormatCurrency(InventoryManager.GetAverageStockValue)]));
      WriteLn(Format('Total transactions recorded: %d', [InventoryManager.Transactions.Count]));
      WriteLn('');
      
      WriteLn('8. Data export test...');
      InventoryManager.ExportToCSV('inventory_export.csv');
      WriteLn('Inventory data exported to CSV file');
      WriteLn('');
      
      WriteLn('9. Data backup test...');
      InventoryManager.BackupData();
      WriteLn('Inventory data backed up successfully');
      WriteLn('');
      
      WriteLn('10. Validation test...');
      InventoryManager.ValidateAllItems;
      WriteLn('');
      
      WriteLn('=== DELPHI INVENTORY MANAGEMENT SYSTEM TESTING COMPLETED ===');
      WriteLn('All features tested successfully!');
      WriteLn('');
      WriteLn('Key features demonstrated:');
      WriteLn('- Object-oriented design with inheritance and polymorphism');
      WriteLn('- Generic collections and custom data structures');
      WriteLn('- Comprehensive exception handling with custom exception types');
      WriteLn('- Event-driven architecture with callbacks');
      WriteLn('- Advanced string manipulation and formatting');
      WriteLn('- JSON serialization and file I/O operations');
      WriteLn('- Memory management and resource cleanup');
      WriteLn('- Complex business logic with validation and constraints');
      WriteLn('- Reporting and data analysis capabilities');
      WriteLn('- Search and filtering functionality');
      WriteLn('- Data import/export capabilities');
      WriteLn('- Comprehensive testing and error scenarios');
      
    finally
      InventoryManager.Free;
    end;
    
  except
    on E: Exception do
    begin
      WriteLn('Critical error: ', E.ClassName, ': ', E.Message);
      ExitCode := 1;
    end;
  end;
  
  WriteLn('');
  WriteLn('Press Enter to exit...');
  ReadLn;
end.