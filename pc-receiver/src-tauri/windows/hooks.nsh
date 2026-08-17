!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog '"$SYSDIR\netsh.exe" advfirewall firewall delete rule name="SMS Bridge Receiver TCP"'
  nsExec::ExecToLog '"$SYSDIR\netsh.exe" advfirewall firewall delete rule name="SMS Bridge Receiver UDP"'
  nsExec::ExecToLog '"$SYSDIR\netsh.exe" advfirewall firewall add rule name="SMS Bridge Receiver TCP" dir=in action=allow protocol=TCP localport=8899 profile=private'
  nsExec::ExecToLog '"$SYSDIR\netsh.exe" advfirewall firewall add rule name="SMS Bridge Receiver UDP" dir=in action=allow protocol=UDP localport=8899 profile=private'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    nsExec::ExecToLog '"$SYSDIR\netsh.exe" advfirewall firewall delete rule name="SMS Bridge Receiver TCP"'
    nsExec::ExecToLog '"$SYSDIR\netsh.exe" advfirewall firewall delete rule name="SMS Bridge Receiver UDP"'
    SetShellVarContext current
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "SMS Bridge Receiver"
    RMDir /r "$APPDATA\com.smsbridge.receiver"
    SetShellVarContext all
  ${EndIf}
!macroend
