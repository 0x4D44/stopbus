; Inno Setup Script for Stop The Bus modernization
; Build with: iscc installers\StopTheBus.iss

#define MyAppName "Stop The Bus"
#define AppVersionEnv GetEnv("STOPBUS_APP_VERSION")
#if AppVersionEnv != ""
  #define MyAppVersion AppVersionEnv
#else
  #define MyAppVersion "2.0.0"
#endif
#define MyAppPublisher "Stop The Bus Modernization Team"
#define MyAppExeName "stopbus.exe"

[Setup]
AppId={{4605150A-E1CD-473B-911E-4CF131748B82}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={pf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableDirPage=no
DisableProgramGroupPage=no
OutputBaseFilename=StopTheBus-{#MyAppVersion}-Setup
OutputDir=..\target\installer
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\RELEASE\STOPBUS.TXT"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion
Source: "..\HELP\STOPBUS.HLP"; DestDir: "{app}\help"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commondesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: files; Name: "{app}\README.txt"
Type: dirifempty; Name: "{app}\help"
