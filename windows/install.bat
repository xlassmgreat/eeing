@echo off

REG ADD "HKCU\SOFTWARE\Mozilla\NativeMessagingHosts\eeing" /ve /t REG_SZ /d "%CD%\eeing.json" /f
