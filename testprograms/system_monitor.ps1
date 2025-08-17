#Requires -Version 5.0

<#
.SYNOPSIS
    Comprehensive System Monitoring and Management Tool in PowerShell

.DESCRIPTION
    This PowerShell script demonstrates advanced PowerShell concepts including:
    - Advanced functions with parameter validation
    - Object-oriented programming with classes
    - Error handling and exception management
    - WMI/CIM cmdlets for system information
    - Registry manipulation
    - Service and process management
    - Performance counter monitoring
    - Event log analysis
    - Scheduled task management
    - Network connectivity testing
    - File system operations
    - PowerShell remoting capabilities
    - Pipeline processing
    - Custom objects and formatting

.PARAMETER ComputerName
    Target computer(s) to monitor. Defaults to localhost.

.PARAMETER Credential
    Credentials for remote computer access.

.PARAMETER OutputPath
    Path to save monitoring reports.

.PARAMETER MonitoringInterval
    Interval in seconds between monitoring cycles.

.PARAMETER EnableAlerts
    Enable alerting for critical thresholds.

.EXAMPLE
    .\system_monitor.ps1 -ComputerName "SERVER01" -EnableAlerts

.EXAMPLE
    .\system_monitor.ps1 -OutputPath "C:\Reports" -MonitoringInterval 30

.NOTES
    Version: 1.0.0
    Author: Rigr Test Suite
    Purpose: PowerShell Language Demonstration
#>

[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true)]
    [ValidateNotNullOrEmpty()]
    [string[]]$ComputerName = $env:COMPUTERNAME,
    
    [Parameter()]
    [System.Management.Automation.PSCredential]$Credential,
    
    [Parameter()]
    [ValidateScript({
        if (-not (Test-Path -Path $_ -IsValid)) {
            throw "Invalid path specified: $_"
        }
        $true
    })]
    [string]$OutputPath = ".\Reports",
    
    [Parameter()]
    [ValidateRange(10, 3600)]
    [int]$MonitoringInterval = 60,
    
    [Parameter()]
    [switch]$EnableAlerts,
    
    [Parameter()]
    [switch]$GenerateReport,
    
    [Parameter()]
    [switch]$TestMode
)

#region Classes and Enums

# System monitoring status enumeration
enum SystemStatus {
    Healthy
    Warning
    Critical
    Unknown
}

# Performance threshold class
class PerformanceThreshold {
    [string]$CounterName
    [double]$WarningThreshold
    [double]$CriticalThreshold
    [string]$Operator  # GreaterThan, LessThan
    
    PerformanceThreshold([string]$counter, [double]$warning, [double]$critical, [string]$operator) {
        $this.CounterName = $counter
        $this.WarningThreshold = $warning
        $this.CriticalThreshold = $critical
        $this.Operator = $operator
    }
    
    [SystemStatus] EvaluateStatus([double]$value) {
        switch ($this.Operator) {
            'GreaterThan' {
                if ($value -ge $this.CriticalThreshold) { return [SystemStatus]::Critical }
                elseif ($value -ge $this.WarningThreshold) { return [SystemStatus]::Warning }
                else { return [SystemStatus]::Healthy }
            }
            'LessThan' {
                if ($value -le $this.CriticalThreshold) { return [SystemStatus]::Critical }
                elseif ($value -le $this.WarningThreshold) { return [SystemStatus]::Warning }
                else { return [SystemStatus]::Healthy }
            }
            default { return [SystemStatus]::Unknown }
        }
    }
}

# System monitoring result class
class SystemMonitoringResult {
    [string]$ComputerName
    [datetime]$Timestamp
    [SystemStatus]$OverallStatus
    [hashtable]$SystemInfo
    [hashtable]$PerformanceCounters
    [array]$RunningProcesses
    [array]$Services
    [array]$EventLogEntries
    [hashtable]$DiskUsage
    [hashtable]$NetworkInfo
    [array]$Alerts
    
    SystemMonitoringResult([string]$computerName) {
        $this.ComputerName = $computerName
        $this.Timestamp = Get-Date
        $this.OverallStatus = [SystemStatus]::Unknown
        $this.SystemInfo = @{}
        $this.PerformanceCounters = @{}
        $this.RunningProcesses = @()
        $this.Services = @()
        $this.EventLogEntries = @()
        $this.DiskUsage = @{}
        $this.NetworkInfo = @{}
        $this.Alerts = @()
    }
    
    [void] AddAlert([string]$severity, [string]$message, [string]$category) {
        $alert = [PSCustomObject]@{
            Timestamp = Get-Date
            Severity = $severity
            Category = $category
            Message = $message
        }
        $this.Alerts += $alert
    }
    
    [string] ToString() {
        return "System Monitoring Result for $($this.ComputerName) at $($this.Timestamp) - Status: $($this.OverallStatus)"
    }
}

#endregion

#region Configuration and Global Variables

# Performance counter thresholds
$script:PerformanceThresholds = @{
    'CPU_Usage' = [PerformanceThreshold]::new('% Processor Time', 80, 95, 'GreaterThan')
    'Memory_Usage' = [PerformanceThreshold]::new('Memory Usage %', 85, 95, 'GreaterThan')
    'Disk_Usage' = [PerformanceThreshold]::new('Disk Usage %', 85, 95, 'GreaterThan')
    'Network_Utilization' = [PerformanceThreshold]::new('Network Utilization %', 80, 95, 'GreaterThan')
    'Available_Memory' = [PerformanceThreshold]::new('Available Memory MB', 1024, 512, 'LessThan')
}

# Email settings for alerts (example configuration)
$script:AlertSettings = @{
    SmtpServer = 'smtp.company.com'
    From = 'monitoring@company.com'
    To = @('admin@company.com')
    Subject = 'System Monitoring Alert'
}

# Logging configuration
$script:LogPath = Join-Path $OutputPath 'SystemMonitoring.log'

#endregion

#region Utility Functions

function Write-Log {
    <#
    .SYNOPSIS
        Writes timestamped log entries to log file and console
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message,
        
        [Parameter()]
        [ValidateSet('Info', 'Warning', 'Error', 'Debug')]
        [string]$Level = 'Info'
    )
    
    begin {
        $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
        $logEntry = "[$timestamp] [$Level] $Message"
        
        # Ensure log directory exists
        $logDir = Split-Path -Path $script:LogPath -Parent
        if (-not (Test-Path -Path $logDir)) {
            New-Item -Path $logDir -ItemType Directory -Force | Out-Null
        }
    }
    
    process {
        # Write to console with appropriate color
        switch ($Level) {
            'Warning' { Write-Warning $logEntry }
            'Error' { Write-Error $logEntry }
            'Debug' { Write-Debug $logEntry }
            default { Write-Host $logEntry -ForegroundColor Green }
        }
        
        # Write to log file
        try {
            Add-Content -Path $script:LogPath -Value $logEntry -ErrorAction Stop
        }
        catch {
            Write-Warning "Failed to write to log file: $_"
        }
    }
}

function Test-ComputerConnectivity {
    <#
    .SYNOPSIS
        Tests network connectivity to target computers
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName,
        
        [Parameter()]
        [int]$TimeoutSeconds = 30
    )
    
    try {
        Write-Log "Testing connectivity to $ComputerName" -Level Debug
        
        # Test network connectivity
        $pingResult = Test-Connection -ComputerName $ComputerName -Count 2 -Quiet -ErrorAction Stop
        
        if ($pingResult) {
            # Test WMI connectivity
            $wmiTest = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_OperatingSystem -ErrorAction Stop
            Write-Log "Successfully connected to $ComputerName" -Level Debug
            return $true
        }
        else {
            Write-Log "Ping failed to $ComputerName" -Level Warning
            return $false
        }
    }
    catch {
        Write-Log "Connectivity test failed for $ComputerName`: $_" -Level Error
        return $false
    }
}

function Get-FormattedFileSize {
    <#
    .SYNOPSIS
        Formats file sizes in human-readable format
    #>
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [long]$Bytes
    )
    
    $units = @('B', 'KB', 'MB', 'GB', 'TB', 'PB')
    $unitIndex = 0
    $size = [double]$Bytes
    
    while ($size -ge 1024 -and $unitIndex -lt ($units.Length - 1)) {
        $size /= 1024
        $unitIndex++
    }
    
    return "{0:N2} {1}" -f $size, $units[$unitIndex]
}

#endregion

#region System Information Gathering Functions

function Get-SystemInformation {
    <#
    .SYNOPSIS
        Retrieves comprehensive system information
    #>
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName
    )
    
    try {
        Write-Log "Gathering system information for $ComputerName" -Level Debug
        
        # Operating System Information
        $os = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_OperatingSystem
        $computer = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_ComputerSystem
        $processor = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_Processor | Select-Object -First 1
        $bios = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_BIOS
        
        # Calculate uptime
        $uptime = (Get-Date) - $os.LastBootUpTime
        
        $systemInfo = @{
            ComputerName = $computer.Name
            Domain = $computer.Domain
            Manufacturer = $computer.Manufacturer
            Model = $computer.Model
            OperatingSystem = $os.Caption
            OSVersion = $os.Version
            OSBuild = $os.BuildNumber
            ServicePack = $os.CSDVersion
            Architecture = $os.OSArchitecture
            InstallDate = $os.InstallDate
            LastBootTime = $os.LastBootUpTime
            Uptime = "{0} days, {1} hours, {2} minutes" -f $uptime.Days, $uptime.Hours, $uptime.Minutes
            TotalPhysicalMemory = Get-FormattedFileSize $computer.TotalPhysicalMemory
            ProcessorName = $processor.Name
            ProcessorCores = $processor.NumberOfCores
            ProcessorLogicalProcessors = $processor.NumberOfLogicalProcessors
            BIOSVersion = $bios.SMBIOSBIOSVersion
            BIOSDate = $bios.ReleaseDate
            TimeZone = $os.CurrentTimeZone
            WindowsDirectory = $os.WindowsDirectory
            SystemDirectory = $os.SystemDirectory
        }
        
        Write-Log "Successfully gathered system information for $ComputerName" -Level Debug
        return $systemInfo
    }
    catch {
        Write-Log "Failed to gather system information for $ComputerName`: $_" -Level Error
        return @{}
    }
}

function Get-PerformanceCounters {
    <#
    .SYNOPSIS
        Retrieves performance counter data
    #>
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName
    )
    
    try {
        Write-Log "Gathering performance counters for $ComputerName" -Level Debug
        
        # Define performance counters to collect
        $counterPaths = @(
            "\Processor(_Total)\% Processor Time",
            "\Memory\Available MBytes",
            "\Memory\Committed Bytes",
            "\Memory\% Committed Bytes In Use",
            "\PhysicalDisk(_Total)\% Disk Time",
            "\PhysicalDisk(_Total)\Disk Reads/sec",
            "\PhysicalDisk(_Total)\Disk Writes/sec",
            "\Network Interface(*)\Bytes Total/sec",
            "\System\Processes",
            "\System\Threads",
            "\System\System Up Time"
        )
        
        $performanceData = @{}
        
        foreach ($counterPath in $counterPaths) {
            try {
                $counterValue = (Get-Counter -ComputerName $ComputerName -Counter $counterPath -SampleInterval 1 -MaxSamples 2 | 
                    Select-Object -ExpandProperty CounterSamples | 
                    Select-Object -Last 1).CookedValue
                
                $counterName = $counterPath -replace '\\.*\\', '' -replace '\(.*\)', ''
                $performanceData[$counterName] = [math]::Round($counterValue, 2)
            }
            catch {
                Write-Log "Failed to collect counter $counterPath`: $_" -Level Warning
            }
        }
        
        # Calculate derived metrics
        if ($performanceData.ContainsKey('Available MBytes')) {
            $totalMemoryGB = (Get-CimInstance -ComputerName $ComputerName -ClassName Win32_ComputerSystem).TotalPhysicalMemory / 1GB
            $availableMemoryGB = $performanceData['Available MBytes'] / 1024
            $memoryUsagePercent = [math]::Round((($totalMemoryGB - $availableMemoryGB) / $totalMemoryGB) * 100, 2)
            $performanceData['Memory Usage %'] = $memoryUsagePercent
        }
        
        Write-Log "Successfully gathered performance counters for $ComputerName" -Level Debug
        return $performanceData
    }
    catch {
        Write-Log "Failed to gather performance counters for $ComputerName`: $_" -Level Error
        return @{}
    }
}

function Get-ProcessInformation {
    <#
    .SYNOPSIS
        Retrieves information about running processes
    #>
    [CmdletBinding()]
    [OutputType([array])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName,
        
        [Parameter()]
        [int]$TopProcesses = 10
    )
    
    try {
        Write-Log "Gathering process information for $ComputerName" -Level Debug
        
        $processes = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_Process | 
            Where-Object { $_.WorkingSetSize -gt 0 } |
            Sort-Object WorkingSetSize -Descending |
            Select-Object -First $TopProcesses |
            ForEach-Object {
                [PSCustomObject]@{
                    ProcessName = $_.Name
                    ProcessId = $_.ProcessId
                    ParentProcessId = $_.ParentProcessId
                    WorkingSetSize = Get-FormattedFileSize $_.WorkingSetSize
                    VirtualSize = Get-FormattedFileSize $_.VirtualSize
                    CreationDate = $_.CreationDate
                    ExecutablePath = $_.ExecutablePath
                    CommandLine = $_.CommandLine
                    Priority = $_.Priority
                }
            }
        
        Write-Log "Successfully gathered process information for $ComputerName" -Level Debug
        return $processes
    }
    catch {
        Write-Log "Failed to gather process information for $ComputerName`: $_" -Level Error
        return @()
    }
}

function Get-ServiceInformation {
    <#
    .SYNOPSIS
        Retrieves information about Windows services
    #>
    [CmdletBinding()]
    [OutputType([array])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName
    )
    
    try {
        Write-Log "Gathering service information for $ComputerName" -Level Debug
        
        $services = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_Service |
            Where-Object { $_.State -ne 'Running' -and $_.StartMode -eq 'Auto' } |
            ForEach-Object {
                [PSCustomObject]@{
                    ServiceName = $_.Name
                    DisplayName = $_.DisplayName
                    State = $_.State
                    StartMode = $_.StartMode
                    Status = $_.Status
                    ProcessId = $_.ProcessId
                    PathName = $_.PathName
                    Description = $_.Description
                }
            }
        
        Write-Log "Found $($services.Count) non-running automatic services on $ComputerName" -Level Debug
        return $services
    }
    catch {
        Write-Log "Failed to gather service information for $ComputerName`: $_" -Level Error
        return @()
    }
}

function Get-DiskUsageInformation {
    <#
    .SYNOPSIS
        Retrieves disk usage information
    #>
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName
    )
    
    try {
        Write-Log "Gathering disk usage information for $ComputerName" -Level Debug
        
        $diskInfo = @{}
        
        Get-CimInstance -ComputerName $ComputerName -ClassName Win32_LogicalDisk |
            Where-Object { $_.DriveType -eq 3 } |  # Fixed drives only
            ForEach-Object {
                $usedSpace = $_.Size - $_.FreeSpace
                $usagePercent = [math]::Round(($usedSpace / $_.Size) * 100, 2)
                
                $diskInfo[$_.DeviceID] = [PSCustomObject]@{
                    Drive = $_.DeviceID
                    TotalSize = Get-FormattedFileSize $_.Size
                    FreeSpace = Get-FormattedFileSize $_.FreeSpace
                    UsedSpace = Get-FormattedFileSize $usedSpace
                    UsagePercent = $usagePercent
                    FileSystem = $_.FileSystem
                    VolumeName = $_.VolumeName
                }
            }
        
        Write-Log "Successfully gathered disk usage information for $ComputerName" -Level Debug
        return $diskInfo
    }
    catch {
        Write-Log "Failed to gather disk usage information for $ComputerName`: $_" -Level Error
        return @{}
    }
}

function Get-NetworkInformation {
    <#
    .SYNOPSIS
        Retrieves network adapter information
    #>
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName
    )
    
    try {
        Write-Log "Gathering network information for $ComputerName" -Level Debug
        
        $networkInfo = @{}
        $adapters = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_NetworkAdapter |
            Where-Object { $_.NetEnabled -eq $true -and $_.AdapterType -notlike '*Loopback*' }
        
        foreach ($adapter in $adapters) {
            $config = Get-CimInstance -ComputerName $ComputerName -ClassName Win32_NetworkAdapterConfiguration |
                Where-Object { $_.Index -eq $adapter.Index }
            
            $networkInfo[$adapter.Name] = [PSCustomObject]@{
                AdapterName = $adapter.Name
                Description = $adapter.Description
                MACAddress = $adapter.MACAddress
                Speed = if ($adapter.Speed) { Get-FormattedFileSize $adapter.Speed } else { 'Unknown' }
                IPAddress = $config.IPAddress -join ', '
                SubnetMask = $config.IPSubnet -join ', '
                DefaultGateway = $config.DefaultIPGateway -join ', '
                DNSServers = $config.DNSServerSearchOrder -join ', '
                DHCPEnabled = $config.DHCPEnabled
                AdapterType = $adapter.AdapterType
            }
        }
        
        Write-Log "Successfully gathered network information for $ComputerName" -Level Debug
        return $networkInfo
    }
    catch {
        Write-Log "Failed to gather network information for $ComputerName`: $_" -Level Error
        return @{}
    }
}

function Get-EventLogEntries {
    <#
    .SYNOPSIS
        Retrieves recent critical and error event log entries
    #>
    [CmdletBinding()]
    [OutputType([array])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName,
        
        [Parameter()]
        [int]$HoursBack = 24,
        
        [Parameter()]
        [int]$MaxEntries = 50
    )
    
    try {
        Write-Log "Gathering event log entries for $ComputerName" -Level Debug
        
        $startTime = (Get-Date).AddHours(-$HoursBack)
        $eventLogs = @('System', 'Application')
        $events = @()
        
        foreach ($logName in $eventLogs) {
            try {
                $logEntries = Get-WinEvent -ComputerName $ComputerName -FilterHashtable @{
                    LogName = $logName
                    Level = @(1, 2, 3)  # Critical, Error, Warning
                    StartTime = $startTime
                } -MaxEvents $MaxEntries -ErrorAction SilentlyContinue
                
                $events += $logEntries | ForEach-Object {
                    [PSCustomObject]@{
                        TimeCreated = $_.TimeCreated
                        LogName = $_.LogName
                        Level = $_.LevelDisplayName
                        Id = $_.Id
                        Source = $_.ProviderName
                        Message = $_.Message -replace '\r?\n', ' ' | 
                                  ForEach-Object { $_.Substring(0, [Math]::Min(200, $_.Length)) }
                    }
                }
            }
            catch {
                Write-Log "Failed to query $logName event log: $_" -Level Warning
            }
        }
        
        $events = $events | Sort-Object TimeCreated -Descending | Select-Object -First $MaxEntries
        
        Write-Log "Successfully gathered $($events.Count) event log entries for $ComputerName" -Level Debug
        return $events
    }
    catch {
        Write-Log "Failed to gather event log entries for $ComputerName`: $_" -Level Error
        return @()
    }
}

#endregion

#region Monitoring and Analysis Functions

function Invoke-SystemMonitoring {
    <#
    .SYNOPSIS
        Performs comprehensive system monitoring for a single computer
    #>
    [CmdletBinding()]
    [OutputType([SystemMonitoringResult])]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ComputerName
    )
    
    Write-Log "Starting system monitoring for $ComputerName"
    
    # Create monitoring result object
    $result = [SystemMonitoringResult]::new($ComputerName)
    
    # Test connectivity first
    if (-not (Test-ComputerConnectivity -ComputerName $ComputerName)) {
        $result.OverallStatus = [SystemStatus]::Critical
        $result.AddAlert('Critical', "Computer $ComputerName is not accessible", 'Connectivity')
        return $result
    }
    
    # Gather system information
    $result.SystemInfo = Get-SystemInformation -ComputerName $ComputerName
    $result.PerformanceCounters = Get-PerformanceCounters -ComputerName $ComputerName
    $result.RunningProcesses = Get-ProcessInformation -ComputerName $ComputerName
    $result.Services = Get-ServiceInformation -ComputerName $ComputerName
    $result.EventLogEntries = Get-EventLogEntries -ComputerName $ComputerName
    $result.DiskUsage = Get-DiskUsageInformation -ComputerName $ComputerName
    $result.NetworkInfo = Get-NetworkInformation -ComputerName $ComputerName
    
    # Analyze performance thresholds
    $statusLevels = @()
    
    foreach ($thresholdName in $script:PerformanceThresholds.Keys) {
        $threshold = $script:PerformanceThresholds[$thresholdName]
        $counterValue = $null
        
        # Map threshold names to actual counter values
        switch ($thresholdName) {
            'CPU_Usage' { 
                $counterValue = $result.PerformanceCounters['% Processor Time']
            }
            'Memory_Usage' { 
                $counterValue = $result.PerformanceCounters['Memory Usage %']
            }
            'Available_Memory' { 
                $counterValue = $result.PerformanceCounters['Available MBytes']
            }
        }
        
        if ($null -ne $counterValue) {
            $status = $threshold.EvaluateStatus($counterValue)
            $statusLevels += $status
            
            if ($status -eq [SystemStatus]::Critical) {
                $result.AddAlert('Critical', "$thresholdName is at critical level: $counterValue", 'Performance')
            }
            elseif ($status -eq [SystemStatus]::Warning) {
                $result.AddAlert('Warning', "$thresholdName is at warning level: $counterValue", 'Performance')
            }
        }
    }
    
    # Check disk usage
    foreach ($drive in $result.DiskUsage.Keys) {
        $diskInfo = $result.DiskUsage[$drive]
        if ($diskInfo.UsagePercent -ge 95) {
            $result.AddAlert('Critical', "Drive $drive is critically full: $($diskInfo.UsagePercent)%", 'DiskSpace')
            $statusLevels += [SystemStatus]::Critical
        }
        elseif ($diskInfo.UsagePercent -ge 85) {
            $result.AddAlert('Warning', "Drive $drive is getting full: $($diskInfo.UsagePercent)%", 'DiskSpace')
            $statusLevels += [SystemStatus]::Warning
        }
    }
    
    # Check for stopped automatic services
    if ($result.Services.Count -gt 0) {
        $result.AddAlert('Warning', "$($result.Services.Count) automatic services are not running", 'Services')
        $statusLevels += [SystemStatus]::Warning
    }
    
    # Check for critical events
    $criticalEvents = $result.EventLogEntries | Where-Object { $_.Level -eq 'Critical' }
    if ($criticalEvents.Count -gt 0) {
        $result.AddAlert('Critical', "$($criticalEvents.Count) critical events found in last 24 hours", 'EventLog')
        $statusLevels += [SystemStatus]::Critical
    }
    
    # Determine overall status
    if ($statusLevels -contains [SystemStatus]::Critical) {
        $result.OverallStatus = [SystemStatus]::Critical
    }
    elseif ($statusLevels -contains [SystemStatus]::Warning) {
        $result.OverallStatus = [SystemStatus]::Warning
    }
    else {
        $result.OverallStatus = [SystemStatus]::Healthy
    }
    
    Write-Log "Completed system monitoring for $ComputerName - Status: $($result.OverallStatus)"
    return $result
}

function Send-MonitoringAlert {
    <#
    .SYNOPSIS
        Sends monitoring alerts via email
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [SystemMonitoringResult]$MonitoringResult
    )
    
    if (-not $EnableAlerts -or $MonitoringResult.Alerts.Count -eq 0) {
        return
    }
    
    try {
        $alertBody = @"
System Monitoring Alert for $($MonitoringResult.ComputerName)
Generated: $($MonitoringResult.Timestamp)
Overall Status: $($MonitoringResult.OverallStatus)

ALERTS:
"@
        
        foreach ($alert in $MonitoringResult.Alerts) {
            $alertBody += "`n[$($alert.Severity)] [$($alert.Category)] $($alert.Message)"
        }
        
        $alertBody += @"

SYSTEM SUMMARY:
- Uptime: $($MonitoringResult.SystemInfo.Uptime)
- CPU Usage: $($MonitoringResult.PerformanceCounters['% Processor Time'])%
- Memory Usage: $($MonitoringResult.PerformanceCounters['Memory Usage %'])%
- Available Memory: $($MonitoringResult.PerformanceCounters['Available MBytes']) MB

For detailed information, please check the full monitoring report.
"@
        
        # In a real implementation, you would send the email here
        Write-Log "Alert notification prepared for $($MonitoringResult.ComputerName)" -Level Warning
        
        # For demonstration, we'll just write the alert to a file
        $alertFile = Join-Path $OutputPath "Alert_$($MonitoringResult.ComputerName)_$(Get-Date -Format 'yyyyMMdd_HHmmss').txt"
        $alertBody | Out-File -FilePath $alertFile -Encoding UTF8
        Write-Log "Alert written to file: $alertFile"
        
    }
    catch {
        Write-Log "Failed to send monitoring alert: $_" -Level Error
    }
}

#endregion

#region Report Generation Functions

function Export-MonitoringReport {
    <#
    .SYNOPSIS
        Exports monitoring results to various formats
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [SystemMonitoringResult[]]$MonitoringResults,
        
        [Parameter()]
        [ValidateSet('HTML', 'CSV', 'JSON', 'XML')]
        [string]$Format = 'HTML'
    )
    
    $timestamp = Get-Date -Format 'yyyyMMdd_HHmmss'
    $reportPath = Join-Path $OutputPath "SystemMonitoring_$timestamp.$($Format.ToLower())"
    
    try {
        switch ($Format) {
            'HTML' {
                $htmlReport = Generate-HTMLReport -MonitoringResults $MonitoringResults
                $htmlReport | Out-File -FilePath $reportPath -Encoding UTF8
            }
            'CSV' {
                $csvData = $MonitoringResults | ForEach-Object {
                    [PSCustomObject]@{
                        ComputerName = $_.ComputerName
                        Timestamp = $_.Timestamp
                        OverallStatus = $_.OverallStatus
                        CPUUsage = $_.PerformanceCounters['% Processor Time']
                        MemoryUsage = $_.PerformanceCounters['Memory Usage %']
                        AvailableMemory = $_.PerformanceCounters['Available MBytes']
                        AlertCount = $_.Alerts.Count
                        ProcessCount = $_.RunningProcesses.Count
                        StoppedServices = $_.Services.Count
                        Uptime = $_.SystemInfo.Uptime
                    }
                }
                $csvData | Export-Csv -Path $reportPath -NoTypeInformation
            }
            'JSON' {
                $MonitoringResults | ConvertTo-Json -Depth 10 | Out-File -FilePath $reportPath -Encoding UTF8
            }
            'XML' {
                $MonitoringResults | Export-Clixml -Path $reportPath
            }
        }
        
        Write-Log "Monitoring report exported: $reportPath"
        return $reportPath
    }
    catch {
        Write-Log "Failed to export monitoring report: $_" -Level Error
        return $null
    }
}

function Generate-HTMLReport {
    <#
    .SYNOPSIS
        Generates an HTML monitoring report
    #>
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory = $true)]
        [SystemMonitoringResult[]]$MonitoringResults
    )
    
    $html = @"
<!DOCTYPE html>
<html>
<head>
    <title>System Monitoring Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #2c3e50; color: white; padding: 20px; border-radius: 5px; }
        .summary { background-color: #ecf0f1; padding: 15px; margin: 20px 0; border-radius: 5px; }
        .computer { border: 1px solid #bdc3c7; margin: 20px 0; padding: 15px; border-radius: 5px; }
        .status-healthy { color: #27ae60; font-weight: bold; }
        .status-warning { color: #f39c12; font-weight: bold; }
        .status-critical { color: #e74c3c; font-weight: bold; }
        .alert { padding: 10px; margin: 5px 0; border-radius: 3px; }
        .alert-critical { background-color: #fadbd8; border-left: 5px solid #e74c3c; }
        .alert-warning { background-color: #fdeaa7; border-left: 5px solid #f39c12; }
        table { border-collapse: collapse; width: 100%; margin: 10px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #34495e; color: white; }
    </style>
</head>
<body>
    <div class="header">
        <h1>System Monitoring Report</h1>
        <p>Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')</p>
        <p>Computers Monitored: $($MonitoringResults.Count)</p>
    </div>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Healthy Systems:</strong> $($MonitoringResults | Where-Object {$_.OverallStatus -eq 'Healthy'} | Measure-Object | Select-Object -ExpandProperty Count)</p>
        <p><strong>Systems with Warnings:</strong> $($MonitoringResults | Where-Object {$_.OverallStatus -eq 'Warning'} | Measure-Object | Select-Object -ExpandProperty Count)</p>
        <p><strong>Critical Systems:</strong> $($MonitoringResults | Where-Object {$_.OverallStatus -eq 'Critical'} | Measure-Object | Select-Object -ExpandProperty Count)</p>
    </div>
"@
    
    foreach ($result in $MonitoringResults) {
        $statusClass = "status-$($result.OverallStatus.ToString().ToLower())"
        
        $html += @"
    <div class="computer">
        <h2>$($result.ComputerName) <span class="$statusClass">[$($result.OverallStatus)]</span></h2>
        
        <h3>System Information</h3>
        <table>
            <tr><th>Property</th><th>Value</th></tr>
            <tr><td>Operating System</td><td>$($result.SystemInfo.OperatingSystem)</td></tr>
            <tr><td>Uptime</td><td>$($result.SystemInfo.Uptime)</td></tr>
            <tr><td>Total Memory</td><td>$($result.SystemInfo.TotalPhysicalMemory)</td></tr>
            <tr><td>Processor</td><td>$($result.SystemInfo.ProcessorName)</td></tr>
        </table>
        
        <h3>Performance Metrics</h3>
        <table>
            <tr><th>Metric</th><th>Value</th></tr>
            <tr><td>CPU Usage</td><td>$($result.PerformanceCounters['% Processor Time'])%</td></tr>
            <tr><td>Memory Usage</td><td>$($result.PerformanceCounters['Memory Usage %'])%</td></tr>
            <tr><td>Available Memory</td><td>$($result.PerformanceCounters['Available MBytes']) MB</td></tr>
        </table>
"@
        
        if ($result.Alerts.Count -gt 0) {
            $html += "<h3>Alerts</h3>"
            foreach ($alert in $result.Alerts) {
                $alertClass = "alert-$($alert.Severity.ToLower())"
                $html += "<div class='alert $alertClass'><strong>$($alert.Severity):</strong> $($alert.Message)</div>"
            }
        }
        
        $html += "</div>"
    }
    
    $html += @"
    <div class="summary">
        <p><em>Report generated by PowerShell System Monitoring Tool v1.0.0</em></p>
    </div>
</body>
</html>
"@
    
    return $html
}

#endregion

#region Main Execution Functions

function Invoke-ContinuousMonitoring {
    <#
    .SYNOPSIS
        Performs continuous monitoring with specified interval
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$ComputerNames,
        
        [Parameter()]
        [int]$Cycles = -1  # -1 for infinite
    )
    
    $cycleCount = 0
    
    Write-Log "Starting continuous monitoring for $($ComputerNames.Count) computer(s)"
    Write-Log "Monitoring interval: $MonitoringInterval seconds"
    
    do {
        $cycleCount++
        Write-Log "=== Monitoring Cycle $cycleCount ==="
        
        $monitoringResults = @()
        
        foreach ($computerName in $ComputerNames) {
            try {
                $result = Invoke-SystemMonitoring -ComputerName $computerName
                $monitoringResults += $result
                
                # Send alerts if enabled
                if ($EnableAlerts -and $result.Alerts.Count -gt 0) {
                    Send-MonitoringAlert -MonitoringResult $result
                }
            }
            catch {
                Write-Log "Failed to monitor $computerName`: $_" -Level Error
            }
        }
        
        # Generate report if requested
        if ($GenerateReport) {
            Export-MonitoringReport -MonitoringResults $monitoringResults -Format HTML
        }
        
        # Display summary
        $healthyCount = ($monitoringResults | Where-Object { $_.OverallStatus -eq 'Healthy' }).Count
        $warningCount = ($monitoringResults | Where-Object { $_.OverallStatus -eq 'Warning' }).Count
        $criticalCount = ($monitoringResults | Where-Object { $_.OverallStatus -eq 'Critical' }).Count
        
        Write-Log "Cycle $cycleCount Summary: $healthyCount Healthy, $warningCount Warning, $criticalCount Critical"
        
        # Wait for next cycle (unless it's the last cycle)
        if ($Cycles -eq -1 -or $cycleCount -lt $Cycles) {
            Write-Log "Waiting $MonitoringInterval seconds until next monitoring cycle..."
            Start-Sleep -Seconds $MonitoringInterval
        }
        
    } while ($Cycles -eq -1 -or $cycleCount -lt $Cycles)
    
    Write-Log "Continuous monitoring completed after $cycleCount cycles"
}

function Show-MonitoringResults {
    <#
    .SYNOPSIS
        Displays monitoring results in a formatted table
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [SystemMonitoringResult[]]$MonitoringResults
    )
    
    begin {
        Write-Host "`n" + "="*100
        Write-Host "SYSTEM MONITORING RESULTS" -ForegroundColor Yellow
        Write-Host "="*100
    }
    
    process {
        foreach ($result in $MonitoringResults) {
            # Determine color based on status
            $statusColor = switch ($result.OverallStatus) {
                'Healthy' { 'Green' }
                'Warning' { 'Yellow' }
                'Critical' { 'Red' }
                default { 'Gray' }
            }
            
            Write-Host "`nComputer: $($result.ComputerName)" -ForegroundColor Cyan
            Write-Host "Status: $($result.OverallStatus)" -ForegroundColor $statusColor
            Write-Host "Timestamp: $($result.Timestamp)"
            Write-Host "Uptime: $($result.SystemInfo.Uptime)"
            
            # Performance summary
            Write-Host "`nPerformance Summary:" -ForegroundColor Yellow
            Write-Host "  CPU Usage: $($result.PerformanceCounters['% Processor Time'])%"
            Write-Host "  Memory Usage: $($result.PerformanceCounters['Memory Usage %'])%"
            Write-Host "  Available Memory: $($result.PerformanceCounters['Available MBytes']) MB"
            
            # Alerts
            if ($result.Alerts.Count -gt 0) {
                Write-Host "`nAlerts:" -ForegroundColor Red
                foreach ($alert in $result.Alerts) {
                    $alertColor = if ($alert.Severity -eq 'Critical') { 'Red' } else { 'Yellow' }
                    Write-Host "  [$($alert.Severity)] $($alert.Message)" -ForegroundColor $alertColor
                }
            }
            
            Write-Host "-"*80
        }
    }
}

#endregion

#region Main Program Logic

# Main execution block
try {
    Write-Host "="*80 -ForegroundColor Green
    Write-Host "PowerShell System Monitoring and Management Tool v1.0.0" -ForegroundColor Green
    Write-Host "Comprehensive system monitoring with performance analysis" -ForegroundColor Green
    Write-Host "="*80 -ForegroundColor Green
    Write-Host ""
    
    # Ensure output directory exists
    if (-not (Test-Path -Path $OutputPath)) {
        New-Item -Path $OutputPath -ItemType Directory -Force | Out-Null
        Write-Log "Created output directory: $OutputPath"
    }
    
    # Initialize logging
    Write-Log "Starting PowerShell System Monitoring Tool"
    Write-Log "Target computers: $($ComputerName -join ', ')"
    Write-Log "Output path: $OutputPath"
    Write-Log "Monitoring interval: $MonitoringInterval seconds"
    
    if ($TestMode) {
        Write-Log "Running in test mode - single monitoring cycle"
        
        # Perform single monitoring cycle
        $monitoringResults = @()
        foreach ($computer in $ComputerName) {
            $result = Invoke-SystemMonitoring -ComputerName $computer
            $monitoringResults += $result
        }
        
        # Display results
        $monitoringResults | Show-MonitoringResults
        
        # Generate report
        if ($GenerateReport) {
            $reportPath = Export-MonitoringReport -MonitoringResults $monitoringResults -Format HTML
            Write-Log "Test report generated: $reportPath"
        }
    }
    else {
        # Start continuous monitoring
        Invoke-ContinuousMonitoring -ComputerNames $ComputerName -Cycles 3  # 3 cycles for demo
    }
    
    Write-Log "System monitoring completed successfully"
    
    # Display summary of demonstrated PowerShell features
    Write-Host "`n" + "="*80 -ForegroundColor Green
    Write-Host "PowerShell Features Demonstrated:" -ForegroundColor Yellow
    Write-Host "- Advanced functions with parameter validation" -ForegroundColor White
    Write-Host "- Object-oriented programming with classes" -ForegroundColor White
    Write-Host "- Error handling and exception management" -ForegroundColor White
    Write-Host "- WMI/CIM cmdlets for system information" -ForegroundColor White
    Write-Host "- Performance counter monitoring" -ForegroundColor White
    Write-Host "- Event log analysis" -ForegroundColor White
    Write-Host "- Custom objects and formatting" -ForegroundColor White
    Write-Host "- Pipeline processing" -ForegroundColor White
    Write-Host "- File system operations" -ForegroundColor White
    Write-Host "- HTML report generation" -ForegroundColor White
    Write-Host "- Comprehensive logging system" -ForegroundColor White
    Write-Host "- Configuration management" -ForegroundColor White
    Write-Host "="*80 -ForegroundColor Green
}
catch {
    Write-Log "Critical error in main execution: $_" -Level Error
    Write-Host "Critical Error: $_" -ForegroundColor Red
    exit 1
}
finally {
    Write-Log "PowerShell System Monitoring Tool session ended"
}

# End of script