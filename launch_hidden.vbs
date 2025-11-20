Set WshShell = CreateObject("WScript.Shell")
Set FSO = CreateObject("Scripting.FileSystemObject")
CurrentDir = FSO.GetParentFolderName(WScript.ScriptFullName)
WshShell.CurrentDirectory = CurrentDir
WshShell.Run Chr(34) & CurrentDir & "\osu-scrobbler.exe" & Chr(34), 0
Set WshShell = Nothing
Set FSO = Nothing