!include "nsExec.nsh"

Function .onInit
  nsExec::Exec '"$SYSDIR\taskkill.exe" /f /im innologylock-svc.exe 2>nul'
  nsExec::Exec '"$SYSDIR\taskkill.exe" /f /im innologylock-monitor.exe 2>nul'
  Sleep 300
FunctionEnd
