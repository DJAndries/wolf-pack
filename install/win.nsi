;NSIS Modern User Interface
	!define REGPATH_UNINSTSUBKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${NAME}"
	!define NAME "Wolf Pack"

;--------------------------------
;Include Modern UI

	!include "MUI2.nsh"
	!include Integration.nsh

;--------------------------------
;General

	;Name and file
	Name "${NAME}"
	OutFile "..\target\release\WolfPack-Install.exe"
	Unicode True

	;Default installation folder
	InstallDir "$ProgramFiles64\${Name}"

	;Get installation folder from registry if available
	InstallDirRegKey HKCU "${REGPATH_UNINSTSUBKEY}" "UninstallString"

	;Request application privileges for Windows Vista
	RequestExecutionLevel Admin

	VIAddVersionKey "ProductName" "Wolf Pack"
	VIFileVersion "${VERSION}.0"
	VIProductVersion "${VERSION}.0"

;--------------------------------
;Interface Settings

	!define MUI_HEADERIMAGE
	!define MUI_HEADERIMAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Header\nsis3-metro.bmp"
	!define MUI_WELCOMEFINISHPAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Wizard\nsis3-branding.bmp"
	!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${NSISDIR}\Contrib\Graphics\Wizard\nsis3-branding.bmp"
	!define MUI_ABORTWARNING

;--------------------------------
;Pages

	!insertmacro MUI_PAGE_WELCOME
	!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
	!insertmacro MUI_PAGE_COMPONENTS
	!insertmacro MUI_PAGE_DIRECTORY
	!insertmacro MUI_PAGE_INSTFILES
	!insertmacro MUI_PAGE_FINISH

	!insertmacro MUI_UNPAGE_WELCOME
	!insertmacro MUI_UNPAGE_CONFIRM
	!insertmacro MUI_UNPAGE_INSTFILES
	!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
;Languages

	!insertmacro MUI_LANGUAGE "English"

;--------------------------------
;Installer Sections

Section "Wolf Pack" SecMain

	SectionIn RO
	SetOutPath "$INSTDIR"

	SetOutPath $InstDir
	WriteUninstaller "$InstDir\Uninst.exe"
	WriteRegStr HKLM "${REGPATH_UNINSTSUBKEY}" "DisplayName" "${NAME}"
	WriteRegStr HKLM "${REGPATH_UNINSTSUBKEY}" "DisplayIcon" "$InstDir\wolf-pack.exe,0"
	WriteRegStr HKLM "${REGPATH_UNINSTSUBKEY}" "DisplayVersion" "${VERSION}"
	WriteRegStr HKLM "${REGPATH_UNINSTSUBKEY}" "Publisher" "Darnell Andries"
	WriteRegStr HKLM "${REGPATH_UNINSTSUBKEY}" "UninstallString" '"$InstDir\Uninst.exe"'
	WriteRegDWORD HKLM "${REGPATH_UNINSTSUBKEY}" "NoModify" 1
	WriteRegDWORD HKLM "${REGPATH_UNINSTSUBKEY}" "NoRepair" 1

	File /r /x *.blend "..\models"
	File /r "..\textures"
	File /r "..\audio"
	File "..\LICENSE"
	File "/oname=$InstDir\wolf-pack.exe" "..\target\release\wolf-pack.exe"

	;Create uninstaller
	WriteUninstaller "$INSTDIR\Uninst.exe"

SectionEnd

Section "Start Menu shortcut"
	CreateShortcut "$SMPrograms\${NAME}.lnk" "$InstDir\wolf-pack.exe"
SectionEnd

;--------------------------------
;Descriptions

	;Language strings
	LangString DESC_SecMain ${LANG_ENGLISH} "The application binary and resources."

	;Assign language strings to sections
	!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
		!insertmacro MUI_DESCRIPTION_TEXT ${SecMain} $(DESC_SecMain)
	!insertmacro MUI_FUNCTION_DESCRIPTION_END

;--------------------------------
;Uninstaller Section

Section "Uninstall"
	${UnpinShortcut} "$SMPrograms\${NAME}.lnk"
	Delete "$SMPrograms\${NAME}.lnk"

	RMDir /r "$INSTDIR"
	DeleteRegKey HKLM "${REGPATH_UNINSTSUBKEY}"

SectionEnd
