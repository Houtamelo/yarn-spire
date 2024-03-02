START /B /wait cargo build

echo off

timeout /t 1

echo on

copy /Y /B "%~dp0target\debug\yarn_spire_codegen.exe" "%~dp0tests\yarn_spire_codegen.exe"

cd tests

START /B /wait "generating_code" yarn_spire_codegen.exe

