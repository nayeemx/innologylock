@echo off
REM InnologyLock Minifilter Driver Installation
REM Run as Administrator

setlocal EnableDelayedExpansion

echo ============================================
echo   InnologyLock Filter Driver Installer
echo ============================================
echo.

REM Check admin
net session >nul 2>&1
if errorlevel 1 (
    echo ERROR: Run this script as Administrator
    echo Right-click ^> Run as administrator
    pause
    exit /b 1
)

REM Check if driver exists
set DRIVER_PATH=%~dp0x64\release\innologylock_filt.sys
if not exist "%DRIVER_PATH%" (
    echo Driver not found at: %DRIVER_PATH%
    echo.
    echo Build the driver first:
    echo   1. Open "Developer Command Prompt for VS 2022"
    echo   2. cd to driver directory
    echo   3. msbuild /t:build /p:Configuration=release,Platform=x64
    echo.
    echo Or use the build.bat script
    pause
    exit /b 1
)

echo Installing filter driver...
echo Source: %DRIVER_PATH%
echo.

REM Copy driver to System32\drivers
copy /Y "%DRIVER_PATH%" "%SystemRoot%\System32\drivers\innologylock_filt.sys"
if errorlevel 1 (
    echo ERROR: Failed to copy driver to System32\drivers
    pause
    exit /b 1
)

REM Stop existing service if running
sc stop InnologyLockFilt >nul 2>&1
sc delete InnologyLockFilt >nul 2>&1

REM Create the driver service
sc create InnologyLockFilt type= kernel binPath= "%SystemRoot%\System32\drivers\innologylock_filt.sys" start= demand
if errorlevel 1 (
    echo ERROR: Failed to create driver service
    pause
    exit /b 1
)

echo.
echo Driver service created.
echo.

REM Start the driver
echo Starting filter driver...
sc start InnologyLockFilt
if errorlevel 1 (
    echo.
    echo WARNING: Driver may need test signing enabled.
    echo Run: bcdedit /set testsigning on
    echo Then reboot and try again.
    echo.
)

echo.
echo ============================================
echo   Installation complete!
echo ============================================
echo.
echo To enable test signing (if needed):
echo   bcdedit /set testsigning on
echo   (Reboot required)
echo.
echo To uninstall:
echo   sc stop InnologyLockFilt
echo   sc delete InnologyLockFilt
echo.
pause
