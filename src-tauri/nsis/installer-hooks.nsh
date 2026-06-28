; Aegis NSIS installer hooks — register AegisService + create data dirs.
; Tauri invokes these macros around its own install/uninstall steps.
; Service registration requires a machine-wide (Administrator) install; on a
; per-user install the sc.exe calls fail harmlessly and only the UI is set up.

!define AEGIS_SERVICE "AegisService"
!define AEGIS_DATA "$COMMONFILES\..\Aegis" ; expands to ProgramData via APPDATA below
!define AEGIS_PROGRAMDATA "$PROGRAMDATA\Aegis"

!macro NSIS_HOOK_POSTINSTALL
  ; --- data directory layout (preserved across upgrades) ---
  CreateDirectory "${AEGIS_PROGRAMDATA}"
  CreateDirectory "${AEGIS_PROGRAMDATA}\Updates"
  CreateDirectory "${AEGIS_PROGRAMDATA}\Quarantine"
  CreateDirectory "${AEGIS_PROGRAMDATA}\Logs"
  CreateDirectory "${AEGIS_PROGRAMDATA}\Database"

  FileOpen $0 "${AEGIS_PROGRAMDATA}\Logs\install.log" a
  FileSeek $0 0 END
  FileWrite $0 "[install] $INSTDIR$\r$\n"
  FileClose $0

  ; --- register the Windows service (idempotent: delete then create) ---
  nsExec::ExecToLog 'sc.exe stop ${AEGIS_SERVICE}'
  nsExec::ExecToLog 'sc.exe delete ${AEGIS_SERVICE}'
  nsExec::ExecToLog 'sc.exe create ${AEGIS_SERVICE} binPath= "\"$INSTDIR\aegis-service.exe\"" start= auto DisplayName= "Aegis Security Service"'
  nsExec::ExecToLog 'sc.exe description ${AEGIS_SERVICE} "Aegis Antivirus background protection (scanning, real-time, quarantine, updates)."'
  ; --- crash recovery: restart on 1st/2nd failure, reset count daily ---
  nsExec::ExecToLog 'sc.exe failure ${AEGIS_SERVICE} reset= 86400 actions= restart/60000/restart/60000/restart/120000'
  nsExec::ExecToLog 'sc.exe failureflag ${AEGIS_SERVICE} 1'
  ; --- start now ---
  nsExec::ExecToLog 'sc.exe start ${AEGIS_SERVICE}'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  FileOpen $0 "${AEGIS_PROGRAMDATA}\Logs\uninstall.log" a
  FileSeek $0 0 END
  FileWrite $0 "[uninstall] stopping service$\r$\n"
  FileClose $0
  nsExec::ExecToLog 'sc.exe stop ${AEGIS_SERVICE}'
  nsExec::ExecToLog 'sc.exe delete ${AEGIS_SERVICE}'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; User data under %PROGRAMDATA%\Aegis is intentionally preserved.
  ; Run deploy/cleanup-data.ps1 for a full wipe.
!macroend
