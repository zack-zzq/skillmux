@echo off
python "%~dp0release.py" %*
exit /b %ERRORLEVEL%
