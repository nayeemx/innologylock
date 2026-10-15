@echo off
REM Build InnologyLock Minifilter Driver
REM Requires: WDK installed, run from "Developer Command Prompt for VS 2022"

setlocal
set PROJECT_DIR=%~dp0

echo === InnologyLock Minifilter Driver Build ===
echo.

REM Check for WDK
where msbuild >nul 2>&1
if errorlevel 1 (
    echo ERROR: msbuild not found. Run from "Developer Command Prompt for VS 2022"
    echo Or install WDK via: winget install Microsoft.WindowsWDK.10.0.26100
    exit /b 1
)

echo Building driver...
cd /d "%PROJECT_DIR%"

msbuild innologylock_filt.vcxproj /t:Build /p:Configuration=Release;Platform=x64;SpectreMitigation=false;EnableTestSign=false;SignMode=NoSign /v:minimal
if errorlevel 1 (
    echo.
    echo BUILD FAILED
    exit /b 1
)

if exist "x64\Release\innologylock_filt.sys" (
    echo.
    echo Build successful!
    echo Driver: x64\Release\innologylock_filt.sys
    echo.
    echo Next steps:
    echo   1. Enable test signing (requires admin):
    echo      bcdedit /set testsigning on
    echo   2. Reboot your computer
    echo   3. Install driver (run install.bat as admin)
    echo.
)

endlocal
