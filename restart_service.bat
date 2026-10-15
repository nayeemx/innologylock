@echo off
echo ==========================================
echo  InnologyLock Service Restart
echo  Run as Administrator:
echo    Right-click this file ^> Run as administrator
echo ==========================================
echo.
echo Stopping InnologyLockService...
sc stop InnologyLockService
timeout /t 3 /nobreak >nul

echo Starting InnologyLockService...
sc start InnologyLockService
timeout /t 2 /nobreak >nul

echo.
sc query InnologyLockService
echo.
echo Done! You can close this window.
pause
