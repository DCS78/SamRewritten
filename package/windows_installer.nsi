
; SamRewritten NSIS Installer Script
; ----------------------------------
; Modernized NSIS script for building the SamRewritten Windows installer.
; Encoding: UTF-8

Name "SamRewritten"
Outfile "SamRewritten-installer.exe"
InstallDir "$PROGRAMFILES64\SamRewritten"
RequestExecutionLevel admin ; Request application privileges

; --- Application Metadata ---
!define APP_NAME "SamRewritten"
!define APP_VERSION "1.0.0"
!define APP_PUBLISHER "Sam Authors"
!define APP_EXE "samrewritten.exe"

; --- Modern UI ---
!include "MUI2.nsh"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Run SamRewritten now"

; Installer pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Language selection
!insertmacro MUI_LANGUAGE "English"

; --- Install Section ---
Section "Install"
  SetOutPath $INSTDIR

  ; Main application files
  File "..\SamRewritten-windows-x86_64\${APP_EXE}"
  File "..\SamRewritten-windows-x86_64\README.txt"
  File /a "..\SamRewritten-windows-x86_64\bin\*.*"

  ; Start menu shortcut
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"

  ; Registry entries for uninstaller
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "InstallLocation" "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

; --- Post-Install: Launch Option ---
Function .onInstSuccess
  ; The MUI_FINISHPAGE_RUN macro provides a "Run now" checkbox on the finish page.
FunctionEnd

; --- Uninstall Section ---
Section "Uninstall"
  ; Remove application files
  Delete "$INSTDIR\*.*"
  RMDir /r "$INSTDIR\bin"
  RMDir "$INSTDIR"

  ; Remove start menu shortcut
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"

  ; Remove registry key
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
SectionEnd