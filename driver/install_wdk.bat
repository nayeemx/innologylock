@echo off
REM Install WDK for InnologyLock minifilter driver build
REM Run as Administrator

echo ============================================
echo   WDK Installer Helper
echo ============================================
echo.
echo This will download and install the Windows Driver Kit (WDK)
echo required to build the InnologyLock minifilter driver.
echo.
echo Requirements:
echo   - Visual Studio 2022 (or Build Tools) - CHECKED
echo   - Windows SDK 10.0.26100 - CHECKED
echo   - WDK - DOWNLOADING...
echo.

REM Check for VS Build Tools
where msbuild >nul 2>&1
if errorlevel 1 (
    echo ERROR: msbuild not found.
    echo Install Visual Studio Build Tools 2022 first:
    echo https://visualstudio.microsoft.com/downloads/
    pause
    exit /b 1
)

REM Check for SDK
if not exist "C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0" (
    echo WARNING: Windows SDK 10.0.26100.0 not found.
    echo Download from: https://developer.microsoft.com/windows/downloads/windows-sdk/
)

echo.
echo Downloading WDK...
echo.

REM Download WDK installer
set WDK_URL=https://go.microsoft.com/fwlink/?LinkId=2371554
set WDK_INSTALLER=%TEMP%\wdk_install.exe

powershell -Command "Invoke-WebRequest -Uri '%WDK_URL%' -OutFile '%WDK_INSTALLER%' -UseBasicParsing"
if errorlevel 1 (
    echo ERROR: Download failed
    pause
    exit /b 1
)

echo Downloaded. Starting installer...
echo.
echo IMPORTANT: During installation:
echo   1. Accept the license
echo   2. It will detect VS Build Tools and install WDK extension
echo   3. Follow the prompts
echo.

start /wait "%WDK_INSTALLER%"

echo.
echo ============================================
echo   WDK Installation Complete
echo ============================================
echo.
echo Next steps:
echo   1. Open "Developer Command Prompt for VS 2022"
echo   2. cd to D:\projects\code\innologylock\driver
echo   3. Run build.bat
echo   4. Run install.bat (as admin)
echo.

pause
