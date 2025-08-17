@echo off
REM ============================================================================
REM Comprehensive Backup Automation System - Windows Batch Script
REM 
REM This batch script demonstrates various batch programming concepts including:
REM - Command-line argument processing and validation
REM - Environment variable manipulation
REM - File and directory operations
REM - Conditional logic and flow control
REM - Loop constructs (FOR loops)
REM - Error handling and logging
REM - Date/time manipulation
REM - Registry operations
REM - Service management
REM - Network operations
REM - Compression and archiving
REM - Email notifications
REM - Task scheduling integration
REM - Advanced batch techniques and optimizations
REM ============================================================================

setlocal enabledelayedexpansion
setlocal enableextensions

REM Script configuration and constants
set SCRIPT_NAME=Comprehensive Backup Automation System
set SCRIPT_VERSION=1.0.0
set SCRIPT_AUTHOR=Rigr Test Suite
set DEBUG_MODE=0

REM Default configuration values
set SOURCE_DIR=""
set DESTINATION_DIR=""
set BACKUP_TYPE=INCREMENTAL
set RETENTION_DAYS=30
set COMPRESS_BACKUPS=1
set VERIFY_BACKUPS=1
set EMAIL_NOTIFICATIONS=0
set LOG_LEVEL=INFO
set MAX_LOG_SIZE=10485760
set CONFIG_FILE=%~dp0backup_config.ini

REM System variables
set BACKUP_DATE=%DATE:~-4,4%%DATE:~-10,2%%DATE:~-7,2%
set BACKUP_TIME=%TIME:~0,2%%TIME:~3,2%%TIME:~6,2%
set BACKUP_TIMESTAMP=%BACKUP_DATE%_%BACKUP_TIME: =0%
set LOG_FILE=%~dp0logs\backup_%BACKUP_DATE%.log
set ERROR_LOG=%~dp0logs\backup_errors_%BACKUP_DATE%.log
set STATS_FILE=%~dp0logs\backup_stats_%BACKUP_DATE%.csv

REM Performance counters
set FILES_COPIED=0
set FILES_SKIPPED=0
set BYTES_COPIED=0
set ERRORS_ENCOUNTERED=0
set START_TIME=%TIME%

REM Create necessary directories
if not exist "%~dp0logs" mkdir "%~dp0logs"
if not exist "%~dp0temp" mkdir "%~dp0temp"
if not exist "%~dp0config" mkdir "%~dp0config"

REM ============================================================================
REM Utility Functions and Subroutines
REM ============================================================================

:WriteLog
REM Function: WriteLog - Writes timestamped messages to log file
REM Parameters: %1 = Log level, %2 = Message
if "%2"=="" goto :eof
set LOG_TIMESTAMP=%DATE% %TIME:~0,8%
echo [%LOG_TIMESTAMP%] [%1] %~2 >> "%LOG_FILE%"
if "%DEBUG_MODE%"=="1" echo [DEBUG] [%LOG_TIMESTAMP%] [%1] %~2
if /i "%1"=="ERROR" (
    echo [%LOG_TIMESTAMP%] [%1] %~2 >> "%ERROR_LOG%"
    echo ERROR: %~2
    set /a ERRORS_ENCOUNTERED+=1
)
goto :eof

:DisplayHeader
REM Function: DisplayHeader - Displays script header information
echo.
echo ================================================================================
echo %SCRIPT_NAME% v%SCRIPT_VERSION%
echo %SCRIPT_AUTHOR%
echo ================================================================================
echo Started: %DATE% %TIME:~0,8%
echo Configuration: %CONFIG_FILE%
echo Log File: %LOG_FILE%
echo.
goto :eof

:ParseArguments
REM Function: ParseArguments - Processes command-line arguments
call :WriteLog INFO "Parsing command-line arguments"

:ArgLoop
if "%1"=="" goto :ArgsDone
if /i "%1"=="/source" (
    set SOURCE_DIR=%~2
    shift
    shift
    goto :ArgLoop
)
if /i "%1"=="/dest" (
    set DESTINATION_DIR=%~2
    shift
    shift
    goto :ArgLoop
)
if /i "%1"=="/type" (
    set BACKUP_TYPE=%~2
    shift
    shift
    goto :ArgLoop
)
if /i "%1"=="/retention" (
    set RETENTION_DAYS=%~2
    shift
    shift
    goto :ArgLoop
)
if /i "%1"=="/config" (
    set CONFIG_FILE=%~2
    shift
    shift
    goto :ArgLoop
)
if /i "%1"=="/compress" (
    set COMPRESS_BACKUPS=1
    shift
    goto :ArgLoop
)
if /i "%1"=="/nocompress" (
    set COMPRESS_BACKUPS=0
    shift
    goto :ArgLoop
)
if /i "%1"=="/verify" (
    set VERIFY_BACKUPS=1
    shift
    goto :ArgLoop
)
if /i "%1"=="/noverify" (
    set VERIFY_BACKUPS=0
    shift
    goto :ArgLoop
)
if /i "%1"=="/email" (
    set EMAIL_NOTIFICATIONS=1
    shift
    goto :ArgLoop
)
if /i "%1"=="/debug" (
    set DEBUG_MODE=1
    set LOG_LEVEL=DEBUG
    shift
    goto :ArgLoop
)
if /i "%1"=="/help" goto :ShowHelp
if /i "%1"=="/?" goto :ShowHelp

REM Unknown argument
call :WriteLog WARNING "Unknown argument: %1"
shift
goto :ArgLoop

:ArgsDone
call :WriteLog INFO "Command-line arguments parsed successfully"
goto :eof

:ShowHelp
echo.
echo USAGE: %~nx0 [OPTIONS]
echo.
echo OPTIONS:
echo   /source DIR        Source directory to backup
echo   /dest DIR          Destination directory for backups
echo   /type TYPE         Backup type (FULL, INCREMENTAL, DIFFERENTIAL)
echo   /retention DAYS    Number of days to retain backups (default: 30)
echo   /config FILE       Configuration file path
echo   /compress          Enable backup compression (default)
echo   /nocompress        Disable backup compression
echo   /verify            Verify backups after creation (default)
echo   /noverify          Skip backup verification
echo   /email             Enable email notifications
echo   /debug             Enable debug mode
echo   /help, /?          Show this help message
echo.
echo EXAMPLES:
echo   %~nx0 /source "C:\Data" /dest "D:\Backups" /type FULL
echo   %~nx0 /source "C:\Users" /dest "\\Server\Backups" /compress /email
echo   %~nx0 /config "custom_config.ini" /debug
echo.
goto :EndScript

:LoadConfiguration
REM Function: LoadConfiguration - Loads settings from configuration file
call :WriteLog INFO "Loading configuration from: %CONFIG_FILE%"

if not exist "%CONFIG_FILE%" (
    call :WriteLog WARNING "Configuration file not found, creating default: %CONFIG_FILE%"
    call :CreateDefaultConfig
)

REM Parse INI file (simplified parser)
for /f "usebackq delims== tokens=1,2" %%A in ("%CONFIG_FILE%") do (
    set CONFIG_KEY=%%A
    set CONFIG_VALUE=%%B
    
    REM Remove leading/trailing spaces
    for /f "tokens=* delims= " %%C in ("!CONFIG_KEY!") do set CONFIG_KEY=%%C
    for /f "tokens=* delims= " %%D in ("!CONFIG_VALUE!") do set CONFIG_VALUE=%%D
    
    REM Process configuration values
    if /i "!CONFIG_KEY!"=="SOURCE_DIR" set SOURCE_DIR=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="DESTINATION_DIR" set DESTINATION_DIR=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="BACKUP_TYPE" set BACKUP_TYPE=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="RETENTION_DAYS" set RETENTION_DAYS=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="COMPRESS_BACKUPS" set COMPRESS_BACKUPS=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="VERIFY_BACKUPS" set VERIFY_BACKUPS=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="EMAIL_NOTIFICATIONS" set EMAIL_NOTIFICATIONS=!CONFIG_VALUE!
    if /i "!CONFIG_KEY!"=="LOG_LEVEL" set LOG_LEVEL=!CONFIG_VALUE!
)

call :WriteLog INFO "Configuration loaded successfully"
goto :eof

:CreateDefaultConfig
REM Function: CreateDefaultConfig - Creates a default configuration file
call :WriteLog INFO "Creating default configuration file"

(
echo # Comprehensive Backup Automation System Configuration
echo # Generated: %DATE% %TIME:~0,8%
echo.
echo [BACKUP_SETTINGS]
echo SOURCE_DIR=C:\BackupSource
echo DESTINATION_DIR=C:\BackupDestination
echo BACKUP_TYPE=INCREMENTAL
echo RETENTION_DAYS=30
echo COMPRESS_BACKUPS=1
echo VERIFY_BACKUPS=1
echo.
echo [NOTIFICATION_SETTINGS]
echo EMAIL_NOTIFICATIONS=0
echo EMAIL_SMTP_SERVER=smtp.company.com
echo EMAIL_FROM=backup@company.com
echo EMAIL_TO=admin@company.com
echo.
echo [LOGGING_SETTINGS]
echo LOG_LEVEL=INFO
echo MAX_LOG_SIZE=10485760
echo ARCHIVE_OLD_LOGS=1
echo.
echo [ADVANCED_SETTINGS]
echo MAX_PARALLEL_OPERATIONS=4
echo NETWORK_TIMEOUT=300
echo RETRY_COUNT=3
echo TEMP_DIRECTORY=%~dp0temp
) > "%CONFIG_FILE%"

call :WriteLog INFO "Default configuration file created: %CONFIG_FILE%"
goto :eof

:ValidateConfiguration
REM Function: ValidateConfiguration - Validates configuration settings
call :WriteLog INFO "Validating configuration settings"

set VALIDATION_ERRORS=0

REM Validate source directory
if "%SOURCE_DIR%"=="" (
    call :WriteLog ERROR "Source directory not specified"
    set /a VALIDATION_ERRORS+=1
) else if not exist "%SOURCE_DIR%" (
    call :WriteLog ERROR "Source directory does not exist: %SOURCE_DIR%"
    set /a VALIDATION_ERRORS+=1
) else (
    call :WriteLog INFO "Source directory validated: %SOURCE_DIR%"
)

REM Validate destination directory
if "%DESTINATION_DIR%"=="" (
    call :WriteLog ERROR "Destination directory not specified"
    set /a VALIDATION_ERRORS+=1
) else (
    REM Create destination directory if it doesn't exist
    if not exist "%DESTINATION_DIR%" (
        call :WriteLog INFO "Creating destination directory: %DESTINATION_DIR%"
        mkdir "%DESTINATION_DIR%" 2>nul
        if errorlevel 1 (
            call :WriteLog ERROR "Failed to create destination directory: %DESTINATION_DIR%"
            set /a VALIDATION_ERRORS+=1
        )
    ) else (
        call :WriteLog INFO "Destination directory validated: %DESTINATION_DIR%"
    )
)

REM Validate backup type
if /i not "%BACKUP_TYPE%"=="FULL" if /i not "%BACKUP_TYPE%"=="INCREMENTAL" if /i not "%BACKUP_TYPE%"=="DIFFERENTIAL" (
    call :WriteLog ERROR "Invalid backup type: %BACKUP_TYPE%. Must be FULL, INCREMENTAL, or DIFFERENTIAL"
    set /a VALIDATION_ERRORS+=1
) else (
    call :WriteLog INFO "Backup type validated: %BACKUP_TYPE%"
)

REM Validate retention days
if %RETENTION_DAYS% LEQ 0 (
    call :WriteLog ERROR "Invalid retention days: %RETENTION_DAYS%. Must be greater than 0"
    set /a VALIDATION_ERRORS+=1
) else (
    call :WriteLog INFO "Retention days validated: %RETENTION_DAYS%"
)

if %VALIDATION_ERRORS% GTR 0 (
    call :WriteLog ERROR "Configuration validation failed with %VALIDATION_ERRORS% error(s)"
    goto :EndScript
) else (
    call :WriteLog INFO "Configuration validation successful"
)

goto :eof

:CheckSystemResources
REM Function: CheckSystemResources - Checks available system resources
call :WriteLog INFO "Checking system resources"

REM Check available disk space on destination
for /f "tokens=3" %%A in ('dir "%DESTINATION_DIR%" ^| find /i "bytes free"') do (
    set AVAILABLE_SPACE=%%A
)

REM Check available memory
for /f "skip=1 tokens=4" %%A in ('wmic OS get TotalVisibleMemorySize /value') do (
    if not "%%A"=="" set TOTAL_MEMORY=%%A
)

for /f "skip=1 tokens=4" %%A in ('wmic OS get FreePhysicalMemory /value') do (
    if not "%%A"=="" set FREE_MEMORY=%%A
)

REM Check CPU usage (simplified)
for /f "skip=1 tokens=2" %%A in ('wmic cpu get loadpercentage /value') do (
    if not "%%A"=="" set CPU_USAGE=%%A
)

call :WriteLog INFO "System resources checked - Available space: %AVAILABLE_SPACE%, Free memory: %FREE_MEMORY%KB, CPU usage: %CPU_USAGE%%%"
goto :eof

:PerformBackup
REM Function: PerformBackup - Main backup operation
call :WriteLog INFO "Starting %BACKUP_TYPE% backup operation"
call :WriteLog INFO "Source: %SOURCE_DIR%"
call :WriteLog INFO "Destination: %DESTINATION_DIR%"

REM Create backup directory structure
set BACKUP_DIR=%DESTINATION_DIR%\%BACKUP_TIMESTAMP%_%BACKUP_TYPE%
call :WriteLog INFO "Creating backup directory: %BACKUP_DIR%"
mkdir "%BACKUP_DIR%" 2>nul

if errorlevel 1 (
    call :WriteLog ERROR "Failed to create backup directory: %BACKUP_DIR%"
    goto :eof
)

REM Determine backup strategy
if /i "%BACKUP_TYPE%"=="FULL" goto :PerformFullBackup
if /i "%BACKUP_TYPE%"=="INCREMENTAL" goto :PerformIncrementalBackup
if /i "%BACKUP_TYPE%"=="DIFFERENTIAL" goto :PerformDifferentialBackup

call :WriteLog ERROR "Unknown backup type: %BACKUP_TYPE%"
goto :eof

:PerformFullBackup
REM Function: PerformFullBackup - Performs a full backup
call :WriteLog INFO "Performing full backup"

REM Use XCOPY for full backup with detailed logging
xcopy "%SOURCE_DIR%" "%BACKUP_DIR%" /E /H /K /Y /C /I /F /R > "%~dp0temp\xcopy_output.txt" 2>&1

if errorlevel 1 (
    call :WriteLog ERROR "Full backup failed with error level %errorlevel%"
    type "%~dp0temp\xcopy_output.txt" >> "%ERROR_LOG%"
) else (
    call :WriteLog INFO "Full backup completed successfully"
    REM Count copied files
    for /f %%A in ('findstr /C:"File(s) copied" "%~dp0temp\xcopy_output.txt"') do set FILES_COPIED=%%A
)

goto :PostBackupTasks

:PerformIncrementalBackup
REM Function: PerformIncrementalBackup - Performs an incremental backup
call :WriteLog INFO "Performing incremental backup"

REM Find the last backup date for incremental comparison
set LAST_BACKUP_DATE=
for /f "delims=" %%A in ('dir "%DESTINATION_DIR%" /B /AD /O-D 2^>nul') do (
    set LAST_BACKUP_DATE=%%A
    goto :FoundLastBackup
)

:FoundLastBackup
if "%LAST_BACKUP_DATE%"=="" (
    call :WriteLog WARNING "No previous backup found, performing full backup instead"
    goto :PerformFullBackup
)

call :WriteLog INFO "Last backup found: %LAST_BACKUP_DATE%"

REM Copy only files newer than the last backup (simplified approach)
forfiles /p "%SOURCE_DIR%" /s /m *.* /C "cmd /c if @isdir==FALSE xcopy @path \"%BACKUP_DIR%\@relpath\" /Y /C /H /K" 2>nul

call :WriteLog INFO "Incremental backup completed"
goto :PostBackupTasks

:PerformDifferentialBackup
REM Function: PerformDifferentialBackup - Performs a differential backup
call :WriteLog INFO "Performing differential backup"

REM Find the last full backup for differential comparison
set LAST_FULL_BACKUP=
for /f "delims=" %%A in ('dir "%DESTINATION_DIR%" /B /AD /O-D ^| findstr /C:"_FULL"') do (
    set LAST_FULL_BACKUP=%%A
    goto :FoundFullBackup
)

:FoundFullBackup
if "%LAST_FULL_BACKUP%"=="" (
    call :WriteLog WARNING "No full backup found, performing full backup instead"
    goto :PerformFullBackup
)

call :WriteLog INFO "Last full backup found: %LAST_FULL_BACKUP%"

REM Copy files modified since the last full backup (simplified)
robocopy "%SOURCE_DIR%" "%BACKUP_DIR%" /E /COPY:DAT /R:3 /W:10 /MT:4 /LOG:"%~dp0temp\robocopy_log.txt"

set ROBOCOPY_EXIT_CODE=%errorlevel%
if %ROBOCOPY_EXIT_CODE% GEQ 8 (
    call :WriteLog ERROR "Differential backup failed with exit code %ROBOCOPY_EXIT_CODE%"
) else (
    call :WriteLog INFO "Differential backup completed with exit code %ROBOCOPY_EXIT_CODE%"
)

goto :PostBackupTasks

:PostBackupTasks
REM Function: PostBackupTasks - Performs post-backup operations
call :WriteLog INFO "Performing post-backup tasks"

REM Compress backup if requested
if "%COMPRESS_BACKUPS%"=="1" (
    call :CompressBackup "%BACKUP_DIR%"
)

REM Verify backup if requested
if "%VERIFY_BACKUPS%"=="1" (
    call :VerifyBackup "%BACKUP_DIR%"
)

REM Clean up old backups
call :CleanupOldBackups

REM Update backup statistics
call :UpdateBackupStatistics

goto :eof

:CompressBackup
REM Function: CompressBackup - Compresses the backup directory
call :WriteLog INFO "Compressing backup: %1"

set ARCHIVE_NAME=%~1.7z
set COMPRESS_CMD=7z a -t7z -mx=7 -mfb=64 -md=32m -ms=on "%ARCHIVE_NAME%" "%~1\*"

call :WriteLog DEBUG "Compression command: %COMPRESS_CMD%"

%COMPRESS_CMD% > "%~dp0temp\compress_output.txt" 2>&1

if errorlevel 1 (
    call :WriteLog ERROR "Backup compression failed"
    type "%~dp0temp\compress_output.txt" >> "%ERROR_LOG%"
) else (
    call :WriteLog INFO "Backup compressed successfully: %ARCHIVE_NAME%"
    
    REM Remove uncompressed directory to save space
    call :WriteLog INFO "Removing uncompressed backup directory"
    rmdir /s /q "%~1" 2>nul
    
    if errorlevel 1 (
        call :WriteLog WARNING "Failed to remove uncompressed backup directory"
    )
)

goto :eof

:VerifyBackup
REM Function: VerifyBackup - Verifies the integrity of the backup
call :WriteLog INFO "Verifying backup: %1"

if exist "%~1.7z" (
    REM Verify compressed archive
    7z t "%~1.7z" > "%~dp0temp\verify_output.txt" 2>&1
    
    if errorlevel 1 (
        call :WriteLog ERROR "Backup verification failed for compressed archive"
        type "%~dp0temp\verify_output.txt" >> "%ERROR_LOG%"
    ) else (
        call :WriteLog INFO "Backup verification successful for compressed archive"
    )
) else if exist "%~1" (
    REM Verify uncompressed directory
    call :WriteLog INFO "Verifying uncompressed backup directory"
    
    REM Simple verification by comparing file counts
    for /f %%A in ('dir "%SOURCE_DIR%" /s /a-d ^| findstr /C:"File(s)"') do set SOURCE_COUNT=%%A
    for /f %%B in ('dir "%~1" /s /a-d ^| findstr /C:"File(s)"') do set BACKUP_COUNT=%%B
    
    if "%SOURCE_COUNT%"=="%BACKUP_COUNT%" (
        call :WriteLog INFO "Backup verification successful - File counts match"
    ) else (
        call :WriteLog WARNING "Backup verification warning - File counts differ (Source: %SOURCE_COUNT%, Backup: %BACKUP_COUNT%)"
    )
) else (
    call :WriteLog ERROR "Backup directory or archive not found for verification"
)

goto :eof

:CleanupOldBackups
REM Function: CleanupOldBackups - Removes old backups based on retention policy
call :WriteLog INFO "Cleaning up old backups (retention: %RETENTION_DAYS% days)"

REM Calculate cutoff date
set /a CUTOFF_DAYS=%RETENTION_DAYS%

REM Use PowerShell for date calculation (more reliable than batch date arithmetic)
powershell -Command "$cutoffDate = (Get-Date).AddDays(-%CUTOFF_DAYS%); Get-ChildItem '%DESTINATION_DIR%' | Where-Object { $_.CreationTime -lt $cutoffDate } | ForEach-Object { Write-Output $_.Name }" > "%~dp0temp\old_backups.txt"

set OLD_BACKUPS_REMOVED=0

for /f "delims=" %%A in ('%~dp0temp\old_backups.txt') do (
    set OLD_BACKUP_PATH=%DESTINATION_DIR%\%%A
    call :WriteLog INFO "Removing old backup: !OLD_BACKUP_PATH!"
    
    if exist "!OLD_BACKUP_PATH!.7z" (
        del /q "!OLD_BACKUP_PATH!.7z" 2>nul
    )
    
    if exist "!OLD_BACKUP_PATH!" (
        rmdir /s /q "!OLD_BACKUP_PATH!" 2>nul
    )
    
    set /a OLD_BACKUPS_REMOVED+=1
)

call :WriteLog INFO "Cleanup completed - %OLD_BACKUPS_REMOVED% old backup(s) removed"
goto :eof

:UpdateBackupStatistics
REM Function: UpdateBackupStatistics - Updates backup statistics file
call :WriteLog INFO "Updating backup statistics"

set END_TIME=%TIME%

REM Calculate duration (simplified)
call :CalculateDuration "%START_TIME%" "%END_TIME%"

REM Write statistics to CSV file
if not exist "%STATS_FILE%" (
    echo Date,Time,Type,Status,Files_Copied,Files_Skipped,Bytes_Copied,Errors,Duration_Minutes > "%STATS_FILE%"
)

echo %DATE%,%TIME:~0,8%,%BACKUP_TYPE%,SUCCESS,%FILES_COPIED%,%FILES_SKIPPED%,%BYTES_COPIED%,%ERRORS_ENCOUNTERED%,%DURATION_MINUTES% >> "%STATS_FILE%"

call :WriteLog INFO "Backup statistics updated"
goto :eof

:CalculateDuration
REM Function: CalculateDuration - Calculates backup duration
REM Parameters: %1 = Start time, %2 = End time
REM This is simplified - real implementation would handle time arithmetic properly
set DURATION_MINUTES=5
goto :eof

:SendEmailNotification
REM Function: SendEmailNotification - Sends email notification about backup status
if "%EMAIL_NOTIFICATIONS%"=="0" goto :eof

call :WriteLog INFO "Sending email notification"

REM Create email body
set EMAIL_BODY=%~dp0temp\email_body.txt

(
echo Backup Report - %COMPUTERNAME%
echo.
echo Backup Type: %BACKUP_TYPE%
echo Date/Time: %DATE% %TIME:~0,8%
echo Source: %SOURCE_DIR%
echo Destination: %DESTINATION_DIR%
echo.
echo Results:
echo - Files Copied: %FILES_COPIED%
echo - Files Skipped: %FILES_SKIPPED%
echo - Bytes Copied: %BYTES_COPIED%
echo - Errors: %ERRORS_ENCOUNTERED%
echo.
echo Status: %BACKUP_STATUS%
echo.
if %ERRORS_ENCOUNTERED% GTR 0 (
    echo Error Details:
    type "%ERROR_LOG%"
)
echo.
echo This is an automated message from the Backup Automation System.
) > "%EMAIL_BODY%"

REM Send email using PowerShell (example - requires SMTP configuration)
powershell -Command "Send-MailMessage -SmtpServer 'smtp.company.com' -From 'backup@company.com' -To 'admin@company.com' -Subject 'Backup Report - %COMPUTERNAME%' -Body (Get-Content '%EMAIL_BODY%' | Out-String)" 2>nul

if errorlevel 1 (
    call :WriteLog WARNING "Failed to send email notification"
) else (
    call :WriteLog INFO "Email notification sent successfully"
)

goto :eof

:GenerateBackupReport
REM Function: GenerateBackupReport - Generates comprehensive backup report
call :WriteLog INFO "Generating backup report"

set REPORT_FILE=%~dp0reports\backup_report_%BACKUP_TIMESTAMP%.html

if not exist "%~dp0reports" mkdir "%~dp0reports"

(
echo ^<html^>
echo ^<head^>^<title^>Backup Report - %COMPUTERNAME%^</title^>^</head^>
echo ^<body^>
echo ^<h1^>Backup Report^</h1^>
echo ^<h2^>System Information^</h2^>
echo ^<p^>Computer: %COMPUTERNAME%^</p^>
echo ^<p^>Date: %DATE%^</p^>
echo ^<p^>Time: %TIME:~0,8%^</p^>
echo ^<h2^>Backup Configuration^</h2^>
echo ^<p^>Source Directory: %SOURCE_DIR%^</p^>
echo ^<p^>Destination Directory: %DESTINATION_DIR%^</p^>
echo ^<p^>Backup Type: %BACKUP_TYPE%^</p^>
echo ^<p^>Compression: %COMPRESS_BACKUPS%^</p^>
echo ^<p^>Verification: %VERIFY_BACKUPS%^</p^>
echo ^<h2^>Results^</h2^>
echo ^<p^>Files Copied: %FILES_COPIED%^</p^>
echo ^<p^>Files Skipped: %FILES_SKIPPED%^</p^>
echo ^<p^>Bytes Copied: %BYTES_COPIED%^</p^>
echo ^<p^>Errors Encountered: %ERRORS_ENCOUNTERED%^</p^>
echo ^</body^>^</html^>
) > "%REPORT_FILE%"

call :WriteLog INFO "Backup report generated: %REPORT_FILE%"
goto :eof

:TestBackupSystem
REM Function: TestBackupSystem - Runs system tests and validations
call :WriteLog INFO "Running backup system tests"

echo.
echo Running Backup System Tests...
echo ================================

REM Test 1: Configuration file test
echo Test 1: Configuration File Validation
if exist "%CONFIG_FILE%" (
    echo [PASS] Configuration file exists
) else (
    echo [FAIL] Configuration file missing
)

REM Test 2: Directory access test
echo Test 2: Directory Access
if exist "%SOURCE_DIR%" (
    echo [PASS] Source directory accessible
) else (
    echo [FAIL] Source directory not accessible
)

REM Test 3: Write permissions test
echo Test 3: Write Permissions
echo test > "%DESTINATION_DIR%\write_test.tmp" 2>nul
if exist "%DESTINATION_DIR%\write_test.tmp" (
    echo [PASS] Destination directory writable
    del "%DESTINATION_DIR%\write_test.tmp" 2>nul
) else (
    echo [FAIL] Destination directory not writable
)

REM Test 4: Compression tool test
echo Test 4: Compression Tool
7z > nul 2>&1
if errorlevel 1 (
    echo [WARN] 7-Zip not found in PATH
) else (
    echo [PASS] 7-Zip compression tool available
)

REM Test 5: Logging system test
echo Test 5: Logging System
call :WriteLog INFO "Test log entry"
if exist "%LOG_FILE%" (
    echo [PASS] Logging system functional
) else (
    echo [FAIL] Logging system not working
)

echo.
echo Test Summary Complete
echo ================================

goto :eof

:DisplaySystemInfo
REM Function: DisplaySystemInfo - Displays system information
echo.
echo System Information:
echo ===================
echo Computer Name: %COMPUTERNAME%
echo User Name: %USERNAME%
echo Domain: %USERDOMAIN%
echo OS: %OS%
echo Processor: %PROCESSOR_IDENTIFIER%
echo Architecture: %PROCESSOR_ARCHITECTURE%
echo Number of Processors: %NUMBER_OF_PROCESSORS%
echo Current Directory: %CD%
echo Script Location: %~dp0
echo.

REM Display disk space information
for %%A in (C D E F) do (
    if exist %%A:\ (
        for /f "tokens=3" %%B in ('dir %%A:\ ^| find "bytes free"') do (
            echo Drive %%A: Free Space: %%B bytes
        )
    )
)

echo.
goto :eof

:ShowStatistics
REM Function: ShowStatistics - Shows backup statistics summary
echo.
echo Backup Statistics Summary:
echo ==========================
if exist "%STATS_FILE%" (
    echo Reading statistics from: %STATS_FILE%
    echo.
    
    REM Display last 10 backup entries
    echo Last 10 Backup Operations:
    echo ---------------------------
    more /E +1 "%STATS_FILE%" | tail -10
) else (
    echo No statistics file found: %STATS_FILE%
    echo Run a backup operation to generate statistics.
)
echo.
goto :eof

REM ============================================================================
REM Main Program Logic
REM ============================================================================

:Main
REM Main program entry point
call :DisplayHeader
call :WriteLog INFO "Starting %SCRIPT_NAME% v%SCRIPT_VERSION%"

REM Parse command-line arguments
call :ParseArguments %*

REM Load configuration
call :LoadConfiguration

REM Validate configuration
call :ValidateConfiguration

REM Check system resources
call :CheckSystemResources

REM Display current configuration
echo Current Configuration:
echo =====================
echo Source Directory: %SOURCE_DIR%
echo Destination Directory: %DESTINATION_DIR%
echo Backup Type: %BACKUP_TYPE%
echo Retention Days: %RETENTION_DAYS%
echo Compression: %COMPRESS_BACKUPS%
echo Verification: %VERIFY_BACKUPS%
echo Email Notifications: %EMAIL_NOTIFICATIONS%
echo Debug Mode: %DEBUG_MODE%
echo.

REM Perform backup operation
call :PerformBackup

REM Generate reports
call :GenerateBackupReport

REM Send notifications
if "%EMAIL_NOTIFICATIONS%"=="1" (
    if %ERRORS_ENCOUNTERED% GTR 0 (
        set BACKUP_STATUS=FAILED
    ) else (
        set BACKUP_STATUS=SUCCESS
    )
    call :SendEmailNotification
)

REM Display final status
echo.
echo ================================================================================
echo Backup Operation Summary
echo ================================================================================
echo Start Time: %START_TIME%
echo End Time: %TIME:~0,8%
echo Files Copied: %FILES_COPIED%
echo Files Skipped: %FILES_SKIPPED%
echo Bytes Copied: %BYTES_COPIED%
echo Errors Encountered: %ERRORS_ENCOUNTERED%

if %ERRORS_ENCOUNTERED% EQU 0 (
    echo Status: SUCCESS
    call :WriteLog INFO "Backup operation completed successfully"
) else (
    echo Status: COMPLETED WITH ERRORS
    call :WriteLog WARNING "Backup operation completed with %ERRORS_ENCOUNTERED% error(s)"
    echo Check error log: %ERROR_LOG%
)

echo ================================================================================
echo.

REM Demonstrate additional features
if "%DEBUG_MODE%"=="1" (
    echo Debug Information:
    echo ==================
    call :DisplaySystemInfo
    call :ShowStatistics
    call :TestBackupSystem
)

echo Key Batch Scripting Features Demonstrated:
echo ==========================================
echo - Command-line argument processing and validation
echo - Configuration file parsing and management
echo - Advanced file and directory operations
echo - Conditional logic and flow control structures
echo - Loop constructs and iterative processing
echo - Comprehensive error handling and logging
echo - Date/time manipulation and formatting
echo - Environment variable manipulation
echo - System resource monitoring
echo - Integration with external tools (7-Zip, RoboCopy)
echo - Email notification system
echo - HTML report generation
echo - Statistical analysis and tracking
echo - Modular function-based architecture
echo - Advanced batch optimization techniques

:EndScript
call :WriteLog INFO "%SCRIPT_NAME% execution completed"

REM Clean up temporary files
if exist "%~dp0temp\*.txt" del /q "%~dp0temp\*.txt" 2>nul

endlocal
exit /b %ERRORS_ENCOUNTERED%